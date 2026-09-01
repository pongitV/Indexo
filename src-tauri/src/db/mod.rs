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
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA mmap_size = 268435456;",
        )?;
        conn.execute_batch(include_str!("schema.sql"))?;
        let _ = conn.execute("ALTER TABLE custom_rules ADD COLUMN subfolder_behavior TEXT DEFAULT 'auto'", []);
        let _ = conn.execute("ALTER TABLE custom_rules ADD COLUMN original_config TEXT", []);
        let _ = conn.execute("ALTER TABLE custom_rules ADD COLUMN version INTEGER DEFAULT 1", []);
        Ok(Self { conn })
    }

    #[cfg(test)]
    pub fn open_in_memory_for_tests() -> anyhow::Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA mmap_size = 268435456;",
        )?;
        conn.execute_batch(include_str!("schema.sql"))?;
        let _ = conn.execute("ALTER TABLE custom_rules ADD COLUMN subfolder_behavior TEXT DEFAULT 'auto'", []);
        let _ = conn.execute("ALTER TABLE custom_rules ADD COLUMN original_config TEXT", []);
        let _ = conn.execute("ALTER TABLE custom_rules ADD COLUMN version INTEGER DEFAULT 1", []);
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

    /// Busca o caminho raiz escaneado de uma sessão
    pub fn get_session_root_path(&self, session_id: &str) -> anyhow::Result<Option<String>> {
        let root = self.conn.query_row(
            "SELECT root_path FROM scan_sessions WHERE id = ?1",
            params![session_id],
            |row| row.get(0),
        ).ok();
        Ok(root)
    }

    // ==========================================
    // REGRAS PERSONALIZADAS (CUSTOM RULES)
    // ==========================================

    pub fn create_custom_rule(&self, input: models::CreateCustomRuleInput) -> anyhow::Result<models::CustomRule> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let priority = input.priority.unwrap_or(10);
        let subfolder = input.subfolder_behavior.unwrap_or_else(|| "auto".to_string());

        let initial_rule = models::CustomRule {
            id: id.clone(),
            name: input.name.clone(),
            condition_field: input.condition_field.clone(),
            condition_operator: input.condition_operator.clone(),
            condition_value: input.condition_value.clone(),
            action_type: input.action_type.clone(),
            action_value: input.action_value.clone(),
            subfolder_behavior: subfolder.clone(),
            original_config: None,
            version: 1,
            is_enabled: true,
            priority,
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        let orig_json = serde_json::to_string(&initial_rule).ok();

        self.conn.execute(
            "INSERT INTO custom_rules (id, name, condition_field, condition_operator, condition_value, action_type, action_value, subfolder_behavior, original_config, version, is_enabled, priority, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, 1, ?10, ?11, ?12)",
            params![id, input.name, input.condition_field, input.condition_operator, input.condition_value, input.action_type, input.action_value, subfolder, orig_json, priority, now, now],
        )?;

        let mut res = initial_rule;
        res.original_config = orig_json;
        Ok(res)
    }

    pub fn list_custom_rules(&self) -> anyhow::Result<Vec<models::CustomRule>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, condition_field, condition_operator, condition_value, action_type, action_value,
                    COALESCE(subfolder_behavior, 'auto'), original_config, COALESCE(version, 1), is_enabled, priority, created_at, updated_at
             FROM custom_rules
             ORDER BY priority DESC, created_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            let is_enabled_int: i64 = row.get(10)?;
            Ok(models::CustomRule {
                id: row.get(0)?,
                name: row.get(1)?,
                condition_field: row.get(2)?,
                condition_operator: row.get(3)?,
                condition_value: row.get(4)?,
                action_type: row.get(5)?,
                action_value: row.get(6)?,
                subfolder_behavior: row.get(7)?,
                original_config: row.get(8)?,
                version: row.get(9)?,
                is_enabled: is_enabled_int != 0,
                priority: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn update_custom_rule(&self, rule: models::CustomRule) -> anyhow::Result<()> {
        let now = chrono::Utc::now().to_rfc3339();

        // 1. Snapshot da versão anterior antes de atualizar
        if let Ok(prev) = self.conn.query_row(
            "SELECT id, name, condition_field, condition_operator, condition_value, action_type, action_value, COALESCE(subfolder_behavior, 'auto'), priority, COALESCE(version, 1)
             FROM custom_rules WHERE id = ?1",
            params![rule.id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                    row.get::<_, i64>(8)?,
                    row.get::<_, i64>(9)?,
                ))
            },
        ) {
            let hist_id = uuid::Uuid::new_v4().to_string();
            let _ = self.conn.execute(
                "INSERT INTO custom_rule_history (id, rule_id, name, condition_field, condition_operator, condition_value, action_type, action_value, subfolder_behavior, priority, version, saved_at, note)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![hist_id, prev.0, prev.1, prev.2, prev.3, prev.4, prev.5, prev.6, prev.7, prev.8, prev.9, now, "Edição do usuário"],
            );
        }

        let next_version = rule.version + 1;
        self.conn.execute(
            "UPDATE custom_rules SET
                name = ?1, condition_field = ?2, condition_operator = ?3, condition_value = ?4,
                action_type = ?5, action_value = ?6, subfolder_behavior = ?7, version = ?8,
                is_enabled = ?9, priority = ?10, updated_at = ?11
             WHERE id = ?12",
            params![
                rule.name, rule.condition_field, rule.condition_operator, rule.condition_value,
                rule.action_type, rule.action_value, rule.subfolder_behavior, next_version,
                if rule.is_enabled { 1 } else { 0 }, rule.priority, now, rule.id
            ],
        )?;
        Ok(())
    }

    pub fn restore_custom_rule_original(&self, rule_id: &str) -> anyhow::Result<models::CustomRule> {
        let now = chrono::Utc::now().to_rfc3339();

        let orig_json: Option<String> = self.conn.query_row(
            "SELECT original_config FROM custom_rules WHERE id = ?1",
            params![rule_id],
            |row| row.get(0),
        )?;

        if let Some(json_str) = orig_json {
            if let Ok(orig_rule) = serde_json::from_str::<models::CustomRule>(&json_str) {
                // Registra snapshot antes de restaurar
                let hist_id = uuid::Uuid::new_v4().to_string();
                let _ = self.conn.execute(
                    "INSERT INTO custom_rule_history (id, rule_id, name, condition_field, condition_operator, condition_value, action_type, action_value, subfolder_behavior, priority, version, saved_at, note)
                     SELECT ?1, id, name, condition_field, condition_operator, condition_value, action_type, action_value, subfolder_behavior, priority, version, ?2, 'Antes de restaurar original'
                     FROM custom_rules WHERE id = ?3",
                    params![hist_id, now, rule_id],
                );

                let current_version: i64 = self.conn.query_row(
                    "SELECT COALESCE(version, 1) FROM custom_rules WHERE id = ?1",
                    params![rule_id],
                    |row| row.get(0),
                ).unwrap_or(1);
                let next_version = current_version + 1;

                self.conn.execute(
                    "UPDATE custom_rules SET
                        name = ?1, condition_field = ?2, condition_operator = ?3, condition_value = ?4,
                        action_type = ?5, action_value = ?6, subfolder_behavior = ?7, version = ?8,
                        priority = ?9, updated_at = ?10
                     WHERE id = ?11",
                    params![
                        orig_rule.name, orig_rule.condition_field, orig_rule.condition_operator, orig_rule.condition_value,
                        orig_rule.action_type, orig_rule.action_value, orig_rule.subfolder_behavior, next_version,
                        orig_rule.priority, now, rule_id
                    ],
                )?;

                return Ok(models::CustomRule {
                    id: rule_id.to_string(),
                    name: orig_rule.name,
                    condition_field: orig_rule.condition_field,
                    condition_operator: orig_rule.condition_operator,
                    condition_value: orig_rule.condition_value,
                    action_type: orig_rule.action_type,
                    action_value: orig_rule.action_value,
                    subfolder_behavior: orig_rule.subfolder_behavior,
                    original_config: Some(json_str),
                    version: next_version,
                    is_enabled: true,
                    priority: orig_rule.priority,
                    created_at: orig_rule.created_at,
                    updated_at: now,
                });
            }
        }

        anyhow::bail!("Configuração original não encontrada para esta regra.")
    }

    pub fn get_custom_rule_history(&self, rule_id: &str) -> anyhow::Result<Vec<models::CustomRuleHistoryRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, rule_id, name, condition_field, condition_operator, condition_value, action_type, action_value,
                    COALESCE(subfolder_behavior, 'auto'), priority, version, saved_at, note
             FROM custom_rule_history
             WHERE rule_id = ?1
             ORDER BY version DESC, saved_at DESC",
        )?;

        let rows = stmt.query_map(params![rule_id], |row| {
            Ok(models::CustomRuleHistoryRecord {
                id: row.get(0)?,
                rule_id: row.get(1)?,
                name: row.get(2)?,
                condition_field: row.get(3)?,
                condition_operator: row.get(4)?,
                condition_value: row.get(5)?,
                action_type: row.get(6)?,
                action_value: row.get(7)?,
                subfolder_behavior: row.get(8)?,
                priority: row.get(9)?,
                version: row.get(10)?,
                saved_at: row.get(11)?,
                note: row.get(12)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn clear_all_user_data(&self) -> anyhow::Result<()> {
        self.conn.execute("DELETE FROM move_log", [])?;
        self.conn.execute("DELETE FROM file_categories", [])?;
        self.conn.execute("DELETE FROM user_corrections", [])?;
        self.conn.execute("DELETE FROM category_history", [])?;
        self.conn.execute("DELETE FROM custom_rule_history", [])?;
        self.conn.execute("DELETE FROM custom_rules", [])?;
        self.conn.execute("DELETE FROM classification_rules", [])?;
        self.conn.execute("DELETE FROM files", [])?;
        self.conn.execute("DELETE FROM scan_sessions", [])?;
        self.conn.execute("DELETE FROM embeddings_cache", [])?;
        self.conn.execute("DELETE FROM categories WHERE created_by = 'user'", [])?;
        let _ = self.reset_all_builtin_categories_config();
        Ok(())
    }

    // ==========================================
    // REGRAS E HEURÍSTICAS PADRÃO (BUILT-IN CONFIG)
    // ==========================================

    pub fn get_default_builtin_categories() -> Vec<models::BuiltinCategoryConfig> {
        vec![
            models::BuiltinCategoryConfig {
                id: "media-images".to_string(),
                group_name: "Media".to_string(),
                display_name: "Imagens e Fotografias".to_string(),
                target_path: "Media/Imagens-Fotografias".to_string(),
                description: "Fotografias, capturas de tela, wallpapers, artes vetoriais e gráficos".to_string(),
                extensions: vec!["jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "tiff", "ico", "raw", "heic", "psd", "ai", "eps", "xcf", "cr2", "nef", "arw"].into_iter().map(String::from).collect(),
                keywords: vec!["screenshot", "wallpaper", "foto", "foto_", "img_", "captura", "arte", "banner"].into_iter().map(String::from).collect(),
                subfolders: vec!["Fotografias", "Wallpapers", "Capturas-Tela", "Design-Vetores"].into_iter().map(String::from).collect(),
                subfolder_behavior: "auto".to_string(),
                is_enabled: true,
                is_customized: false,
            },
            models::BuiltinCategoryConfig {
                id: "media-videos".to_string(),
                group_name: "Media".to_string(),
                display_name: "Vídeos e Gravações".to_string(),
                target_path: "Media/Videos-Gravacoes".to_string(),
                description: "Filmes, gravações de tela, clipes de jogos e tutoriais".to_string(),
                extensions: vec!["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "3gp", "ts", "mpg", "mpeg"].into_iter().map(String::from).collect(),
                keywords: vec!["video", "gravacao", "clip", "replay", "screen", "gameplay", "filme"].into_iter().map(String::from).collect(),
                subfolders: vec!["Gravacoes-Tela", "Clipes-Jogos", "Filmes-Series", "Tutoriais"].into_iter().map(String::from).collect(),
                subfolder_behavior: "auto".to_string(),
                is_enabled: true,
                is_customized: false,
            },
            models::BuiltinCategoryConfig {
                id: "media-audio".to_string(),
                group_name: "Media".to_string(),
                display_name: "Áudios e Músicas".to_string(),
                target_path: "Media/Audios-Musicas".to_string(),
                description: "Músicas, podcasts, efeitos sonoros (SFX) e gravações de voz".to_string(),
                extensions: vec!["mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "mid", "midi", "opus", "aiff", "alac"].into_iter().map(String::from).collect(),
                keywords: vec!["musica", "audio", "podcast", "sfx", "sound", "track", "faixa", "voz"].into_iter().map(String::from).collect(),
                subfolders: vec!["Albuns-Artistas", "Podcasts", "Efeitos-Sonoros-SFX", "Gravacoes-Voz"].into_iter().map(String::from).collect(),
                subfolder_behavior: "auto".to_string(),
                is_enabled: true,
                is_customized: false,
            },
            models::BuiltinCategoryConfig {
                id: "exec-roms".to_string(),
                group_name: "Executaveis".to_string(),
                display_name: "Jogos, Emuladores e ROMs".to_string(),
                target_path: "Executaveis/Jogos-Emuladores-ROMs".to_string(),
                description: "ROMs e imagens de console separadas por plataforma".to_string(),
                extensions: vec!["nes", "sfc", "smc", "gba", "gbc", "gb", "nds", "3ds", "cia", "n64", "z64", "v64", "nsp", "xci", "rvz", "wbfs", "wad", "gcz", "pbp", "cso", "gen", "smd", "cdi", "gdi", "chd"].into_iter().map(String::from).collect(),
                keywords: vec!["rom", "emulator", "emulador", "save", "bios", "patch"].into_iter().map(String::from).collect(),
                subfolders: vec!["Nintendo-NES", "Super-Nintendo-SNES", "Game-Boy-Advance-GBA", "Game-Boy-Color-GBC", "Nintendo-DS", "Nintendo-3DS", "Nintendo-64", "Nintendo-Switch", "Nintendo-Wii-GameCube", "PlayStation-PSP", "PlayStation-1", "PlayStation-2", "PlayStation-3", "Sega-MegaDrive", "Sega-Dreamcast", "Sega-Saturn"].into_iter().map(String::from).collect(),
                subfolder_behavior: "auto".to_string(),
                is_enabled: true,
                is_customized: false,
            },
            models::BuiltinCategoryConfig {
                id: "exec-apps".to_string(),
                group_name: "Executaveis".to_string(),
                display_name: "Aplicativos e Softwares".to_string(),
                target_path: "Executaveis/Aplicativos-Utilitarios".to_string(),
                description: "Aplicativos instalados, navegadores, IDEs e utilitários".to_string(),
                extensions: vec!["exe", "msi", "appimage", "dmg", "pkg", "deb", "rpm", "apk"].into_iter().map(String::from).collect(),
                keywords: vec!["chrome", "firefox", "edge", "vscode", "idea", "photoshop", "figma", "driver"].into_iter().map(String::from).collect(),
                subfolders: vec!["Aplicativos-Navegadores", "Aplicativos-IDEs", "Aplicativos-Design-Edicao", "Aplicativos-Utilitarios"].into_iter().map(String::from).collect(),
                subfolder_behavior: "auto".to_string(),
                is_enabled: true,
                is_customized: false,
            },
            models::BuiltinCategoryConfig {
                id: "exec-installers".to_string(),
                group_name: "Executaveis".to_string(),
                display_name: "Instaladores e Imagens de Disco".to_string(),
                target_path: "Executaveis/Instaladores-Setups".to_string(),
                description: "Setups de instalação, drivers de hardware e arquivos ISO".to_string(),
                extensions: vec!["exe", "msi", "iso", "img", "vhd", "vhdx", "bin", "cue"].into_iter().map(String::from).collect(),
                keywords: vec!["setup", "installer", "instalador", "driver", "geforce", "realtek"].into_iter().map(String::from).collect(),
                subfolders: vec!["Instaladores-Setups", "Instaladores-Drivers", "Instaladores-ISOs"].into_iter().map(String::from).collect(),
                subfolder_behavior: "auto".to_string(),
                is_enabled: true,
                is_customized: false,
            },
            models::BuiltinCategoryConfig {
                id: "docs-finance".to_string(),
                group_name: "Documentos".to_string(),
                display_name: "Documentos Fiscais e Pessoais".to_string(),
                target_path: "Documentos/Fiscais-Pessoais".to_string(),
                description: "Boletos, faturas, comprovantes, recibos, DANFE, notas fiscais e contratos".to_string(),
                extensions: vec!["pdf", "xml", "docx", "doc", "xlsx", "xls", "txt"].into_iter().map(String::from).collect(),
                keywords: vec!["boleto", "fatura", "recibo", "comprovante", "danfe", "nota_fiscal", "contrato", "declaracao", "imposto"].into_iter().map(String::from).collect(),
                subfolders: vec!["Boletos-Faturas", "Comprovantes-Pagamento", "Notas-Fiscais-DANFE", "Contratos-Declaracoes"].into_iter().map(String::from).collect(),
                subfolder_behavior: "by_year".to_string(),
                is_enabled: true,
                is_customized: false,
            },
            models::BuiltinCategoryConfig {
                id: "docs-work".to_string(),
                group_name: "Documentos".to_string(),
                display_name: "Relatórios e Trabalho".to_string(),
                target_path: "Documentos/Trabalho".to_string(),
                description: "Apresentações, planilhas corporativas, relatórios de reuniões e planejamentos".to_string(),
                extensions: vec!["pdf", "docx", "doc", "xlsx", "xls", "pptx", "ppt", "csv", "txt", "md"].into_iter().map(String::from).collect(),
                keywords: vec!["relatorio", "report", "reuniao", "projeto", "apresentacao", "slides", "cronograma", "orcamento", "proposta"].into_iter().map(String::from).collect(),
                subfolders: vec!["Relatorios", "Planilhas-Orcamentos", "Apresentacoes", "Reunioes-Atas"].into_iter().map(String::from).collect(),
                subfolder_behavior: "by_year".to_string(),
                is_enabled: true,
                is_customized: false,
            },
            models::BuiltinCategoryConfig {
                id: "docs-study".to_string(),
                group_name: "Documentos".to_string(),
                display_name: "Estudos e Acadêmico".to_string(),
                target_path: "Documentos/Estudos".to_string(),
                description: "Livros digitais, apostilas, artigos acadêmicos, TCC, teses e resumos".to_string(),
                extensions: vec!["pdf", "epub", "mobi", "azw3", "cbr", "cbz", "docx", "txt", "md"].into_iter().map(String::from).collect(),
                keywords: vec!["aula", "exercicio", "tcc", "artigo", "tese", "monografia", "livro", "apostila", "resumo", "curso"].into_iter().map(String::from).collect(),
                subfolders: vec!["Livros-Ebooks", "Artigos-Teses", "Aulas-Apostilas", "Cursos-Certificados"].into_iter().map(String::from).collect(),
                subfolder_behavior: "auto".to_string(),
                is_enabled: true,
                is_customized: false,
            },
            models::BuiltinCategoryConfig {
                id: "proj-repos".to_string(),
                group_name: "Projetos".to_string(),
                display_name: "Repositórios e Código-Fonte".to_string(),
                target_path: "Projetos/Repositorios-Locais".to_string(),
                description: "Repositórios de código, scripts de automação e arquivos de desenvolvimento".to_string(),
                extensions: vec!["rs", "js", "ts", "py", "html", "css", "cpp", "c", "h", "java", "go", "php", "sh", "bat", "ps1", "sql", "json", "yaml", "toml"].into_iter().map(String::from).collect(),
                keywords: vec!["github", "repo", "script", "automacao", "api", "backend", "frontend"].into_iter().map(String::from).collect(),
                subfolders: vec!["Repositorios-GitHub", "Repositorios-Locais", "Scripts-Automacoes"].into_iter().map(String::from).collect(),
                subfolder_behavior: "auto".to_string(),
                is_enabled: true,
                is_customized: false,
            },
            models::BuiltinCategoryConfig {
                id: "proj-3d".to_string(),
                group_name: "Projetos".to_string(),
                display_name: "Modelos 3D e CAD".to_string(),
                target_path: "Projetos/Modelos-3D-CAD".to_string(),
                description: "Cenas do Blender, modelos para impressão 3D e projetos de engenharia".to_string(),
                extensions: vec!["blend", "stl", "step", "obj", "fbx", "dae", "3ds", "iges", "dwg", "dxf", "blend1", "skp", "ply"].into_iter().map(String::from).collect(),
                keywords: vec!["model", "render", "print3d", "cad", "blender", "peca", "malha"].into_iter().map(String::from).collect(),
                subfolders: vec!["Projetos-Blender", "Impressao-3D", "Projetos-CAD-DWG"].into_iter().map(String::from).collect(),
                subfolder_behavior: "auto".to_string(),
                is_enabled: true,
                is_customized: false,
            },
            models::BuiltinCategoryConfig {
                id: "archives-backups".to_string(),
                group_name: "Compactados-Backups".to_string(),
                display_name: "Compactados e Backups".to_string(),
                target_path: "Compactados-Backups".to_string(),
                description: "Pacotes ZIP, RAR, 7Z e cópias de segurança".to_string(),
                extensions: vec!["zip", "rar", "7z", "tar", "gz", "bz2", "xz", "tgz", "cab", "lz", "zst", "bak"].into_iter().map(String::from).collect(),
                keywords: vec!["backup", "archive", "pack", "dist", "bkp"].into_iter().map(String::from).collect(),
                subfolders: vec!["Backups", "Arquivos-ZIP", "Arquivos-RAR", "Arquivos-7Z"].into_iter().map(String::from).collect(),
                subfolder_behavior: "auto".to_string(),
                is_enabled: true,
                is_customized: false,
            },
            models::BuiltinCategoryConfig {
                id: "fonts-typography".to_string(),
                group_name: "Fontes-Tipografia".to_string(),
                display_name: "Fontes e Tipografia".to_string(),
                target_path: "Fontes-Tipografia".to_string(),
                description: "Fontes TrueType, OpenType, WebFonts e ícones tipográficos".to_string(),
                extensions: vec!["ttf", "otf", "woff", "woff2", "eot", "fon"].into_iter().map(String::from).collect(),
                keywords: vec!["font", "type", "sans", "serif", "mono", "regular", "bold"].into_iter().map(String::from).collect(),
                subfolders: vec!["Fontes-Principais", "Icones-Fontes", "Web-Fonts"].into_iter().map(String::from).collect(),
                subfolder_behavior: "auto".to_string(),
                is_enabled: true,
                is_customized: false,
            },
        ]
    }

    pub fn get_active_builtin_categories(&self) -> Vec<models::BuiltinCategoryConfig> {
        if let Ok(Some(json_str)) = self.get_setting("builtin_categories_config") {
            if let Ok(saved) = serde_json::from_str::<Vec<models::BuiltinCategoryConfig>>(&json_str) {
                return saved;
            }
        }
        Self::get_default_builtin_categories()
    }

    pub fn save_builtin_category_config(&self, mut config: models::BuiltinCategoryConfig) -> anyhow::Result<()> {
        let mut list = self.get_active_builtin_categories();
        config.is_customized = true;

        if let Some(pos) = list.iter().position(|c| c.id == config.id) {
            list[pos] = config;
        } else {
            list.push(config);
        }

        let json_str = serde_json::to_string(&list)?;
        self.set_setting("builtin_categories_config", &json_str)?;
        Ok(())
    }

    pub fn reset_builtin_category_config(&self, id: &str) -> anyhow::Result<models::BuiltinCategoryConfig> {
        let defaults = Self::get_default_builtin_categories();
        let default_item = defaults.into_iter().find(|c| c.id == id)
            .ok_or_else(|| anyhow::anyhow!("Categoria padrão não encontrada"))?;

        let mut list = self.get_active_builtin_categories();
        if let Some(pos) = list.iter().position(|c| c.id == id) {
            list[pos] = default_item.clone();
        }

        let json_str = serde_json::to_string(&list)?;
        self.set_setting("builtin_categories_config", &json_str)?;
        Ok(default_item)
    }

    pub fn reset_all_builtin_categories_config(&self) -> anyhow::Result<Vec<models::BuiltinCategoryConfig>> {
        let defaults = Self::get_default_builtin_categories();
        let json_str = serde_json::to_string(&defaults)?;
        self.set_setting("builtin_categories_config", &json_str)?;
        Ok(defaults)
    }

    pub fn delete_custom_rule(&self, id: &str) -> anyhow::Result<()> {
        self.conn.execute("DELETE FROM custom_rules WHERE id = ?1", params![id])?;
        self.conn.execute("DELETE FROM custom_rule_history WHERE rule_id = ?1", params![id])?;
        Ok(())
    }

    pub fn toggle_custom_rule(&self, id: &str, is_enabled: bool) -> anyhow::Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE custom_rules SET is_enabled = ?1, updated_at = ?2 WHERE id = ?3",
            params![if is_enabled { 1 } else { 0 }, now, id],
        )?;
        Ok(())
    }

    pub fn list_all_learned_rules(&self) -> anyhow::Result<Vec<models::LearnedRuleInfo>> {
        let mut stmt = self.conn.prepare(
            "SELECT r.id, r.pattern_type, r.pattern_value, r.category_id, c.name, c.color,
                    r.confidence_weight, r.created_from, r.hit_count, r.created_at, r.updated_at
             FROM classification_rules r
             LEFT JOIN categories c ON c.id = r.category_id
             ORDER BY r.hit_count DESC, r.updated_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(models::LearnedRuleInfo {
                id: row.get(0)?,
                pattern_type: row.get(1)?,
                pattern_value: row.get(2)?,
                category_id: row.get(3)?,
                category_name: row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "Geral".to_string()),
                category_color: row.get(5)?,
                confidence_weight: row.get(6)?,
                created_from: row.get(7)?,
                hit_count: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn delete_learned_rule(&self, id: &str) -> anyhow::Result<()> {
        self.conn.execute("DELETE FROM classification_rules WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_storage_analytics(&self) -> anyhow::Result<models::StorageAnalytics> {
        let history = self.get_organization_history()?;
        let total_sessions_count = history.len() as i64;
        let mut total_organized_files: i64 = 0;
        let mut total_organized_bytes: i64 = 0;
        let mut cat_bytes_map: std::collections::HashMap<String, (String, Option<String>, i64, i64)> = std::collections::HashMap::new();
        let mut dates_set = std::collections::BTreeSet::new();

        for s in &history {
            dates_set.insert(s.started_at.chars().take(10).collect::<String>());
            for f in &s.files {
                total_organized_files += 1;
                total_organized_bytes += f.size_bytes;

                let cat_name = if f.category_name.is_empty() { "Outros".to_string() } else { f.category_name.clone() };
                let entry = cat_bytes_map.entry(cat_name.clone()).or_insert((cat_name, f.category_color.clone(), 0, 0));
                entry.2 += 1;
                entry.3 += f.size_bytes;
            }
        }

        let mut categories_breakdown = Vec::new();
        for (_, (cat_name, color, count, bytes)) in cat_bytes_map {
            let percentage = if total_organized_bytes > 0 {
                (bytes as f64 / total_organized_bytes as f64) * 100.0
            } else {
                0.0
            };
            categories_breakdown.push(models::StorageCategoryStat {
                category_name: cat_name,
                category_color: color,
                total_files: count,
                total_bytes: bytes,
                percentage,
            });
        }

        categories_breakdown.sort_by(|a, b| b.total_bytes.cmp(&a.total_bytes));
        let recent_activity_dates = dates_set.into_iter().rev().take(14).collect();

        Ok(models::StorageAnalytics {
            total_organized_files,
            total_organized_bytes,
            total_sessions_count,
            categories_breakdown,
            recent_activity_dates,
        })
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

    /// Renomeia uma categoria e registra o histórico da alteração
    pub fn rename_category(&self, id: &str, new_name: &str) -> anyhow::Result<()> {
        self.rename_category_with_details(id, new_name, "user", None)
    }

    /// Renomeia categoria com detalhes de autoria e motivo (user | ai_refinement | merge | auto)
    pub fn rename_category_with_details(
        &self,
        id: &str,
        new_name: &str,
        changed_by: &str,
        reason: Option<&str>,
    ) -> anyhow::Result<()> {
        let trimmed = new_name.trim();
        if trimmed.is_empty() {
            anyhow::bail!("O nome da categoria não pode ser vazio.");
        }

        let old_name: Option<String> = self.conn.query_row(
            "SELECT name FROM categories WHERE id = ?1",
            params![id],
            |row| row.get(0),
        ).ok();

        if let Some(old) = old_name {
            if old != trimmed {
                let slug = slugify(trimmed);
                self.conn.execute(
                    "UPDATE categories SET name = ?1, slug = ?2 WHERE id = ?3",
                    params![trimmed, slug, id],
                )?;

                self.record_category_change(id, &old, trimmed, changed_by, reason)?;
            }
        }
        Ok(())
    }

    /// Registra uma entrada no histórico de mudanças de uma categoria/tag
    pub fn record_category_change(
        &self,
        category_id: &str,
        old_name: &str,
        new_name: &str,
        changed_by: &str,
        reason: Option<&str>,
    ) -> anyhow::Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO category_history (id, category_id, old_name, new_name, changed_by, reason, changed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, category_id, old_name, new_name, changed_by, reason, now],
        )?;
        Ok(())
    }

    /// Busca a timeline do histórico de mudanças de uma categoria/tag
    pub fn get_category_history(&self, category_id: &str) -> anyhow::Result<Vec<models::CategoryHistoryRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, category_id, old_name, new_name, changed_by, reason, changed_at
             FROM category_history
             WHERE category_id = ?1
             ORDER BY changed_at DESC",
        )?;

        let rows = stmt.query_map(params![category_id], |row| {
            Ok(models::CategoryHistoryRecord {
                id: row.get(0)?,
                category_id: row.get(1)?,
                old_name: row.get(2)?,
                new_name: row.get(3)?,
                changed_by: row.get(4)?,
                reason: row.get(5)?,
                changed_at: row.get(6)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
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

    /// Limpa categorias automáticas que não possuem nenhum arquivo associado
    pub fn clean_unused_auto_categories(&self) -> anyhow::Result<usize> {
        let tx = self.conn.unchecked_transaction()?;
        let deleted = tx.execute(
            "DELETE FROM categories
             WHERE created_by = 'auto'
               AND id NOT IN (SELECT DISTINCT category_id FROM file_categories)",
            [],
        )?;
        tx.commit()?;
        Ok(deleted)
    }

    /// Expurga todas as categorias automáticas de teste, preservando categorias e regras criadas manualmente pelo usuário
    pub fn purge_all_auto_categories(&self) -> anyhow::Result<usize> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM file_categories
             WHERE category_id IN (SELECT id FROM categories WHERE created_by = 'auto')",
            [],
        )?;
        tx.execute(
            "DELETE FROM classification_rules
             WHERE created_from = 'learned'
               AND category_id IN (SELECT id FROM categories WHERE created_by = 'auto')",
            [],
        )?;
        let deleted = tx.execute(
            "DELETE FROM categories WHERE created_by = 'auto'",
            [],
        )?;
        tx.commit()?;
        let _ = self.conn.execute("VACUUM", []);
        Ok(deleted)
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

    /// Busca o histórico de todas as sessões de organização com detalhes de árvore, categorias, tags, arquivos e renomeações
    pub fn get_organization_history(&self) -> anyhow::Result<Vec<models::OrganizationSessionSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.root_path, s.started_at, s.finished_at, s.status, s.files_scanned
             FROM scan_sessions s
             ORDER BY s.started_at DESC",
        )?;

        let session_rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })?;

        let default_rename_config = crate::engine::renamer::RenameConfig {
            preset: "semantic".to_string(),
            separator: "_".to_string(),
            case_style: "title".to_string(),
            date_format: "none".to_string(),
            include_category: true,
            remove_noise: true,
            custom_template: None,
            structure_order: None,
        };

        let mut summaries = Vec::new();

        for s in session_rows {
            let (session_id, root_path, started_at, finished_at, status, files_scanned) = s?;

            // 1. Buscar todos os moves da sessão
            let mut move_stmt = self.conn.prepare(
                "SELECT id, session_id, file_id, from_path, to_path, moved_at, undone
                 FROM move_log WHERE session_id = ?1 ORDER BY moved_at DESC",
            )?;

            let move_rows = move_stmt.query_map(params![session_id], |row| {
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

            let mut moves = Vec::new();
            let mut undone_count = 0;
            for m in move_rows {
                let rec = m?;
                if rec.undone != 0 {
                    undone_count += 1;
                }
                moves.push(rec);
            }
            let files_moved_count = moves.len();

            // 2. Buscar todos os arquivos desta sessão com suas categorias
            let mut file_stmt = self.conn.prepare(
                "SELECT f.id, f.filename, f.original_path, f.size_bytes,
                        COALESCE(c.id, ''), COALESCE(c.name, 'Outros Arquivos'), c.color, COALESCE(c.created_by, 'auto')
                 FROM files f
                 LEFT JOIN file_categories fc ON fc.file_id = f.id
                 LEFT JOIN categories c ON c.id = fc.category_id
                 WHERE f.session_id = ?1
                 ORDER BY f.filename ASC",
            )?;

            let mut files = Vec::new();
            let mut cat_map: std::collections::HashMap<String, models::SessionCategoryInfo> = std::collections::HashMap::new();

            let file_rows = file_stmt.query_map(params![session_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, String>(7)?,
                ))
            })?;

            for fr in file_rows {
                let (f_id, fname, orig_path, size, cat_id, cat_name, cat_color, created_by) = fr?;

                // Gerar sugestão semântica de preview
                let proposed = crate::engine::renamer::generate_proposed_name(
                    &fname,
                    &cat_name,
                    None,
                    None,
                    &default_rename_config,
                );

                let is_already_org = crate::engine::heuristics::detect_already_organized_folder(&orig_path, &root_path).is_some();

                // Agrupar categorias apenas para arquivos genuinamente categorizados
                if !is_already_org && !cat_id.is_empty() {
                    let entry = cat_map.entry(cat_name.clone()).or_insert_with(|| models::SessionCategoryInfo {
                        id: cat_id.clone(),
                        name: cat_name.clone(),
                        color: cat_color.clone(),
                        created_by: created_by.clone(),
                        file_count: 0,
                    });
                    entry.file_count += 1;
                }

                files.push(models::SessionFileInfo {
                    file_id: f_id,
                    filename: fname,
                    original_path: orig_path,
                    category_name: cat_name,
                    category_color: cat_color,
                    size_bytes: size,
                    is_already_organized: is_already_org,
                    proposed_name: Some(proposed),
                });
            }

            let mut categories_assigned: Vec<models::SessionCategoryInfo> = cat_map.into_values().collect();
            categories_assigned.sort_by(|a, b| b.file_count.cmp(&a.file_count));

            // 3. Mapear renames a partir dos arquivos e do move_log
            let mut move_map: std::collections::HashMap<String, &models::MoveLogRecord> = std::collections::HashMap::new();
            for m in &moves {
                move_map.insert(m.file_id.clone(), m);
            }

            let mut renames = Vec::new();
            for f in &files {
                let proposed_name = f.proposed_name.clone().unwrap_or_else(|| f.filename.clone());
                if let Some(m) = move_map.get(&f.file_id) {
                    let from_name = std::path::Path::new(&m.from_path)
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| f.filename.clone());
                    let to_name = std::path::Path::new(&m.to_path)
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| proposed_name.clone());

                    renames.push(models::SessionRenameInfo {
                        file_id: f.file_id.clone(),
                        original_name: from_name,
                        proposed_name: proposed_name.clone(),
                        final_name: Some(to_name),
                        from_path: m.from_path.clone(),
                        to_path: m.to_path.clone(),
                        applied: true,
                        undone: m.undone != 0,
                    });
                } else {
                    let parent_p = std::path::Path::new(&f.original_path)
                        .parent()
                        .map(|p| p.join(&proposed_name).to_string_lossy().to_string())
                        .unwrap_or_default();

                    renames.push(models::SessionRenameInfo {
                        file_id: f.file_id.clone(),
                        original_name: f.filename.clone(),
                        proposed_name: proposed_name.clone(),
                        final_name: None,
                        from_path: f.original_path.clone(),
                        to_path: parent_p,
                        applied: false,
                        undone: false,
                    });
                }
            }

            summaries.push(models::OrganizationSessionSummary {
                session_id,
                root_path,
                started_at,
                finished_at,
                status,
                files_scanned,
                files_moved_count,
                undone_count,
                categories_assigned,
                files,
                moves,
                renames,
            });
        }

        Ok(summaries)
    }

    /// Desfaz todos os moves de uma sessão específica
    pub fn undo_session_moves(&self, session_id: &str) -> anyhow::Result<usize> {
        let moves = self.get_session_moves(session_id)?;
        let mut undone_count = 0;

        for m in moves {
            let from_p = std::path::Path::new(&m.from_path);
            let to_p = std::path::Path::new(&m.to_path);

            if to_p.exists() {
                if crate::fs_ops::mover::safe_move(to_p, from_p).is_ok() {
                    let _ = self.mark_move_undone(&m.id);
                    undone_count += 1;
                }
            } else {
                let _ = self.mark_move_undone(&m.id);
                undone_count += 1;
            }
        }

        Ok(undone_count)
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

        // 7. Histórico de Mudanças da Categoria
        let hist_cat = db.get_or_create_category("Recibos Antigos", "user", None).unwrap();
        db.rename_category_with_details(&hist_cat.id, "Comprovantes 2026", "user", Some("Atualização manual")).unwrap();
        db.rename_category_with_details(&hist_cat.id, "Comprovantes Fiscais", "ai_refinement", Some("Refinamento semântico")).unwrap();
        let history = db.get_category_history(&hist_cat.id).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].new_name, "Comprovantes Fiscais");
        assert_eq!(history[0].old_name, "Comprovantes 2026");
        assert_eq!(history[0].changed_by, "ai_refinement");
        assert_eq!(history[1].new_name, "Comprovantes 2026");
        assert_eq!(history[1].old_name, "Recibos Antigos");

        // 8. Histórico Geral de Organizações
        let org_history = db.get_organization_history().unwrap();
        assert!(!org_history.is_empty());
        let session_record = org_history.iter().find(|s| s.session_id == session_id).unwrap();
        assert_eq!(session_record.files_moved_count, 1);
        assert!(session_record.categories_assigned.iter().any(|c| c.name == "Contratos"));
        assert_eq!(session_record.files.len(), 2);
        assert_eq!(session_record.renames.len(), 2);

        // 9. Limpeza e expurgo de categorias automáticas
        let auto_unused = db.get_or_create_category("Auto Tag Orfa", "auto", None).unwrap();
        assert!(db.list_categories().unwrap().iter().any(|c| c.id == auto_unused.id));
        let cleaned = db.clean_unused_auto_categories().unwrap();
        assert!(cleaned >= 1);
        assert!(!db.list_categories().unwrap().iter().any(|c| c.id == auto_unused.id));

        // Purge total preservando usuario
        let user_cat = db.get_or_create_category("Minha Tag Manual", "user", None).unwrap();
        let auto_cat = db.get_or_create_category("Tag Auto Temp", "auto", None).unwrap();
        db.assign_file_category(&file_ids[0], &auto_cat.id, 0.8, "heuristic").unwrap();

        let purged = db.purge_all_auto_categories().unwrap();
        assert!(purged >= 1);
        let remaining_cats = db.list_categories().unwrap();
        assert!(remaining_cats.iter().any(|c| c.id == user_cat.id));
        assert!(!remaining_cats.iter().any(|c| c.id == auto_cat.id));

        // 10. Regras Customizadas: Criação, Edição com Histórico, e Restauração Original
        let rule_input = models::CreateCustomRuleInput {
            name: "ROMs Game Boy Advance".to_string(),
            condition_field: "extension".to_string(),
            condition_operator: "equals".to_string(),
            condition_value: "gba".to_string(),
            action_type: "move_category".to_string(),
            action_value: "Executaveis/Jogos-Emuladores-ROMs/Game-Boy-Advance-GBA".to_string(),
            subfolder_behavior: Some("auto".to_string()),
            priority: Some(80),
        };
        let created_rule = db.create_custom_rule(rule_input).unwrap();
        assert_eq!(created_rule.version, 1);
        assert!(created_rule.original_config.is_some());

        // Edição da regra
        let mut updated_rule = created_rule.clone();
        updated_rule.name = "ROMs GBA Atualizada".to_string();
        updated_rule.subfolder_behavior = "by_year".to_string();
        db.update_custom_rule(updated_rule.clone()).unwrap();

        let list = db.list_custom_rules().unwrap();
        let current = list.iter().find(|r| r.id == created_rule.id).unwrap();
        assert_eq!(current.version, 2);
        assert_eq!(current.name, "ROMs GBA Atualizada");
        assert_eq!(current.subfolder_behavior, "by_year");

        // Histórico registrado
        let history = db.get_custom_rule_history(&created_rule.id).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].version, 1);
        assert_eq!(history[0].name, "ROMs Game Boy Advance");

        // Restauração da original
        let restored = db.restore_custom_rule_original(&created_rule.id).unwrap();
        assert_eq!(restored.name, "ROMs Game Boy Advance");
        assert_eq!(restored.subfolder_behavior, "auto");
        assert_eq!(restored.version, 3); // incrementa versão mantendo rastreabilidade

        // 11. Limpar Todos os Dados do Usuário
        db.clear_all_user_data().unwrap();
        assert_eq!(db.list_custom_rules().unwrap().len(), 0);
        assert_eq!(db.get_classification_rules().unwrap().len(), 0);
    }
}

