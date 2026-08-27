use anyhow::Result;
use std::path::Path;

/// Extensões de imagem suportadas para leitura por OCR
pub const OCR_SUPPORTED_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "bmp", "tiff", "tif", "ico",
];

/// Verifica se a extensão é suportada para reconhecimento óptico de caracteres
pub fn is_ocr_supported_extension(ext: &str) -> bool {
    let lower = ext.to_lowercase();
    OCR_SUPPORTED_EXTENSIONS.contains(&lower.as_str())
}

/// Extrai texto legível de um arquivo de imagem utilizando o OCR nativo do Windows (Windows.Media.Ocr).
///
/// Não gera dependências externas de binários ou modelos pesados de IA,
/// processando o reconhecimento em dezenas de milissegundos.
pub fn extract_text_from_image(path: &Path, max_chars: usize) -> Result<String> {
    if !path.exists() || !path.is_file() {
        return Ok(String::new());
    }

    // Proteção contra arquivos gigantes (ignora imagens > 25MB para não sobrecarregar memória)
    if let Ok(meta) = path.metadata() {
        if meta.len() > 25 * 1024 * 1024 {
            return Ok(String::new());
        }
    }

    #[cfg(target_os = "windows")]
    {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| extract_text_windows_native(path))) {
            Ok(Ok(text)) => {
                let cleaned = sanitize_ocr_text(&text, max_chars);
                Ok(cleaned)
            }
            Ok(Err(err)) => {
                // Loga falha não fatal e retorna texto vazio para prosseguir com fallback
                eprintln!("[OCR] Erro ao extrair texto de {:?}: {}", path, err);
                Ok(String::new())
            }
            Err(_) => {
                eprintln!("[OCR] Pânico capturado ao processar imagem {:?}", path);
                Ok(String::new())
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = max_chars;
        Ok(String::new())
    }
}

#[cfg(target_os = "windows")]
fn extract_text_windows_native(path: &Path) -> Result<String> {
    use windows::{
        core::*,
        Globalization::Language,
        Graphics::Imaging::BitmapDecoder,
        Media::Ocr::OcrEngine,
        Storage::Streams::*,
    };

    let bytes = std::fs::read(path)?;
    if bytes.is_empty() {
        return Ok(String::new());
    }

    // Cria o stream de memória WinRT em memória
    let stream = InMemoryRandomAccessStream::new()?;
    let writer = DataWriter::CreateDataWriter(&stream)?;
    writer.WriteBytes(&bytes)?;
    writer.StoreAsync()?.get()?;
    writer.FlushAsync()?.get()?;
    stream.Seek(0)?;

    // Decodifica a imagem com o BitmapDecoder nativo do Windows
    let decoder = BitmapDecoder::CreateAsync(&stream)?.get()?;
    let software_bitmap = decoder.GetSoftwareBitmapAsync()?.get()?;

    // Inicializa a engine de OCR a partir das linguagens instaladas no Windows do usuário
    let engine = match OcrEngine::TryCreateFromUserProfileLanguages() {
        Ok(eng) => eng,
        Err(_) => {
            // Fallback para pt-BR ou en-US
            let lang = Language::CreateLanguage(&HSTRING::from("pt-BR"))
                .or_else(|_| Language::CreateLanguage(&HSTRING::from("en-US")))?;
            OcrEngine::TryCreateFromLanguage(&lang)?
        }
    };

    let ocr_result = engine.RecognizeAsync(&software_bitmap)?.get()?;
    let text_hstring = ocr_result.Text()?;
    let text = text_hstring.to_string();

    Ok(text)
}

/// Sanitiza e normaliza o texto extraído pelo OCR
fn sanitize_ocr_text(text: &str, max_chars: usize) -> String {
    let mut cleaned = String::with_capacity(text.len().min(max_chars));
    let mut last_was_space = false;

    for c in text.chars() {
        if c == '\n' || c == '\r' || c == '\t' || c == ' ' {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocr_extension_check() {
        assert!(is_ocr_supported_extension("png"));
        assert!(is_ocr_supported_extension("JPG"));
        assert!(is_ocr_supported_extension("jpeg"));
        assert!(is_ocr_supported_extension("webp"));
        assert!(is_ocr_supported_extension("bmp"));
        assert!(!is_ocr_supported_extension("pdf"));
        assert!(!is_ocr_supported_extension("zip"));
    }

    #[test]
    fn test_extract_nonexistent_image() {
        let path = Path::new("c:/caminho/nao/existente/imagem_fantasma.png");
        let result = extract_text_from_image(path, 1000).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_sanitize_ocr_text() {
        let raw = "  BANCO DO BRASIL \n\n COMPROVANTE DE PIX \r\n\t VALOR: R$ 150,00  ";
        let cleaned = sanitize_ocr_text(raw, 500);
        assert_eq!(cleaned, "BANCO DO BRASIL COMPROVANTE DE PIX VALOR: R$ 150,00");
    }
}
