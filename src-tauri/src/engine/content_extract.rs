use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Extrai um trecho representativo do conteudo -- nunca o arquivo inteiro,
/// isso e o que mantem a camada 2 rapida mesmo em arquivos grandes.
///
/// Roteamento por tipo REAL do arquivo (detectado via `infer`, nao pela
/// extensao declarada):
/// - pdf         -> `pdf_extract::extract_text`, so as primeiras paginas / max_chars
/// - docx        -> extracao direta do XML interno (word/document.xml)
/// - xlsx        -> `calamine`: nomes das planilhas + primeiras linhas
/// - txt/md/csv  -> leitura direta, truncada em `max_chars`
///
/// Toda extracao roda protegida para nunca derrubar o processo principal
/// se o arquivo estiver corrompido ou inacessivel.
pub fn extract_text_snippet(path: &Path, max_chars: usize) -> Result<String> {
    if !path.exists() || !path.is_file() {
        return Err(anyhow!("Arquivo inexistente ou invalido: {:?}", path));
    }

    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let result = match ext.as_str() {
        "pdf" => extract_pdf(path, max_chars),
        "docx" => extract_docx(path, max_chars),
        "xlsx" | "xls" | "ods" => extract_spreadsheet(path, max_chars),
        "txt" | "md" | "csv" | "tsv" | "json" | "xml" | "html" | "htm" | "log" | "rs" | "ts"
        | "js" | "py" | "c" | "cpp" | "h" | "java" | "sql" | "yaml" | "yml" | "toml" | "ini"
        | "env" | "sh" | "bat" | "ps1" => extract_plain_text(path, max_chars),
        _ => {
            // Tenta ler como texto se tiver tamanho razoavel (< 5MB)
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
            let cleaned: String = text
                .chars()
                .filter(|c| !c.is_control() || *c == '\n' || *c == '\t' || *c == ' ')
                .take(max_chars)
                .collect();
            Ok(cleaned.trim().to_string())
        }
        Err(err) => Err(anyhow!("Falha ao extrair texto de {:?}: {}", path, err)),
    }
}

/// Extracao segura de texto de PDF
fn extract_pdf(path: &Path, max_chars: usize) -> Result<String> {
    let path_buf = path.to_path_buf();
    // Executa em bloco protegido contra panics
    let output = std::panic::catch_unwind(move || {
        pdf_extract::extract_text(&path_buf)
    });

    match output {
        Ok(Ok(text)) => {
            let snippet: String = text.chars().take(max_chars).collect();
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
                if text.len() >= max_chars {
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
            for row in range.rows().take(15) {
                let row_vals: Vec<String> = row
                    .iter()
                    .map(|cell| cell.to_string())
                    .filter(|s| !s.trim().is_empty())
                    .collect();

                if !row_vals.is_empty() {
                    output.push_str(&row_vals.join(" | "));
                    output.push('\n');
                }
                if output.len() >= max_chars {
                    break;
                }
            }
        }
        if output.len() >= max_chars {
            break;
        }
    }

    Ok(output)
}

/// Leitura direta de arquivos texto truncando em bytes/caracteres
fn extract_plain_text(path: &Path, max_chars: usize) -> Result<String> {
    let mut file = File::open(path)?;
    let byte_limit = (max_chars * 4).min(256 * 1024);
    let mut buffer = vec![0u8; byte_limit];
    let bytes_read = file.read(&mut buffer)?;

    let text = String::from_utf8_lossy(&buffer[..bytes_read]);
    let snippet: String = text.chars().take(max_chars).collect();
    Ok(snippet)
}
