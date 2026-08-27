use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Extensões de arquivos conhecidamente binários (executáveis, mídia pesada, arquivos compactados)
const BINARY_EXTENSIONS: &[&str] = &[
    "exe", "dll", "sys", "so", "dylib", "bin", "iso", "img", "dat", "db", "sqlite",
    "sqlite3", "db3", "pak", "assets", "pdb", "o", "obj", "lib", "a", "class", "pyc",
    "wasm", "node", "mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "mid", "midi",
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "3gp", "zip", "rar", "7z",
    "tar", "gz", "bz2", "xz", "tgz", "ttf", "otf", "woff", "woff2", "eot", "rsrc", "pdata",
    "rdata", "reloc", "vdi", "vmdk", "qcow2", "dmp", "elf",
];

/// Extrai um trecho representativo do conteudo -- nunca o arquivo inteiro,
/// mantendo a Camada 2 rapida e protegida contra arquivos binarios ou corrompidos.
pub fn extract_text_snippet(path: &Path, max_chars: usize) -> Result<String> {
    if !path.exists() || !path.is_file() {
        return Err(anyhow!("Arquivo inexistente ou invalido: {:?}", path));
    }

    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    // Se for imagem suportada, executa o reconhecimento óptico de caracteres (OCR)
    if crate::engine::ocr::is_ocr_supported_extension(&ext) {
        if let Ok(ocr_text) = crate::engine::ocr::extract_text_from_image(path, max_chars) {
            if !ocr_text.trim().is_empty() {
                return Ok(ocr_text);
            }
        }
        return Ok(String::new());
    }

    if BINARY_EXTENSIONS.contains(&ext.as_str()) {
        return Ok(String::new());
    }

    // Se o infer detectar imagem, tenta OCR antes de descartar
    if let Ok(Some(inferred)) = infer::get_from_path(path) {
        let mime = inferred.mime_type();
        if mime.starts_with("image/") {
            if let Ok(ocr_text) = crate::engine::ocr::extract_text_from_image(path, max_chars) {
                if !ocr_text.trim().is_empty() {
                    return Ok(ocr_text);
                }
            }
            return Ok(String::new());
        }

        if mime.starts_with("audio/")
            || mime.starts_with("video/")
            || mime.starts_with("font/")
            || mime == "application/zip"
            || mime == "application/x-tar"
            || mime == "application/x-rar-compressed"
            || mime == "application/x-7z-compressed"
            || mime == "application/vnd.microsoft.portable-executable"
            || mime == "application/x-executable"
            || mime == "application/x-sharedlib"
            || mime == "application/x-mach-binary"
            || mime == "application/octet-stream"
        {
            // Exceto docx/xlsx que sao zips validos para extratores especializados
            if ext != "docx" && ext != "xlsx" && ext != "ods" {
                return Ok(String::new());
            }
        }
    }

    let result = match ext.as_str() {
        "pdf" => extract_pdf(path, max_chars),
        "docx" => extract_docx(path, max_chars),
        "xlsx" | "xls" | "ods" => extract_spreadsheet(path, max_chars),
        "txt" | "md" | "csv" | "tsv" | "json" | "xml" | "html" | "htm" | "log" | "rs" | "ts"
        | "js" | "py" | "c" | "cpp" | "h" | "java" | "sql" | "yaml" | "yml" | "toml" | "ini"
        | "env" | "sh" | "bat" | "ps1" | "rtf" => extract_plain_text(path, max_chars),
        _ => {
            // Tenta ler como texto se tiver tamanho razoavel (< 5MB) e passar no teste de pureza ASCII/UTF-8
            if let Ok(meta) = path.metadata() {
                if meta.len() < 5 * 1024 * 1024 {
                    extract_plain_text(path, max_chars)
                } else {
                    Ok(String::new())
                }
            } else {
                Ok(String::new())
            }
        }
    };

    match result {
        Ok(text) => {
            let cleaned = sanitize_snippet(&text, max_chars);
            Ok(cleaned)
        }
        Err(err) => Err(anyhow!("Falha ao extrair texto de {:?}: {}", path, err)),
    }
}

/// Extracao segura de texto de PDF
fn extract_pdf(path: &Path, max_chars: usize) -> Result<String> {
    let path_buf = path.to_path_buf();
    let output = std::panic::catch_unwind(move || {
        pdf_extract::extract_text(&path_buf)
    });

    match output {
        Ok(Ok(text)) => {
            let snippet: String = text.chars().take(max_chars * 2).collect();
            Ok(snippet)
        }
        Ok(Err(e)) => Err(anyhow!("Erro ao processar PDF: {}", e)),
        Err(_) => Err(anyhow!("Panico capturado ao ler PDF corrompido")),
    }
}

/// Extracao rapida e sem panics de DOCX abrindo o zip interno
fn extract_docx(path: &Path, max_chars: usize) -> Result<String> {
    let file = File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    if let Ok(mut doc_xml) = archive.by_name("word/document.xml") {
        let mut raw_xml = String::new();
        doc_xml.read_to_string(&mut raw_xml)?;

        let mut text = String::with_capacity(max_chars);
        let mut inside_tag = false;

        for ch in raw_xml.chars() {
            if ch == '<' {
                inside_tag = true;
            } else if ch == '>' {
                inside_tag = false;
                text.push(' ');
            } else if !inside_tag {
                text.push(ch);
                if text.len() >= max_chars * 2 {
                    break;
                }
            }
        }
        return Ok(text);
    }

    Ok(String::new())
}

/// Extracao de planilhas usando calamine
fn extract_spreadsheet(path: &Path, max_chars: usize) -> Result<String> {
    use calamine::{open_workbook_auto, Reader};

    let mut workbook = open_workbook_auto(path)?;
    let mut output = String::new();

    for sheet_name in workbook.sheet_names() {
        output.push_str(&format!("Planilha: {}\n", sheet_name));
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            for row in range.rows().take(20) {
                let row_vals: Vec<String> = row
                    .iter()
                    .map(|cell| cell.to_string())
                    .filter(|s| !s.trim().is_empty())
                    .collect();

                if !row_vals.is_empty() {
                    output.push_str(&row_vals.join(" | "));
                    output.push('\n');
                }
                if output.len() >= max_chars * 2 {
                    break;
                }
            }
        }
        if output.len() >= max_chars * 2 {
            break;
        }
    }

    Ok(output)
}

/// Leitura direta de arquivos texto truncando em bytes e validando contra binários
fn extract_plain_text(path: &Path, max_chars: usize) -> Result<String> {
    let mut file = File::open(path)?;
    let byte_limit = (max_chars * 4).min(256 * 1024);
    let mut buffer = vec![0u8; byte_limit];
    let bytes_read = file.read(&mut buffer)?;

    if bytes_read == 0 {
        return Ok(String::new());
    }

    let slice = &buffer[..bytes_read];

    // Checagem rigorosa de arquivo binario (null bytes ou excesso de caracteres de controle)
    if is_binary_buffer(slice) {
        return Ok(String::new());
    }

    let text = String::from_utf8_lossy(slice);
    let snippet: String = text.chars().take(max_chars).collect();
    Ok(snippet)
}

/// Detecta se um buffer contem dados binarios (null bytes, alta taxa de bytes de controle)
fn is_binary_buffer(buffer: &[u8]) -> bool {
    let sample_len = buffer.len().min(1024);
    if sample_len == 0 {
        return false;
    }
    let sample = &buffer[..sample_len];

    let mut null_count = 0;
    let mut control_count = 0;

    for &b in sample {
        if b == 0 {
            null_count += 1;
        } else if b < 32 && b != b'\t' && b != b'\n' && b != b'\r' {
            control_count += 1;
        }
    }

    // Se contiver mais de 1 byte nulo nos primeiros 1024 bytes -> binario
    if null_count > 1 {
        return true;
    }

    // Se mais de 10% dos bytes forem caracteres de controle nao-espaco -> binario
    if (control_count as f32 / sample_len as f32) > 0.10 {
        return true;
    }

    false
}

/// Limpa e sanitiza o texto extraido removendo ruídos e normalizando espacos
fn sanitize_snippet(text: &str, max_chars: usize) -> String {
    let mut cleaned = String::with_capacity(text.len().min(max_chars));
    let mut last_was_space = false;

    for c in text.chars() {
        if c == '\n' || c == '\t' || c == '\r' || c == ' ' {
            if !last_was_space && !cleaned.is_empty() {
                cleaned.push(' ');
                last_was_space = true;
            }
        } else if !c.is_control() {
            cleaned.push(c);
            last_was_space = false;
        }

        if cleaned.len() >= max_chars {
            break;
        }
    }

    cleaned.trim().to_string()
}

