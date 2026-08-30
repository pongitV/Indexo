use crate::db::models::{DuplicateGroup, DuplicateResolveAction};
use crate::engine::duplicates::find_duplicates_in_folder;
use std::path::Path;

#[tauri::command]
pub async fn scan_folder_duplicates(folder_path: String) -> Result<Vec<DuplicateGroup>, String> {
    tokio::task::spawn_blocking(move || {
        find_duplicates_in_folder(&folder_path).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("Falha na tarefa de duplicatas: {}", e))?
}

#[tauri::command]
pub async fn resolve_duplicates_actions(actions: Vec<DuplicateResolveAction>) -> Result<usize, String> {
    tokio::task::spawn_blocking(move || {
        let mut resolved_count = 0;

        for action in actions {
            for del_path_str in action.delete_or_move_paths {
                let p = Path::new(&del_path_str);
                if !p.exists() {
                    continue;
                }

                match action.action_type.as_str() {
                    "trash" => {
                        if let Err(e) = trash::delete(p) {
                            eprintln!("Erro ao mover para a lixeira {}: {}", del_path_str, e);
                        } else {
                            resolved_count += 1;
                        }
                    }
                    "delete" => {
                        if let Err(e) = std::fs::remove_file(p) {
                            eprintln!("Erro ao excluir arquivo {}: {}", del_path_str, e);
                        } else {
                            resolved_count += 1;
                        }
                    }
                    "archive_folder" => {
                        if let Some(ref archive_dir) = action.archive_folder_path {
                            let dest_dir = Path::new(archive_dir);
                            let _ = std::fs::create_dir_all(dest_dir);
                            if let Some(fname) = p.file_name() {
                                let dest_file = dest_dir.join(fname);
                                if let Err(e) = std::fs::rename(p, dest_file) {
                                    eprintln!("Erro ao arquivar {}: {}", del_path_str, e);
                                } else {
                                    resolved_count += 1;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(resolved_count)
    })
    .await
    .map_err(|e| format!("Falha ao resolver duplicatas: {}", e))?
}
