use serde::Deserialize;
use std::path::Path;
use crate::engine::renamer::{generate_proposed_name, resolve_name_collisions, RenameConfig, RenameSuggestion};
use crate::commands::apply::ApplySummary;
use crate::AppState;

#[derive(Deserialize, Debug, Clone)]
pub struct RenameOperation {
    pub file_id: String,
    pub from_path: String,
    pub to_path: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FileRenameCandidate {
    #[serde(default)]
    pub file_id: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub filename: String,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default)]
    pub category_color: Option<String>,
    #[serde(default)]
    pub size_bytes: u64,
    #[serde(default)]
    pub modified_at: Option<String>,
    #[serde(default)]
    pub text_sample: Option<String>,
}

fn default_category() -> String {
    "Outros".to_string()
}

/// Gera sugestões semânticas de novos nomes com base na configuração e arquivos da sessão
#[tauri::command]
pub async fn suggest_semantic_names(
    files: Vec<FileRenameCandidate>,
    config: RenameConfig,
) -> Result<Vec<RenameSuggestion>, String> {
    let mut suggestions: Vec<RenameSuggestion> = Vec::with_capacity(files.len());

    for f in &files {
        let size = if f.size_bytes > 0 {
            f.size_bytes
        } else {
            std::fs::metadata(&f.path).map(|m| m.len()).unwrap_or(0)
        };

        let filename = if f.filename.is_empty() {
            Path::new(&f.path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Arquivo".to_string())
        } else {
            f.filename.clone()
        };

        let proposed = generate_proposed_name(
            &filename,
            &f.category,
            f.text_sample.as_deref(),
            f.modified_at.as_deref(),
            &config,
        );

        let parent_dir = Path::new(&f.path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let proposed_path = Path::new(&parent_dir)
            .join(&proposed)
            .to_string_lossy()
            .to_string();

        suggestions.push(RenameSuggestion {
            file_id: f.file_id.clone(),
            current_path: f.path.clone(),
            current_filename: filename,
            proposed_filename: proposed,
            proposed_path,
            category: f.category.clone(),
            category_color: f.category_color.clone(),
            size_bytes: size,
            is_modified_by_user: false,
            is_ignored: false,
            has_collision: false,
        });
    }

    // Resolve colisões dentro da mesma pasta
    resolve_name_collisions(&mut suggestions);

    Ok(suggestions)
}

/// Aplica as operações de renomeação confirmadas pelo usuário, gravando no log transacional
#[tauri::command]
pub async fn apply_renames(
    session_id: String,
    renames: Vec<RenameOperation>,
    state: tauri::State<'_, AppState>,
) -> Result<ApplySummary, String> {
    let mut moved_count = 0;
    let mut failed_moves: Vec<String> = Vec::new();

    let db = state.db.lock().map_err(|e| e.to_string())?;

    for op in renames {
        let from = Path::new(&op.from_path);
        let to = Path::new(&op.to_path);

        if !from.exists() {
            failed_moves.push(format!("Arquivo de origem não encontrado: {}", op.from_path));
            continue;
        }

        // Se o nome não mudou, pula
        if op.from_path == op.to_path {
            continue;
        }

        // 1. Grava no move_log ANTES da operação para garantir integridade e reversão
        let log_id = match db.record_move(&session_id, &op.file_id, &op.from_path, &op.to_path) {
            Ok(id) => id,
            Err(e) => {
                failed_moves.push(format!("Falha ao registrar log de auditoria para {}: {}", op.from_path, e));
                continue;
            }
        };

        // 2. Executa a renomeação segura
        match crate::fs_ops::mover::safe_move(from, to) {
            Ok(_) => {
                moved_count += 1;
            }
            Err(e) => {
                let _ = db.mark_move_undone(&log_id);
                failed_moves.push(format!("Erro ao renomear {}: {}", op.from_path, e));
            }
        }
    }

    Ok(ApplySummary {
        moved: moved_count,
        failed: failed_moves,
    })
}
