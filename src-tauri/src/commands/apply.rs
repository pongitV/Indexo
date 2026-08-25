use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct FileMove {
    pub file_id: String,
    pub from_path: String,
    pub to_path: String,
}

#[derive(Serialize, Debug)]
pub struct ApplySummary {
    pub moved: usize,
    pub failed: Vec<String>,
}

/// Executa os moves aprovados pelo usuario na tela de preview.
/// NUNCA deleta. Sempre grava em `move_log` ANTES de mover, para permitir undo
/// mesmo em caso de crash no meio do processo (ver secao 9 do plano).
#[tauri::command]
pub async fn apply_organization(
    session_id: String,
    moves: Vec<FileMove>,
    state: tauri::State<'_, crate::AppState>,
) -> Result<ApplySummary, String> {
    let mut moved_count = 0;
    let mut failed_moves: Vec<String> = Vec::new();

    let db = state.db.lock().map_err(|e| e.to_string())?;

    for mv in moves {
        let from = Path::new(&mv.from_path);
        let to = Path::new(&mv.to_path);

        if !from.exists() {
            failed_moves.push(format!("Arquivo de origem nao encontrado: {}", mv.from_path));
            continue;
        }

        // 1. Grava no move_log ANTES da operacao fisica de mover
        let log_id = match db.record_move(&session_id, &mv.file_id, &mv.from_path, &mv.to_path) {
            Ok(id) => id,
            Err(e) => {
                failed_moves.push(format!("Falha ao registrar log de auditoria para {}: {}", mv.from_path, e));
                continue;
            }
        };

        // 2. Executa a movimentacao segura com resolucao de colisoes
        match crate::fs_ops::mover::safe_move(from, to) {
            Ok(_) => {
                moved_count += 1;
            }
            Err(e) => {
                // Marca como undone no banco caso a operacao no disco tenha falhado
                let _ = db.mark_move_undone(&log_id);
                failed_moves.push(format!("Erro ao mover {}: {}", mv.from_path, e));
            }
        }
    }

    Ok(ApplySummary {
        moved: moved_count,
        failed: failed_moves,
    })
}

/// Reverte a ultima aplicacao usando o `move_log` da sessao.
#[tauri::command]
pub async fn undo_last_apply(
    session_id: Option<String>,
    state: tauri::State<'_, crate::AppState>,
) -> Result<usize, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let moves_to_undo = match session_id {
        Some(ref sid) if !sid.trim().is_empty() => db.get_session_moves(sid).map_err(|e| e.to_string())?,
        _ => db.get_last_session_moves().map_err(|e| e.to_string())?,
    };

    if moves_to_undo.is_empty() {
        return Ok(0);
    }

    let mut undone_count = 0;

    for rec in moves_to_undo {
        let from_orig = Path::new(&rec.from_path);
        let to_moved = Path::new(&rec.to_path);

        if to_moved.exists() {
            if let Ok(_) = crate::fs_ops::mover::undo_single_move(from_orig, to_moved) {
                let _ = db.mark_move_undone(&rec.id);
                undone_count += 1;
            }
        } else {
            // Se o arquivo ja nao esta mais no destino, apenas marca como undone no log
            let _ = db.mark_move_undone(&rec.id);
        }
    }

    Ok(undone_count)
}
