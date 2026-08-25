pub mod models;

use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use uuid::Uuid;

use models::{CategoryRecord, CategoryWithCount, ClassificationRule, FileRecord, MoveLogRecord};

pub struct Database {
    pub conn: Connection,
}

impl Database {
    /// Abre (criando se necessario) o banco em `./data/profile.db`, sempre
    /// relativo a pasta do executavel. E isso que torna o app "portatil":
    /// mover a pasta inteira (exe + data/) leva o perfil do usuario junto.
    pub fn open_beside_executable() -> anyhow::Result<Self> {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let data_dir = exe_dir.join("data");
        std::fs::create_dir_all(&data_dir)?;
        let db_path = data_dir.join("profile.db");
        let conn = Connection::open(db_path)?;
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;",
        )?;
        conn.execute_batch(include_str!("schema.sql"))?;
        Ok(Self { conn })
    }

    #[cfg(test)]
    pub fn open_in_memory_for_tests() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;",
        )?;
        conn.execute_batch(include_str!("schema.sql"))?;
        Ok(Self { conn })
    }

    /// Cria uma nova sessao de varredura
    pub fn create_session(&self, root_path: &str) -> anyhow::Result<String> {
        let session_id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO scan_sessions (id, root_path, started_at, status) VALUES (?1, ?2, ?3, 'running')",
            params![session_id, root_path, now],
        )?;
        Ok(session_id)
    }

    /// Finaliza uma sessao de varredura
    pub fn finish_session(&self, session_id: &str, files_scanned: usize, status: &str) -> anyhow::Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE scan_sessions SET finished_at = ?1, files_scanned = ?2, status = ?3 WHERE id = ?4",
            params![now, files_scanned as i64, status, session_id],
        )?;
        Ok(())
    }

    /// Insere ou atualiza arquivos escaneados
    pub fn insert_scanned_files(&self, session_id: &str, files: &[crate::commands::scan::FileMeta]) -> anyhow::Result<Vec<String>> {
        let mut file_ids = Vec::with_capacity(files.len());
        let now = Utc::now().to_rfc3339();

        let tx = self.conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO files (
                    id, session_id, original_path, filename, extension_declared,
                    extension_detected, size_bytes, content_hash, created_at, modified_at, last_scanned_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                ON CONFLICT(original_path) DO UPDATE SET
                    session_id = excluded.session_id,
                    filename = excluded.filename,
                    extension_declared = excluded.extension_declared,
                    extension_detected = excluded.extension_detected,
                    size_bytes = excluded.size_bytes,
                    modified_at = excluded.modified_at,
                    last_scanned_at = excluded.last_scanned_at",
            )?;

            for file in files {
                // Verificar se ja existe ID para esse path
                let existing_id: Option<String> = tx
                    .query_row("SELECT id FROM files WHERE original_path = ?1", params![file.path], |row| row.get(0))
                    .ok();

                let file_id = existing_id.unwrap_or_else(|| Uuid::new_v4().to_string());
                stmt.execute(params![
                    file_id,
                    session_id,
                    file.path,
                    file.filename,
                    file.extension_declared,
                    file.extension_detected,
                    file.size_bytes as i64,
                    None::<String>,
                    file.created_at,
                    file.modified_at,
                    now
                ])?;
                file_ids.push(file_id);
            }
        }
        tx.commit()?;
        Ok(file_ids)
    }

    /// Busca todos os arquivos de uma sessao
    pub fn get_files_by_session(&self, session_id: &str) -> anyhow::Result<Vec<FileRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, original_path, filename, extension_declared,
                    extension_detected, size_bytes, content_hash, created_at, modified_at, last_scanned_at
             FROM files WHERE session_id = ?1 ORDER BY filename ASC",
        )?;

        let rows = stmt.query_map(params![session_id], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                session_id: row.get(1)?,
                original_path: row.get(2)?,
                filename: row.get(3)?,
                extension_declared: row.get(4)?,
                extension_detected: row.get(5)?,
                size_bytes: row.get(6)?,
                content_hash: row.get(7)?,
                created_at: row.get(8)?,
                modified_at: row.get(9)?,
                last_scanned_at: row.get(10)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Busca ou cria uma categoria pelo nome
    pub fn get_or_create_category(&self, name: &str, created_by: &str, color: Option<&str>) -> anyhow::Result<CategoryRecord> {
        let trimmed_name = name.trim();
        let slug = slugify(trimmed_name);

        let existing = self.conn.query_row(
            "SELECT id, name, slug, color, parent_id, created_by, created_at FROM categories WHERE slug = ?1 OR LOWER(name) = LOWER(?2)",
            params![slug, trimmed_name],
            |row| {
                Ok(CategoryRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    slug: row.get(2)?,
                    color: row.get(3)?,
                    parent_id: row.get(4)?,
                    created_by: row.get(5)?,
                    created_at: row.get(6)?,
                })
            },
        );

        if let Ok(cat) = existing {
            return Ok(cat);
        }

        let id = Uuid::new_v4().to_string();
        let default_color = color
            .map(|c| c.to_string())
            .unwrap_or_else(|| pick_category_color(trimmed_name));
        let now = Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO categories (id, name, slug, color, parent_id, created_by, created_at)
             VALUES (?1, ?2, ?3, ?4, NULL, ?5, ?6)",
            params![id, trimmed_name, slug, default_color, created_by, now],
        )?;

        Ok(CategoryRecord {
            id,
            name: trimmed_name.to_string(),
            slug,
            color: Some(default_color),
            parent_id: None,
            created_by: created_by.to_string(),
            created_at: now,
        })
    }

    /// Lista todas as categorias com a contagem de arquivos
    pub fn list_categories(&self) -> anyhow::Result<Vec<CategoryWithCount>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id, c.name, c.slug, c.color, c.parent_id, c.created_by,
                    COALESCE(COUNT(DISTINCT fc.file_id), 0) AS file_count
             FROM categories c
             LEFT JOIN file_categories fc ON fc.category_id = c.id
             GROUP BY c.id
             ORDER BY file_count DESC, c.name ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(CategoryWithCount {
                id: row.get(0)?,
                name: row.get(1)?,
                slug: row.get(2)?,
                color: row.get(3)?,
                parent_id: row.get(4)?,
                created_by: row.get(5)?,
                file_count: row.get(6)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Renomeia uma categoria
    pub fn rename_category(&self, id: &str, new_name: &str) -> anyhow::Result<()> {
        let trimmed = new_name.trim();
        let slug = slugify(trimmed);
        self.conn.execute(
            "UPDATE categories SET name = ?1, slug = ?2 WHERE id = ?3",
            params![trimmed, slug, id],
        )?;
        Ok(())
    }

    /// Mescla categoria source_id na target_id
    pub fn merge_categories(&self, source_id: &str, target_id: &str) -> anyhow::Result<()> {
        if source_id == target_id {
            return Ok(());
        }
        let tx = self.conn.unchecked_transaction()?;
        // Reatribuir associacoes de arquivo
        tx.execute(
            "INSERT OR IGNORE INTO file_categories (file_id, category_id, confidence, assigned_by, assigned_at)
             SELECT file_id, ?1, confidence, assigned_by, assigned_at
             FROM file_categories WHERE category_id = ?2",
            params![target_id, source_id],
        )?;
        tx.execute("DELETE FROM file_categories WHERE category_id = ?1", params![source_id])?;

        // Reatribuir regras de classificacao
        tx.execute(
            "UPDATE classification_rules SET category_id = ?1 WHERE category_id = ?2",
            params![target_id, source_id],
        )?;

        // Reatribuir correcoes de usuario
        tx.execute(
            "UPDATE user_corrections SET new_category_id = ?1 WHERE new_category_id = ?2",
            params![target_id, source_id],
        )?;

        // Deletar categoria de origem
        tx.execute("DELETE FROM categories WHERE id = ?1", params![source_id])?;

        tx.commit()?;
        Ok(())
    }

    /// Exclui uma categoria e desassocia arquivos
    pub fn delete_category(&self, id: &str) -> anyhow::Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM file_categories WHERE category_id = ?1", params![id])?;
        tx.execute("DELETE FROM classification_rules WHERE category_id = ?1", params![id])?;
        tx.execute("DELETE FROM categories WHERE id = ?1", params![id])?;
        tx.commit()?;
        Ok(())
    }

    /// Associa arquivo a categoria
    pub fn assign_file_category(&self, file_id: &str, category_id: &str, confidence: f32, assigned_by: &str) -> anyhow::Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO file_categories (file_id, category_id, confidence, assigned_by, assigned_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(file_id, category_id) DO UPDATE SET
                confidence = excluded.confidence,
                assigned_by = excluded.assigned_by,
                assigned_at = excluded.assigned_at",
            params![file_id, category_id, confidence, assigned_by, now],
        )?;
        Ok(())
    }

    /// Lista todas as regras de classificacao aprendidas e manuais
    pub fn get_classification_rules(&self) -> anyhow::Result<Vec<ClassificationRule>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, pattern_type, pattern_value, category_id, confidence_weight, created_from, hit_count, created_at, updated_at
             FROM classification_rules
             ORDER BY confidence_weight DESC, hit_count DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(ClassificationRule {
                id: row.get(0)?,
                pattern_type: row.get(1)?,
                pattern_value: row.get(2)?,
                category_id: row.get(3)?,
                confidence_weight: row.get(4)?,
                created_from: row.get(5)?,
                hit_count: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Cria ou reforca uma regra de classificacao
    pub fn upsert_learned_rule(
        &self,
        pattern_type: &str,
        pattern_value: &str,
        category_id: &str,
        confidence_weight: f32,
        created_from: &str,
    ) -> anyhow::Result<()> {
        let now = Utc::now().to_rfc3339();
        let existing_id: Option<String> = self.conn.query_row(
            "SELECT id FROM classification_rules WHERE pattern_type = ?1 AND LOWER(pattern_value) = LOWER(?2)",
            params![pattern_type, pattern_value],
            |row| row.get(0),
        ).ok();

        if let Some(id) = existing_id {
            self.conn.execute(
                "UPDATE classification_rules SET
                    category_id = ?1,
                    confidence_weight = MIN(1.0, confidence_weight + 0.1),
                    hit_count = hit_count + 1,
                    updated_at = ?2
                 WHERE id = ?3",
                params![category_id, now, id],
            )?;
        } else {
            let id = Uuid::new_v4().to_string();
            self.conn.execute(
                "INSERT INTO classification_rules (id, pattern_type, pattern_value, category_id, confidence_weight, created_from, hit_count, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 1, ?7, ?8)",
                params![id, pattern_type, pattern_value, category_id, confidence_weight, created_from, now, now],
            )?;
        }
        Ok(())
    }

    /// Registra correcao manual do usuario
    pub fn record_user_correction(&self, file_id: &str, old_category_id: Option<&str>, new_category_id: &str) -> anyhow::Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO user_corrections (id, file_id, old_category_id, new_category_id, corrected_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, file_id, old_category_id, new_category_id, now],
        )?;
        Ok(())
    }

    /// Registra operacao no move_log (ANTES do move real)
    pub fn record_move(&self, session_id: &str, file_id: &str, from_path: &str, to_path: &str) -> anyhow::Result<String> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO move_log (id, session_id, file_id, from_path, to_path, moved_at, undone)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)",
            params![id, session_id, file_id, from_path, to_path, now],
        )?;
        Ok(id)
    }

    /// Busca operacoes de move ativas para uma sessao
    pub fn get_session_moves(&self, session_id: &str) -> anyhow::Result<Vec<MoveLogRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, session_id, file_id, from_path, to_path, moved_at, undone
             FROM move_log WHERE session_id = ?1 AND undone = 0 ORDER BY moved_at DESC",
        )?;

        let rows = stmt.query_map(params![session_id], |row| {
            Ok(MoveLogRecord {
                id: row.get(0)?,
                session_id: row.get(1)?,
                file_id: row.get(2)?,
                from_path: row.get(3)?,
                to_path: row.get(4)?,
                moved_at: row.get(5)?,
                undone: row.get(6)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    /// Busca operacoes de move da ultima sessao que contem registros nao desfeitos
    pub fn get_last_session_moves(&self) -> anyhow::Result<Vec<MoveLogRecord>> {
        let last_session_id: Option<String> = self.conn.query_row(
            "SELECT session_id FROM move_log WHERE undone = 0 ORDER BY moved_at DESC LIMIT 1",
            [],
            |row| row.get(0),
        ).ok();

        if let Some(session_id) = last_session_id {
            self.get_session_moves(&session_id)
        } else {
            Ok(Vec::new())
        }
    }

    /// Marca operacao como desfeita (undone = 1)
    pub fn mark_move_undone(&self, id: &str) -> anyhow::Result<()> {
        self.conn.execute("UPDATE move_log SET undone = 1 WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Busca configuracao em settings
    pub fn get_setting(&self, key: &str) -> anyhow::Result<Option<String>> {
        let val = self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        ).ok();
        Ok(val)
    }

    /// Salva configuracao em settings
    pub fn set_setting(&self, key: &str, value: &str) -> anyhow::Result<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }
}

/// Helper para converter strings em slugs limpos
fn slugify(text: &str) -> String {
    let mut slug = String::new();
    for c in text.to_lowercase().chars() {
        if c.is_alphanumeric() {
            slug.push(c);
        } else if c.is_whitespace() || c == '-' || c == '_' {
            if !slug.ends_with('-') && !slug.is_empty() {
                slug.push('-');
            }
        }
    }
    if slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "categoria".to_string()
    } else {
        slug
    }
}

/// Gera cores consistentes e bonitas a partir do nome da categoria
fn pick_category_color(name: &str) -> String {
    let palette = [
        "#3B82F6", // Blue
        "#10B981", // Emerald
        "#8B5CF6", // Violet
        "#F59E0B", // Amber
        "#EC4899", // Pink
        "#06B6D4", // Cyan
        "#6366F1", // Indigo
        "#14B8A6", // Teal
        "#F97316", // Orange
        "#84CC16", // Lime
        "#A855F7", // Purple
        "#0EA5E9", // Sky
    ];
    let hash: usize = name.bytes().fold(0usize, |acc, b| acc.wrapping_add(b as usize));
    palette[hash % palette.len()].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::scan::FileMeta;

    #[test]
    fn test_db_operations() {
        let db = Database::open_in_memory_for_tests().unwrap();

        // 1. Criar sessao e inserir arquivos
        let session_id = db.create_session(r"C:\Users\Test\Downloads").unwrap();
        assert!(!session_id.is_empty());

        let files = vec![
            FileMeta {
                path: r"C:\Users\Test\Downloads\fatura1.pdf".to_string(),
                filename: "fatura1.pdf".to_string(),
                extension_declared: Some("pdf".to_string()),
                extension_detected: Some("pdf".to_string()),
                size_bytes: 1000,
                created_at: "2026-01-01T00:00:00Z".to_string(),
                modified_at: "2026-01-01T00:00:00Z".to_string(),
            },
            FileMeta {
                path: r"C:\Users\Test\Downloads\contrato.docx".to_string(),
                filename: "contrato.docx".to_string(),
                extension_declared: Some("docx".to_string()),
                extension_detected: Some("docx".to_string()),
                size_bytes: 2000,
                created_at: "2026-01-01T00:00:00Z".to_string(),
                modified_at: "2026-01-01T00:00:00Z".to_string(),
            },
        ];

        let file_ids = db.insert_scanned_files(&session_id, &files).unwrap();
        assert_eq!(file_ids.len(), 2);

        let retrieved = db.get_files_by_session(&session_id).unwrap();
        assert_eq!(retrieved.len(), 2);

        // 2. Categorias: criar, listar, renomear
        let cat1 = db.get_or_create_category("Boletos de Luz", "auto", None).unwrap();
        let cat2 = db.get_or_create_category("Contratos", "user", None).unwrap();

        db.assign_file_category(&file_ids[0], &cat1.id, 0.95, "heuristic").unwrap();
        db.assign_file_category(&file_ids[1], &cat2.id, 0.85, "embedding").unwrap();

        let categories = db.list_categories().unwrap();
        assert_eq!(categories.len(), 2);

        db.rename_category(&cat1.id, "Faturas de Energia").unwrap();
        let updated_cats = db.list_categories().unwrap();
        assert!(updated_cats.iter().any(|c| c.name == "Faturas de Energia"));

        // 3. Mesclar categorias
        db.merge_categories(&cat1.id, &cat2.id).unwrap();
        let after_merge = db.list_categories().unwrap();
        assert_eq!(after_merge.len(), 1);

        // 4. Regras aprendidas e correções
        db.record_user_correction(&file_ids[0], None, &cat2.id).unwrap();
        db.upsert_learned_rule("filename_regex", "fatura", &cat2.id, 0.9, "learned").unwrap();
        let rules = db.get_classification_rules().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].pattern_value, "fatura");

        // 5. Move log e Undo
        let move_id = db.record_move(&session_id, &file_ids[0], r"C:\origem\fatura1.pdf", r"C:\destino\fatura1.pdf").unwrap();
        let moves = db.get_session_moves(&session_id).unwrap();
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].undone, 0);

        db.mark_move_undone(&move_id).unwrap();
        let moves_after = db.get_session_moves(&session_id).unwrap();
        assert_eq!(moves_after.len(), 0);

        // 6. Settings
        db.set_setting("theme", "dark").unwrap();
        let theme_setting = db.get_setting("theme").unwrap();
        assert_eq!(theme_setting, Some("dark".to_string()));
    }
}

