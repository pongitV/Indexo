use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Clone, Debug)]
pub struct FilePreviewData {
    pub filename: String,
    pub path: String,
    pub file_type: String, // "image" | "text" | "pdf" | "spreadsheet" | "audio" | "video" | "binary"
    pub mime_type: String,
    pub size_bytes: u64,
    pub text_content: Option<String>,
    pub data_url: Option<String>,
    pub error: Option<String>,
}

fn base64_encode(data: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        result.push(CHARSET[(b0 >> 2) as usize] as char);
        result.push(CHARSET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARSET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARSET[(b2 & 0x3f) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

#[tauri::command]
pub async fn open_in_explorer(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    if !p.exists() {
        return Err(format!("Caminho não encontrado: {}", path));
    }

    #[cfg(target_os = "windows")]
    {
        let win_path = path.replace('/', "\\");
        if p.is_file() {
            std::process::Command::new("explorer")
                .args(["/select,", &win_path])
                .spawn()
                .map_err(|e| format!("Falha ao abrir explorer: {}", e))?;
        } else {
            std::process::Command::new("explorer")
                .arg(&win_path)
                .spawn()
                .map_err(|e| format!("Falha ao abrir explorer: {}", e))?;
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let target = if p.is_file() {
            p.parent().unwrap_or(p)
        } else {
            p
        };
        std::process::Command::new("open")
            .arg(target)
            .spawn()
            .map_err(|e| format!("Falha ao abrir gerenciador: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_file_preview(path: String) -> Result<FilePreviewData, String> {
    let p = Path::new(&path);
    if !p.exists() || !p.is_file() {
        return Err(format!("Arquivo não encontrado ou inválido: {}", path));
    }

    let meta = std::fs::metadata(p).map_err(|e| e.to_string())?;
    let size_bytes = meta.len();
    let filename = p
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "arquivo".to_string());
    let ext = p
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let image_exts = ["png", "jpg", "jpeg", "webp", "gif", "svg", "bmp", "ico"];
    let audio_exts = ["mp3", "wav", "flac", "aac", "ogg", "m4a"];
    let video_exts = ["mp4", "webm", "mkv", "avi", "mov"];

    if image_exts.contains(&ext.as_str()) {
        let mime = match ext.as_str() {
            "png" => "image/png",
            "jpg" | "jpeg" => "image/jpeg",
            "webp" => "image/webp",
            "gif" => "image/gif",
            "svg" => "image/svg+xml",
            "bmp" => "image/bmp",
            "ico" => "image/x-icon",
            _ => "image/png",
        };
        if size_bytes <= 30 * 1024 * 1024 {
            match std::fs::read(p) {
                Ok(bytes) => {
                    let base64_str = base64_encode(&bytes);
                    return Ok(FilePreviewData {
                        filename,
                        path,
                        file_type: "image".to_string(),
                        mime_type: mime.to_string(),
                        size_bytes,
                        text_content: None,
                        data_url: Some(format!("data:{};base64,{}", mime, base64_str)),
                        error: None,
                    });
                }
                Err(e) => {
                    return Ok(FilePreviewData {
                        filename,
                        path,
                        file_type: "image".to_string(),
                        mime_type: mime.to_string(),
                        size_bytes,
                        text_content: None,
                        data_url: None,
                        error: Some(format!("Falha ao ler imagem: {}", e)),
                    });
                }
            }
        }
    }

    if audio_exts.contains(&ext.as_str()) {
        let mime = format!("audio/{}", ext);
        if size_bytes <= 25 * 1024 * 1024 {
            if let Ok(bytes) = std::fs::read(p) {
                let base64_str = base64_encode(&bytes);
                return Ok(FilePreviewData {
                    filename,
                    path,
                    file_type: "audio".to_string(),
                    mime_type: mime.clone(),
                    size_bytes,
                    text_content: None,
                    data_url: Some(format!("data:{};base64,{}", mime, base64_str)),
                    error: None,
                });
            }
        }
    }

    if video_exts.contains(&ext.as_str()) {
        let mime = format!("video/{}", ext);
        if size_bytes <= 50 * 1024 * 1024 {
            if let Ok(bytes) = std::fs::read(p) {
                let base64_str = base64_encode(&bytes);
                return Ok(FilePreviewData {
                    filename,
                    path,
                    file_type: "video".to_string(),
                    mime_type: mime.clone(),
                    size_bytes,
                    text_content: None,
                    data_url: Some(format!("data:{};base64,{}", mime, base64_str)),
                    error: None,
                });
            }
        }
    }

    // Snippet / text extraction
    let max_preview_chars = 30_000;
    match crate::engine::content_extract::extract_text_snippet(p, max_preview_chars) {
        Ok(text) if !text.is_empty() => {
            let file_type = if ext == "pdf" {
                "pdf"
            } else if ["xlsx", "xls", "ods", "csv", "tsv"].contains(&ext.as_str()) {
                "spreadsheet"
            } else {
                "text"
            };
            Ok(FilePreviewData {
                filename,
                path,
                file_type: file_type.to_string(),
                mime_type: format!("text/{}", ext),
                size_bytes,
                text_content: Some(text),
                data_url: None,
                error: None,
            })
        }
        _ => {
            Ok(FilePreviewData {
                filename,
                path,
                file_type: "binary".to_string(),
                mime_type: "application/octet-stream".to_string(),
                size_bytes,
                text_content: None,
                data_url: None,
                error: None,
            })
        }
    }
}
