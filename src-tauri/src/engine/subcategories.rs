use std::collections::{HashMap, HashSet};

/// Informações sobre agrupamento semântico dinâmico descoberto pela IA
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DiscoveredCluster {
    pub folder_path: String,
    pub token: String,
    pub count: usize,
    pub reason: String,
}

/// Refina categorias planas criando subcategorias hierárquicas dinâmicas (sem hardcode)
/// quando 2 ou mais arquivos da mesma categoria compartilham um assunto, franquia, padrão ou ano comum.
/// Retorna:
/// 1. Mapa de file_id -> subcategoria refinada
/// 2. Lista de sugestões de pastas descobertas dinamicamente
pub fn refine_hierarchical_subcategories(
    items: &[(String, String, String)], // (file_id, filename, current_category)
) -> (HashMap<String, String>, Vec<DiscoveredCluster>) {
    let mut result: HashMap<String, String> = HashMap::new();
    let mut discovered_suggestions: Vec<DiscoveredCluster> = Vec::new();

    // 1. Agrupar itens por categoria principal
    let mut by_category: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for (id, filename, cat) in items {
        by_category
            .entry(cat.clone())
            .or_default()
            .push((id.clone(), filename.clone()));
    }

    for (main_cat, file_list) in by_category {
        // Se a categoria tiver menos de 2 arquivos, mantém na categoria pai
        if file_list.len() < 2 {
            for (id, _) in file_list {
                result.insert(id, main_cat.clone());
            }
            continue;
        }

        // =========================================================================
        // A. PARSER DE ÁUDIO E MÚSICA (Artista - Álbum / Artista - Faixa)
        // =========================================================================
        if main_cat.starts_with("Media/Audios") || main_cat.contains("Audio") || main_cat.contains("Musica") {
            let mut audio_artist_map: HashMap<String, Vec<String>> = HashMap::new(); // Artist -> Vec<file_id>
            let mut file_artist_match: HashMap<String, (String, Option<String>)> = HashMap::new(); // file_id -> (Artist, Option<Album>)

            for (id, filename) in &file_list {
                if let Some((artist, album)) = extract_audio_structure(filename) {
                    audio_artist_map.entry(artist.clone()).or_default().push(id.clone());
                    file_artist_match.insert(id.clone(), (artist, album));
                }
            }

            for (id, _) in &file_list {
                if let Some((artist, maybe_album)) = file_artist_match.get(id) {
                    if let Some(matching_ids) = audio_artist_map.get(artist) {
                        if matching_ids.len() >= 2 {
                            let mut path = format!("{}/{}", main_cat, artist);
                            if let Some(album) = maybe_album {
                                path.push('/');
                                path.push_str(album);
                            }
                            result.insert(id.clone(), path);
                            continue;
                        }
                    }
                }
            }
        }

        // =========================================================================
        // B. MINERAÇÃO DINÂMICA DE TOKENS E N-GRAMAS (Sem listas fixas)
        // =========================================================================
        // Para cada arquivo, extrai tokens limpos significativos
        let mut file_tokens: Vec<(String, Vec<String>)> = Vec::new(); // (file_id, tokens)
        let mut token_counts: HashMap<String, HashSet<String>> = HashMap::new(); // token -> Set<file_id>

        for (id, filename) in &file_list {
            if result.contains_key(id) {
                continue; // já resolvido por áudio
            }

            let tokens = extract_semantic_tokens(filename);
            for t in &tokens {
                token_counts.entry(t.clone()).or_default().insert(id.clone());
            }
            file_tokens.push((id.clone(), tokens));
        }

        // Seleciona os melhores tokens com frequência >= 2 arquivos
        // Ordena por maior frequência e depois por maior comprimento do token (mais específico)
        let mut candidate_tokens: Vec<(String, usize)> = token_counts
            .iter()
            .filter(|(_, ids)| ids.len() >= 2)
            .map(|(t, ids)| (t.clone(), ids.len()))
            .collect();

        candidate_tokens.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| b.0.len().cmp(&a.0.len())));

        // Associa cada arquivo ao seu melhor token compartilhado
        let mut file_assigned: HashSet<String> = HashSet::new();

        for (best_token, count) in candidate_tokens {
            let matching_file_ids = token_counts.get(&best_token).unwrap();
            let unassigned_matches: Vec<&String> = matching_file_ids
                .iter()
                .filter(|id| !file_assigned.contains(*id) && !result.contains_key(*id))
                .collect();

            if unassigned_matches.len() >= 2 {
                let formatted_subfolder = format_subfolder_name(&best_token, &main_cat);
                let subcategory_path = format!("{}/{}", main_cat, formatted_subfolder);

                for id in unassigned_matches {
                    file_assigned.insert(id.clone());
                    result.insert(id.clone(), subcategory_path.clone());
                }

                // Se houver >= 3 arquivos com este padrão descoberto, registra como sugestão proativa da IA
                if count >= 3 {
                    discovered_suggestions.push(DiscoveredCluster {
                        folder_path: subcategory_path.clone(),
                        token: best_token.clone(),
                        count,
                        reason: format!(
                            "Identificado padrão recorrente '{}' em {} arquivos de {}",
                            formatted_subfolder, count, main_cat
                        ),
                    });
                }
            }
        }

        // Arquivos avulsos restantes permanecem na categoria principal
        for (id, _) in file_list {
            if !result.contains_key(&id) {
                result.insert(id, main_cat.clone());
            }
        }
    }

    (result, discovered_suggestions)
}

/// Extrai tokens semânticos limpos de um nome de arquivo
fn extract_semantic_tokens(filename: &str) -> Vec<String> {
    let stem = std::path::Path::new(filename)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    if stem.trim().is_empty() {
        return Vec::new();
    }

    let clean_stem = clean_leading_noise(&stem);
    let raw_tokens: Vec<&str> = clean_stem
        .split(|c: char| c == '_' || c == '-' || c == ' ' || c == '.' || c == '(' || c == ')' || c == '[' || c == ']')
        .filter(|t| !t.trim().is_empty())
        .collect();

    let mut tokens: Vec<String> = Vec::new();

    // 1. Unigramas limpos
    for t in &raw_tokens {
        let t_clean = t.trim().to_lowercase();
        if t_clean.len() >= 3 && !is_stopword_or_noise(&t_clean) {
            tokens.push(t_clean);
        }
    }

    // 2. Bigramas significativos (ex: "super mario", "dark souls", "elden ring", "banco inter")
    for i in 0..raw_tokens.len().saturating_sub(1) {
        let t1 = raw_tokens[i].trim().to_lowercase();
        let t2 = raw_tokens[i + 1].trim().to_lowercase();
        if t1.len() >= 3 && t2.len() >= 3 && !is_stopword_or_noise(&t1) && !is_stopword_or_noise(&t2) {
            tokens.push(format!("{}-{}", t1, t2));
        }
    }

    tokens
}

/// Extrai Artista e Álbum para arquivos de mídia/áudio
fn extract_audio_structure(filename: &str) -> Option<(String, Option<String>)> {
    let stem = std::path::Path::new(filename)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    // Padrão comum: "Artista - Álbum - 01 Faixa" ou "Artista - Faixa"
    if stem.contains(" - ") {
        let parts: Vec<&str> = stem.split(" - ").map(|p| p.trim()).filter(|p| !p.is_empty()).collect();
        if parts.len() >= 3 {
            let artist = capitalize_words(parts[0]);
            let album = capitalize_words(parts[1]);
            return Some((artist, Some(album)));
        } else if parts.len() == 2 {
            let artist = capitalize_words(parts[0]);
            return Some((artist, None));
        }
    }

    None
}

/// Formata o nome da subpasta de forma elegante (PascalCase com hífens)
fn format_subfolder_name(token: &str, category: &str) -> String {
    let cat_lower = category.to_lowercase();

    // Se for jogos ou ROMs e tiver padrão conhecido
    let is_game = cat_lower.contains("jogo") || cat_lower.contains("rom") || cat_lower.contains("executaveis");
    
    // Normaliza partes com hífen
    let parts: Vec<String> = token
        .split('-')
        .map(|p| capitalize_words(p))
        .collect();

    let joined = parts.join("-");

    // Para jogos ou franquias conhecidas de mídia, se for um nome simples
    if is_game && !joined.starts_with("Jogos-") && !joined.starts_with("Saga-") {
        // Se o usuário preferir nomes diretos de franquias:
        joined
    } else {
        joined
    }
}

fn capitalize_words(s: &str) -> String {
    s.split(|c: char| c == '_' || c == '-' || c == ' ')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join("-")
}

fn clean_leading_noise(s: &str) -> &str {
    let lower = s.to_lowercase();
    let noise_prefixes = [
        "img_", "img-", "img ",
        "dsc_", "dsc-", "dsc ",
        "photo_", "photo-", "photo ",
        "screenshot_", "screenshot-", "screenshot ",
        "captura_", "captura-", "captura ",
        "scan_", "scan-", "scan ",
        "doc_", "doc-", "doc ",
        "documento_", "documento-",
        "fatura_", "fatura-", "boleto_", "boleto-",
        "comprovante_", "comprovante-",
        "relatorio_", "relatorio-",
        "whatsapp image ", "whatsapp-image-", "whatsapp_image_",
    ];

    for prefix in noise_prefixes {
        if lower.starts_with(prefix) {
            return &s[prefix.len()..];
        }
    }
    s
}

fn is_stopword_or_noise(t: &str) -> bool {
    let lower = t.to_lowercase();
    let noise = [
        "final", "edit", "copia", "copy", "novo", "new", "temp", "tmp",
        "versao", "version", "v1", "v2", "v3", "v4", "v5", "page", "pag",
        "part", "parte", "doc", "file", "arquivo", "img", "foto", "photo",
        "ano", "mes", "dia", "pdf", "png", "jpg", "mp4", "zip", "txt", "docx",
        "the", "and", "para", "com", "dos", "das", "por", "que",
    ];
    noise.contains(&lower.as_str()) || t.chars().all(|c| c.is_ascii_digit())
}
