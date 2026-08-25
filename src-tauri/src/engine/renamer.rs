use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RenameConfig {
    pub preset: String, // "semantic" | "date_first" | "clean_only" | "custom"
    pub separator: String, // "_" | "-" | " " | "."
    pub case_style: String, // "title" | "lower" | "upper" | "camel" | "snake" | "kebab"
    pub date_format: String, // "YYYY-MM-DD" | "YYYY-MM" | "DD-MM-YYYY" | "none"
    pub include_category: bool,
    pub remove_noise: bool,
    pub custom_template: Option<String>,
    #[serde(default)]
    pub structure_order: Option<Vec<String>>, // ex: ["date", "subject", "clean_name"]
}

impl Default for RenameConfig {
    fn default() -> Self {
        Self {
            preset: "semantic".to_string(),
            separator: "_".to_string(),
            case_style: "title".to_string(),
            date_format: "YYYY-MM".to_string(),
            include_category: true,
            remove_noise: true,
            custom_template: None,
            structure_order: Some(vec!["date".to_string(), "subject".to_string(), "clean_name".to_string()]),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RenameSuggestion {
    pub file_id: String,
    pub current_path: String,
    pub current_filename: String,
    pub proposed_filename: String,
    pub proposed_path: String,
    pub category: String,
    pub category_color: Option<String>,
    pub size_bytes: u64,
    pub is_modified_by_user: bool,
    pub is_ignored: bool,
    pub has_collision: bool,
}

/// Extrai datas no formato YYYY-MM-DD ou YYYY-MM a partir de texto ou nome do arquivo
pub fn extract_date_str(text: &str, format: &str) -> Option<String> {
    if format == "none" {
        return None;
    }

    // Procura padrões de 4 dígitos para ano (2000 a 2099)
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    // 1. Padrão YYYY-MM-DD ou YYYY-MM (mínimo 7 caracteres: YYYY-MM)
    for i in 0..=len.saturating_sub(7) {
        if chars[i] == '2' && chars[i + 1] == '0' && chars[i + 2].is_ascii_digit() && chars[i + 3].is_ascii_digit() {
            let sep1 = chars[i + 4];
            if (sep1 == '-' || sep1 == '_' || sep1 == '.' || sep1 == '/')
                && chars[i + 5].is_ascii_digit() && chars[i + 6].is_ascii_digit()
            {
                let year: String = chars[i..i + 4].iter().collect();
                let month: String = chars[i + 5..i + 7].iter().collect();
                let m_num: u32 = month.parse().unwrap_or(0);
                if (1..=12).contains(&m_num) {
                    if i + 9 < len {
                        let sep2 = chars[i + 7];
                        if (sep2 == '-' || sep2 == '_' || sep2 == '.' || sep2 == '/')
                            && chars[i + 8].is_ascii_digit() && chars[i + 9].is_ascii_digit()
                        {
                            let day: String = chars[i + 8..i + 10].iter().collect();
                            let d_num: u32 = day.parse().unwrap_or(0);
                            if (1..=31).contains(&d_num) {
                                return match format {
                                    "YYYY-MM-DD" => Some(format!("{}-{}-{}", year, month, day)),
                                    "YYYY-MM" => Some(format!("{}-{}", year, month)),
                                    "DD-MM-YYYY" => Some(format!("{}-{}-{}", day, month, year)),
                                    _ => Some(format!("{}-{}", year, month)),
                                };
                            }
                        }
                    }
                    return match format {
                        "YYYY-MM-DD" => Some(format!("{}-{}-01", year, month)),
                        "YYYY-MM" => Some(format!("{}-{}", year, month)),
                        "DD-MM-YYYY" => Some(format!("01-{}-{}", month, year)),
                        _ => Some(format!("{}-{}", year, month)),
                    };
                }
            }
        }
    }

    // 2. Padrão DD-MM-YYYY ou DD_MM_YYYY
    for i in 0..len.saturating_sub(9) {
        if chars[i].is_ascii_digit() && chars[i + 1].is_ascii_digit() {
            let sep1 = chars[i + 2];
            if (sep1 == '-' || sep1 == '_' || sep1 == '.' || sep1 == '/')
                && chars[i + 3].is_ascii_digit() && chars[i + 4].is_ascii_digit()
            {
                let sep2 = chars[i + 5];
                if (sep2 == '-' || sep2 == '_' || sep2 == '.' || sep2 == '/')
                    && chars[i + 6] == '2' && chars[i + 7] == '0' && chars[i + 8].is_ascii_digit() && chars[i + 9].is_ascii_digit()
                {
                    let day: String = chars[i..i + 2].iter().collect();
                    let month: String = chars[i + 3..i + 5].iter().collect();
                    let year: String = chars[i + 6..i + 10].iter().collect();
                    let d_num: u32 = day.parse().unwrap_or(0);
                    let m_num: u32 = month.parse().unwrap_or(0);
                    if (1..=31).contains(&d_num) && (1..=12).contains(&m_num) {
                        return match format {
                            "YYYY-MM-DD" => Some(format!("{}-{}-{}", year, month, day)),
                            "YYYY-MM" => Some(format!("{}-{}", year, month)),
                            "DD-MM-YYYY" => Some(format!("{}-{}-{}", day, month, year)),
                            _ => Some(format!("{}-{}", year, month)),
                        };
                    }
                }
            }
        }
    }

    None
}

/// Remove ruídos comuns de nomes gerados por apps (WhatsApp, scanners, câmeras, hashes)
pub fn clean_filename_noise(filename_without_ext: &str) -> String {
    let mut name = filename_without_ext.to_string();

    let prefixes_to_strip = [
        "whatsapp image ", "whatsapp-image-", "whatsapp_image_",
        "whatsapp document ", "whatsapp-document-", "whatsapp_document_",
        "whatsapp audio ", "whatsapp-audio-", "whatsapp_audio_",
        "whatsapp video ", "whatsapp-video-", "whatsapp_video_",
        "img_", "img-", "img ",
        "image_", "image-", "image ",
        "scan_", "scan-", "scan ",
        "scanned_", "scanned-", "scanned ",
        "doc_", "doc-", "doc ",
        "documento_", "documento-", "documento ",
        "pxl_", "pxl-", "dsc_", "dsc-",
        "screenshot_", "screenshot-", "captura_", "captura-",
        "download_", "download-", "download ",
        "novo_", "novo-", "novo ", "new_", "new-", "new ",
    ];

    let mut lower = name.to_lowercase();
    for prefix in prefixes_to_strip {
        if lower.starts_with(prefix) {
            name = name[prefix.len()..].trim().to_string();
            lower = name.to_lowercase();
        }
    }

    // Remove sufixos como (1), (2), _copy, -copia
    let suffix_patterns = [" (1)", " (2)", " (3)", "_copy", "-copia", "_copia", " - copia"];
    for suf in suffix_patterns {
        if lower.ends_with(suf) {
            name = name[..name.len() - suf.len()].trim().to_string();
            lower = name.to_lowercase();
        }
    }

    // Se o nome ficou vazio ou só números/hashes, retorna nome padrão limpo
    let alphanumeric_count = name.chars().filter(|c| c.is_alphanumeric()).count();
    if alphanumeric_count == 0 {
        return "Arquivo".to_string();
    }

    name
}

/// Sanitiza strings para serem seguras como nomes de arquivo no Windows
pub fn sanitize_filename_part(part: &str) -> String {
    part.chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            _ => c,
        })
        .collect::<String>()
        .trim_matches(|c| c == ' ' || c == '.' || c == '_' || c == '-')
        .to_string()
}

/// Aplica o estilo de capitalização (Title, lower, upper, camel, snake, kebab)
pub fn apply_casing(text: &str, case_style: &str, sep: &str) -> String {
    let words: Vec<&str> = text
        .split(|c: char| c.is_whitespace() || c == '_' || c == '-' || c == '.')
        .filter(|w| !w.is_empty())
        .collect();

    if words.is_empty() {
        return text.to_string();
    }

    match case_style {
        "lower" => words.join(sep).to_lowercase(),
        "upper" => words.join(sep).to_uppercase(),
        "snake" => words.join("_").to_lowercase(),
        "kebab" => words.join("-").to_lowercase(),
        "camel" => {
            let mut result = String::new();
            for (i, word) in words.iter().enumerate() {
                let mut chars = word.chars();
                if let Some(first) = chars.next() {
                    if i == 0 {
                        result.push(first.to_ascii_lowercase());
                    } else {
                        result.push(first.to_ascii_uppercase());
                    }
                    result.push_str(&chars.as_str().to_lowercase());
                }
            }
            result
        }
        "title" | _ => {
            let capitalized_words: Vec<String> = words
                .iter()
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        Some(first) => {
                            let mut s = String::new();
                            s.push(first.to_ascii_uppercase());
                            s.push_str(&chars.as_str().to_lowercase());
                            s
                        }
                        None => String::new(),
                    }
                })
                .collect();
            capitalized_words.join(sep)
        }
    }
}

/// Extrai assunto semântico relevante a partir do conteúdo do texto ou metadados (jogos, locais, documentos)
pub fn extract_content_subject(text: &str) -> Option<String> {
    let lower = text.to_lowercase();

    // 1. Tipos de Documentos e Finanças
    let doc_keywords = [
        ("nota fiscal", "Nota_Fiscal"),
        ("danfe", "Nota_Fiscal_Danfe"),
        ("fatura", "Fatura"),
        ("comprovante de pagamento", "Comprovante_Pagamento"),
        ("comprovante pix", "Comprovante_Pix"),
        ("comprovante", "Comprovante"),
        ("recibo de pagamento", "Recibo_Pagamento"),
        ("recibo", "Recibo"),
        ("extrato bancario", "Extrato_Bancario"),
        ("extrato de conta", "Extrato_Conta"),
        ("extrato", "Extrato"),
        ("holerite", "Holerite"),
        ("contrato de locacao", "Contrato_Locacao"),
        ("contrato de trabalho", "Contrato_Trabalho"),
        ("contrato", "Contrato"),
        ("declaracao de imposto", "Declaracao_IR"),
        ("declaracao", "Declaracao"),
        ("relatorio mensal", "Relatorio_Mensal"),
        ("relatorio financeiro", "Relatorio_Financeiro"),
        ("relatorio", "Relatorio"),
        ("certificado", "Certificado"),
        ("orcamento", "Orcamento"),
        ("proposta comercial", "Proposta_Comercial"),
    ];

    for (k, label) in doc_keywords {
        if lower.contains(k) {
            return Some(label.to_string());
        }
    }

    // 2. Jogos, Softwares e Projetos
    let game_and_app_keywords = [
        ("valorant", "Valorant"),
        ("minecraft", "Minecraft"),
        ("gta v", "GTA_V"),
        ("grand theft auto", "GTA"),
        ("counter strike", "CSGO"),
        ("cs:go", "CSGO"),
        ("cs2", "CS2"),
        ("roblox", "Roblox"),
        ("fortnite", "Fortnite"),
        ("league of legends", "LoL"),
        ("genshin impact", "Genshin"),
        ("elden ring", "Elden_Ring"),
        ("cyberpunk", "Cyberpunk"),
        ("steam", "Steam"),
        ("unity", "Unity"),
        ("unreal engine", "Unreal"),
        ("blender", "Blender"),
        ("photoshop", "Photoshop"),
        ("figma", "Figma"),
        ("obsidian", "Obsidian"),
        ("discord", "Discord"),
    ];

    for (k, label) in game_and_app_keywords {
        if lower.contains(k) {
            return Some(label.to_string());
        }
    }

    // 3. Locais e Temas
    let location_and_theme_keywords = [
        ("sao paulo", "Sao_Paulo"),
        ("rio de janeiro", "Rio_De_Janeiro"),
        ("brasilia", "Brasilia"),
        ("curitiba", "Curitiba"),
        ("salvador", "Salvador"),
        ("screenshot", "Screenshot"),
        ("captura de tela", "Captura_Tela"),
        ("wallpaper", "Wallpaper"),
        ("gravacao", "Gravacao"),
        ("podcast", "Podcast"),
    ];

    for (k, label) in location_and_theme_keywords {
        if lower.contains(k) {
            return Some(label.to_string());
        }
    }

    None
}

/// Extrai o assunto mais específico da categoria ou subcategoria
pub fn extract_category_subject(category: &str) -> Option<String> {
    let cat = category.trim();
    if cat.is_empty() || cat.eq_ignore_ascii_case("Outros") || cat.eq_ignore_ascii_case("Outros Arquivos") {
        return None;
    }

    // Se for subcategoria multinível (ex: "Fotos e Imagens/Jogos/Zelda"), pega a folha mais específica
    let cat_norm = cat.replace('\\', "/");
    let segments: Vec<&str> = cat_norm
        .split('/')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if let Some(&last_seg) = segments.last() {
        if !last_seg.eq_ignore_ascii_case("Jogos") && !last_seg.eq_ignore_ascii_case("Imagens") {
            return Some(last_seg.to_string());
        }
    }

    if let Some(&first_seg) = segments.first() {
        let first_lower = first_seg.to_lowercase();
        if first_lower.contains("boleto") || first_lower.contains("fatura") {
            return Some("Fatura".to_string());
        } else if first_lower.contains("comprovante") {
            return Some("Comprovante".to_string());
        } else if first_lower.contains("contrato") {
            return Some("Contrato".to_string());
        } else if first_lower.contains("recibo") {
            return Some("Recibo".to_string());
        } else if first_lower.contains("relat") {
            return Some("Relatorio".to_string());
        } else if first_lower.contains("planilha") {
            return Some("Planilha".to_string());
        }
    }

    segments.last().map(|s| s.to_string())
}

/// Extrai sufixo de sequência numérica do nome original (ex: "foto_01" -> "01", "img (2)" -> "02")
pub fn extract_sequence_number(stem: &str) -> Option<String> {
    let s = stem.trim();
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    // 1. Padrão no final: (1), (2), (01)...
    if s.ends_with(')') {
        if let Some(open_paren) = s.rfind('(') {
            let inner = &s[open_paren + 1..s.len() - 1].trim();
            if let Ok(num) = inner.parse::<u32>() {
                return Some(format!("{:02}", num));
            }
        }
    }

    // 2. Padrão no final com separador: _01, -02, _1, -2
    if len >= 2 {
        let mut num_start = len;
        while num_start > 0 && chars[num_start - 1].is_ascii_digit() {
            num_start -= 1;
        }
        if num_start < len && num_start > 0 {
            let sep = chars[num_start - 1];
            if sep == '_' || sep == '-' || sep == ' ' || sep == '.' {
                let digits: String = chars[num_start..len].iter().collect();
                if let Ok(num) = digits.parse::<u32>() {
                    // Se o número tiver 4 dígitos e for ano (ex: 2024), não é sequência
                    if num >= 1990 && num <= 2099 {
                        return None;
                    }
                    if digits.len() >= 2 {
                        return Some(digits);
                    } else {
                        return Some(format!("{:02}", num));
                    }
                }
            }
        }
    }

    None
}

/// Gera o novo nome proposto para um arquivo baseado nas opções e conteúdo real
pub fn generate_proposed_name(
    filename: &str,
    category: &str,
    extracted_text: Option<&str>,
    file_modified_date: Option<&str>,
    config: &RenameConfig,
) -> String {
    let p = Path::new(filename);
    let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or(filename);
    let ext = p.extension().and_then(|s| s.to_str()).unwrap_or("");

    // 1. Limpa ruídos do nome original se ativado
    let clean_stem = if config.remove_noise {
        clean_filename_noise(stem)
    } else {
        stem.to_string()
    };

    // 2. Extrai data do conteúdo ou metadado
    let date_str = extracted_text
        .and_then(|t| extract_date_str(t, &config.date_format))
        .or_else(|| extract_date_str(stem, &config.date_format))
        .or_else(|| {
            if config.date_format != "none" {
                file_modified_date.and_then(|d| extract_date_str(d, &config.date_format))
            } else {
                None
            }
        });

    // 3. Extrai assunto inteligente pelo conteúdo/texto, nome do arquivo ou categoria/subcategoria
    let cat_subject = if config.include_category {
        extract_category_subject(category)
    } else {
        None
    };

    let text_subject = extracted_text.and_then(extract_content_subject);
    let stem_subject = extract_content_subject(stem);

    let content_subject = if let Some(ts) = text_subject {
        Some(ts)
    } else if let Some(cs) = cat_subject {
        // Se a categoria tiver uma entidade específica (ex: "Zelda", "Enel", "Contrato"), prioriza
        Some(cs)
    } else {
        stem_subject
    };

    // 4. Extrai sequencial numérico se existente
    let seq_num = extract_sequence_number(stem);

    let stem_clean = sanitize_filename_part(&clean_stem);
    let sep = &config.separator;

    let mut final_stem = stem_clean.clone();
    if let Some(subj) = &content_subject {
        let lower_subj = subj.to_lowercase();
        let lower_stem = final_stem.to_lowercase();
        if lower_stem == lower_subj {
            final_stem.clear();
        } else if lower_stem.starts_with(&format!("{}_", lower_subj))
            || lower_stem.starts_with(&format!("{}-", lower_subj))
            || lower_stem.starts_with(&format!("{} ", lower_subj))
            || lower_stem.starts_with(&format!("{}.", lower_subj))
        {
            final_stem = final_stem[subj.len() + 1..].to_string();
        } else if lower_stem.ends_with(&format!("_{}", lower_subj))
            || lower_stem.ends_with(&format!("-{}", lower_subj))
            || lower_stem.ends_with(&format!(" {}", lower_subj))
            || lower_stem.ends_with(&format!(".{}", lower_subj))
        {
            final_stem = final_stem[..final_stem.len() - subj.len() - 1].to_string();
        }
    }

    let mut parts: Vec<String> = Vec::new();

    let default_order = vec!["date".to_string(), "subject".to_string(), "clean_name".to_string()];
    let order = config.structure_order.as_ref().unwrap_or(&default_order);

    for element in order {
        match element.as_str() {
            "date" => {
                if let Some(d) = &date_str {
                    if !parts.contains(d) {
                        parts.push(d.clone());
                    }
                }
            }
            "subject" => {
                if let Some(subj) = &content_subject {
                    if !parts.contains(subj) {
                        parts.push(subj.clone());
                    }
                }
            }
            "clean_name" => {
                if !final_stem.is_empty() {
                    let lower_stem = final_stem.to_lowercase();
                    let is_redundant = parts.iter().any(|p| p.to_lowercase() == lower_stem);
                    if !is_redundant {
                        parts.push(final_stem.clone());
                    }
                }
            }
            _ => {}
        }
    }

    // Se tiver sequencial e ele não estiver no clean_name, anexa ao final
    if let Some(seq) = &seq_num {
        let has_seq_already = parts.iter().any(|p| p.ends_with(seq));
        if !has_seq_already {
            parts.push(seq.clone());
        }
    }

    if parts.is_empty() {
        parts.push(if !stem_clean.is_empty() { stem_clean } else { "Arquivo".to_string() });
    }

    let cased_parts: Vec<String> = parts
        .iter()
        .map(|p| {
            if p.chars().all(|c| c.is_ascii_digit() || c == '-') && p.contains('-') {
                p.clone()
            } else {
                apply_casing(p, &config.case_style, sep)
            }
        })
        .collect();

    let joined = cased_parts.join(sep);

    if ext.is_empty() {
        joined
    } else {
        format!("{}.{}", joined, ext)
    }
}

/// Resolve colisões de nomes dentro de um diretório atribuindo sufixos sequenciais (1), (2)...
pub fn resolve_name_collisions(suggestions: &mut [RenameSuggestion]) {
    let mut used_paths: HashSet<String> = HashSet::new();
    let mut folder_counts: HashMap<String, usize> = HashMap::new();

    for s in suggestions.iter_mut() {
        if s.is_ignored {
            continue;
        }

        let parent_dir = Path::new(&s.current_path)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let mut candidate_name = s.proposed_filename.clone();
        let mut candidate_path = Path::new(&parent_dir)
            .join(&candidate_name)
            .to_string_lossy()
            .to_string();

        if used_paths.contains(&candidate_path.to_lowercase()) {
            s.has_collision = true;
            let stem = Path::new(&candidate_name)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&candidate_name)
                .to_string();
            let ext = Path::new(&candidate_name)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            let count_key = format!("{}::{}", parent_dir.to_lowercase(), candidate_name.to_lowercase());
            let count = folder_counts.entry(count_key).or_insert(1);

            while used_paths.contains(&candidate_path.to_lowercase()) {
                candidate_name = if ext.is_empty() {
                    format!("{} ({})", stem, count)
                } else {
                    format!("{} ({}).{}", stem, count, ext)
                };
                candidate_path = Path::new(&parent_dir)
                    .join(&candidate_name)
                    .to_string_lossy()
                    .to_string();
                *count += 1;
            }
        }

        used_paths.insert(candidate_path.to_lowercase());
        s.proposed_filename = candidate_name;
        s.proposed_path = candidate_path;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_filename_noise() {
        assert_eq!(clean_filename_noise("WhatsApp Image 2024-05-10 at 14.30.22"), "2024-05-10 at 14.30.22");
        assert_eq!(clean_filename_noise("IMG_20260515_91823"), "20260515_91823");
        assert_eq!(clean_filename_noise("Scan_0012_contrato"), "0012_contrato");
        assert_eq!(clean_filename_noise("relatorio_financeiro (1)"), "relatorio_financeiro");
    }

    #[test]
    fn test_extract_date_str() {
        let date1 = extract_date_str("Fatura Vencimento 2026-05-15 valor R$ 150", "YYYY-MM");
        assert_eq!(date1, Some("2026-05".to_string()));

        let date2 = extract_date_str("documento_15-08-2024_comprovante", "YYYY-MM-DD");
        assert_eq!(date2, Some("2024-08-15".to_string()));
    }

    #[test]
    fn test_generate_proposed_name() {
        let config = RenameConfig {
            preset: "semantic".to_string(),
            separator: "_".to_string(),
            case_style: "title".to_string(),
            date_format: "YYYY-MM".to_string(),
            include_category: true,
            remove_noise: true,
            custom_template: None,
            structure_order: None,
        };

        let proposed = generate_proposed_name(
            "IMG_enel_maio.pdf",
            "",
            Some("Enel Distribuição Fatura consumo 2026-05-12"),
            None,
            &config,
        );

        assert!(proposed.starts_with("2026-05_Fatura"));
        assert!(proposed.ends_with(".pdf"));
    }

    #[test]
    fn test_reorder_structure() {
        let config = RenameConfig {
            preset: "custom".to_string(),
            separator: "-".to_string(),
            case_style: "title".to_string(),
            date_format: "YYYY-MM".to_string(),
            include_category: false,
            remove_noise: true,
            custom_template: None,
            structure_order: Some(vec!["subject".to_string(), "clean_name".to_string(), "date".to_string()]),
        };

        let proposed = generate_proposed_name(
            "IMG_recibo_aluguel.pdf",
            "",
            Some("Recibo referente ao mes de 2026-06"),
            None,
            &config,
        );

        // Ordem: Assunto (Recibo) - Nome Limpo (Aluguel) - Data (2026-06)
        assert_eq!(proposed, "Recibo-Aluguel-2026-06.pdf");
    }

    #[test]
    fn test_category_subject_and_sequence() {
        let config = RenameConfig {
            preset: "semantic".to_string(),
            separator: "_".to_string(),
            case_style: "title".to_string(),
            date_format: "none".to_string(),
            include_category: true,
            remove_noise: true,
            custom_template: None,
            structure_order: Some(vec!["subject".to_string(), "clean_name".to_string()]),
        };

        // Arquivo de jogo com subcategoria e sequencial
        let p1 = generate_proposed_name(
            "IMG_zelda_totk_01.png",
            "Fotos e Imagens/Jogos/Zelda",
            None,
            None,
            &config,
        );
        assert_eq!(p1, "Zelda_Totk_01.png");

        let p2 = generate_proposed_name(
            "IMG_screenshot_02.png",
            "Fotos e Imagens/Jogos/Zelda",
            None,
            None,
            &config,
        );
        // "screenshot_" é removido por clean_filename_noise, preservando a entidade e o sequencial
        assert_eq!(p2, "Zelda_02.png");

        // Fatura com subcategoria da Enel
        let p3 = generate_proposed_name(
            "doc_energia_janeiro.pdf",
            "Boletos e Faturas/Enel",
            None,
            None,
            &config,
        );
        assert_eq!(p3, "Enel_Energia_Janeiro.pdf");
    }
}
