use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use zip::write::FileOptions;
use zip::ZipWriter;

use crate::db::models::UserCorrection;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub created_by: String, // "auto" | "user"
    pub file_count: i64,
}

#[tauri::command]
pub async fn list_categories(
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<Category>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let list = db.list_categories().map_err(|e| e.to_string())?;
    Ok(list
        .into_iter()
        .map(|c| Category {
            id: c.id,
            name: c.name,
            color: c.color,
            created_by: c.created_by,
            file_count: c.file_count,
        })
        .collect())
}

#[tauri::command]
pub async fn create_category(
    name: String,
    color: Option<String>,
    state: tauri::State<'_, crate::AppState>,
) -> Result<Category, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let cat = db
        .get_or_create_category(&name, "user", color.as_deref())
        .map_err(|e| e.to_string())?;
    Ok(Category {
        id: cat.id,
        name: cat.name,
        color: cat.color,
        created_by: cat.created_by,
        file_count: 0,
    })
}

#[tauri::command]
pub async fn rename_category(
    id: String,
    new_name: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.rename_category(&id, &new_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn merge_categories(
    source_id: String,
    target_id: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.merge_categories(&source_id, &target_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_category(
    id: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.delete_category(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clean_unused_categories(
    state: tauri::State<'_, crate::AppState>,
) -> Result<usize, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.clean_unused_auto_categories().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn purge_auto_categories(
    state: tauri::State<'_, crate::AppState>,
) -> Result<usize, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.purge_all_auto_categories().map_err(|e| e.to_string())
}

/// Chamado toda vez que o usuario corrige uma classificacao pelo clique
/// direito. Grava em `user_corrections` e alimenta `engine::rules::learn_from_correction`.
#[tauri::command]
pub async fn record_user_correction(
    file_id: String,
    old_category_id: Option<String>,
    new_category_id: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.record_user_correction(&file_id, old_category_id.as_deref(), &new_category_id)
        .map_err(|e| e.to_string())?;

    let correction = UserCorrection {
        id: uuid::Uuid::new_v4().to_string(),
        file_id,
        old_category_id,
        new_category_id,
        corrected_at: chrono::Utc::now().to_rfc3339(),
    };

    crate::engine::rules::learn_from_correction(&correction, &db).map_err(|e| e.to_string())
}

/// Backup de perfil (RF12): zipa a pasta data/ inteira (profile.db + configuracoes).
#[tauri::command]
pub async fn export_profile(
    destination_zip_path: String,
    _state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let data_dir = exe_dir.join("data");
    if !data_dir.exists() {
        return Err("Diretorio de dados nao encontrado para exportacao".to_string());
    }

    let zip_file = File::create(&destination_zip_path).map_err(|e| e.to_string())?;
    let mut zip = ZipWriter::new(zip_file);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let walker = walkdir::WalkDir::new(&data_dir);
    for entry_result in walker {
        let entry = entry_result.map_err(|e| e.to_string())?;
        let path = entry.path();
        let name = path
            .strip_prefix(&data_dir)
            .map_err(|e| e.to_string())?
            .to_string_lossy();

        if name.is_empty() {
            continue;
        }

        if path.is_file() {
            zip.start_file(name, options).map_err(|e| e.to_string())?;
            let mut f = File::open(path).map_err(|e| e.to_string())?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer).map_err(|e| e.to_string())?;
            zip.write_all(&buffer).map_err(|e| e.to_string())?;
        } else if path.is_dir() {
            zip.add_directory(name, options).map_err(|e| e.to_string())?;
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(())
}

/// Restaura o perfil do usuario a partir de um arquivo .zip
#[tauri::command]
pub async fn import_profile(
    source_zip_path: String,
    _state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    let file = File::open(&source_zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let data_dir = exe_dir.join("data");
    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = match file.enclosed_name() {
            Some(path) => data_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
                }
            }
            let mut outfile = File::create(&outpath).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn get_category_history(
    category_id: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<crate::db::models::CategoryHistoryRecord>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_category_history(&category_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_organization_history(
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<crate::db::models::OrganizationSessionSummary>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_organization_history().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn undo_session(
    session_id: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<usize, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.undo_session_moves(&session_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_setting(
    key: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<Option<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_setting(&key).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_setting(
    key: String,
    value: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.set_setting(&key, &value).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_all_user_data(
    confirmation: String,
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    if confirmation.trim().to_lowercase() != "sim" {
        return Err("Confirmação inválida. É necessário digitar exatamente 'sim' para autorizar a limpeza de todos os dados.".to_string());
    }

    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.clear_all_user_data().map_err(|e| e.to_string())
}

