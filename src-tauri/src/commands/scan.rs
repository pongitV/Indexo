use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::UNIX_EPOCH;
use tauri::Emitter;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileMeta {
    pub path: String,
    pub filename: String,
    pub extension_declared: Option<String>,
    pub extension_detected: Option<String>, // via `infer`, lido pelos bytes
    pub size_bytes: u64,
    pub created_at: String,
    pub modified_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ScanSummary {
    pub session_id: String,
    pub total_files: usize,
    pub total_size_bytes: u64,
}

#[derive(Serialize, Clone, Debug)]
pub struct ScanProgressPayload {
    pub files_scanned: usize,
    pub total_size_bytes: u64,
    pub current_file: String,
}

/// Varre recursivamente a pasta selecionada.
/// Emite eventos "scan://progress" para o frontend a cada N arquivos.
#[tauri::command]
pub async fn scan_folder(
    path: String,
    window: tauri::Window,
    state: tauri::State<'_, crate::AppState>,
) -> Result<ScanSummary, String> {
    let target_dir = Path::new(&path);
    if !target_dir.exists() || !target_dir.is_dir() {
        return Err(format!("O caminho fornecido nao e uma pasta valida: {}", path));
    }

    let mut collected_files: Vec<FileMeta> = Vec::new();
    let mut total_size_bytes: u64 = 0;

    let walker = WalkDir::new(target_dir)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !crate::engine::heuristics::is_excluded_path(entry.path()));

    let mut count = 0;

    for entry_result in walker {
        let entry = match entry_result {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let file_path = entry.path();
        let filename = file_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        if filename.is_empty() {
            continue;
        }

        let extension_declared = file_path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase());

        // Deteccao de tipo real lendo os primeiros bytes (magic bytes via infer)
        let extension_detected = detect_magic_extension(file_path);

        let metadata = match file_path.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let size_bytes = metadata.len();
        total_size_bytes += size_bytes;

        let created_at = metadata
            .created()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| DateTime::from_timestamp(d.as_secs() as i64, 0).unwrap_or_default().to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339());

        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| DateTime::from_timestamp(d.as_secs() as i64, 0).unwrap_or_default().to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339());

        collected_files.push(FileMeta {
            path: file_path.to_string_lossy().to_string(),
            filename: filename.clone(),
            extension_declared,
            extension_detected,
            size_bytes,
            created_at,
            modified_at,
        });

        count += 1;
        if count % 50 == 0 {
            let _ = window.emit(
                "scan://progress",
                ScanProgressPayload {
                    files_scanned: count,
                    total_size_bytes,
                    current_file: filename,
                },
            );
        }
    }

    // Persistencia no SQLite
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let session_id = db.create_session(&path).map_err(|e| e.to_string())?;
    db.insert_scanned_files(&session_id, &collected_files)
        .map_err(|e| e.to_string())?;
    db.finish_session(&session_id, collected_files.len(), "done")
        .map_err(|e| e.to_string())?;

    // Emissao de progresso final
    let _ = window.emit(
        "scan://progress",
        ScanProgressPayload {
            files_scanned: collected_files.len(),
            total_size_bytes,
            current_file: "Concluído".to_string(),
        },
    );

    Ok(ScanSummary {
        session_id,
        total_files: collected_files.len(),
        total_size_bytes,
    })
}

/// Le ate 512 bytes para detectar a extensao real do arquivo pelos magic numbers
fn detect_magic_extension(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut buffer = [0u8; 512];
    let bytes_read = file.read(&mut buffer).ok()?;
    if bytes_read == 0 {
        return None;
    }
    infer::get(&buffer[..bytes_read]).map(|t| t.extension().to_string())
}

/// Varre uma lista de arquivos específicos selecionados pelo usuário
#[tauri::command]
pub async fn scan_specific_files(
    file_paths: Vec<String>,
    state: tauri::State<'_, crate::AppState>,
) -> Result<ScanSummary, String> {
    let mut collected_files: Vec<FileMeta> = Vec::new();
    let mut total_size_bytes: u64 = 0;

    for path_str in &file_paths {
        let p = Path::new(path_str);
        if !p.exists() || p.is_dir() || crate::engine::heuristics::is_excluded_path(p) {
            continue;
        }

        let filename = p.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
        let extension_declared = p.extension().map(|e| e.to_string_lossy().to_string().to_lowercase());
        let extension_detected = detect_magic_extension(p);

        let metadata = std::fs::metadata(p).ok();
        let size_bytes = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
        total_size_bytes += size_bytes;

        let created_at = metadata
            .as_ref()
            .and_then(|m| m.created().ok())
            .map(|t| {
                let dt: DateTime<Utc> = t.into();
                dt.to_rfc3339()
            })
            .unwrap_or_default();

        let modified_at = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .map(|t| {
                let dt: DateTime<Utc> = t.into();
                dt.to_rfc3339()
            })
            .unwrap_or_default();

        collected_files.push(FileMeta {
            path: path_str.clone(),
            filename,
            extension_declared,
            extension_detected,
            size_bytes,
            created_at,
            modified_at,
        });
    }

    let root_path = if let Some(first) = collected_files.first() {
        Path::new(&first.path).parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()
    } else {
        "".to_string()
    };

    let total_files = collected_files.len();

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let session_id = db.create_session(&root_path).map_err(|e| e.to_string())?;
    db.insert_scanned_files(&session_id, &collected_files).map_err(|e| e.to_string())?;
    db.finish_session(&session_id, collected_files.len(), "done").map_err(|e| e.to_string())?;

    Ok(ScanSummary {
        session_id,
        total_files,
        total_size_bytes,
    })
}
