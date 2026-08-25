use std::collections::HashMap;

/// Refina categorias planas criando subcategorias hierárquicas (ex: "Fotos e Imagens/Zelda", "Boletos e Faturas/Enel")
/// quando 2 ou mais arquivos da mesma categoria compartilham um assunto, franquia, empresa ou prefixo comum.
pub fn refine_hierarchical_subcategories(
    items: &[(String, String, String)], // (file_id, filename, current_category)
) -> HashMap<String, String> { // file_id -> refined_category_path
    let mut result: HashMap<String, String> = HashMap::new();

    // 1. Agrupar itens por categoria principal
    let mut by_category: HashMap<String, Vec<(String, String)>> = HashMap::new();
    for (id, filename, cat) in items {
        by_category
            .entry(cat.clone())
            .or_default()
            .push((id.clone(), filename.clone()));
    }

    for (main_cat, file_list) in by_category {
        // Se a categoria tiver menos de 2 arquivos, não há o que agrupar em subpastas
        if file_list.len() < 2 {
            for (id, _) in file_list {
                result.insert(id, main_cat.clone());
            }
            continue;
        }

        // 2. Extrai entidade/assunto candidato para cada arquivo
        let mut entity_map: HashMap<String, Vec<String>> = HashMap::new(); // entity -> Vec<file_id>
        let mut file_entity_match: HashMap<String, String> = HashMap::new(); // file_id -> entity

        for (id, filename) in &file_list {
            if let Some(entity) = extract_subject_entity(filename, &main_cat) {
                entity_map.entry(entity.clone()).or_default().push(id.clone());
                file_entity_match.insert(id.clone(), entity);
            }
        }

        // 3. Apenas entidades que aparecem em >= 2 arquivos formam subcategoria
        for (id, _) in file_list {
            if let Some(entity) = file_entity_match.get(&id) {
                if let Some(matching_ids) = entity_map.get(entity) {
                    if matching_ids.len() >= 2 {
                        let subcategory_path = format!("{}/{}", main_cat, entity);
                        result.insert(id, subcategory_path);
                        continue;
                    }
                }
            }
            // Arquivos avulsos permanecem na categoria principal
            result.insert(id, main_cat.clone());
        }
    }

    result
}

/// Extrai a entidade semântica principal do nome do arquivo (jogo, empresa, assunto ou prefixo compartilhado)
fn extract_subject_entity(filename: &str, category: &str) -> Option<String> {
    let stem = std::path::Path::new(filename)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();

    if stem.trim().is_empty() {
        return None;
    }

    let lower = stem.to_lowercase();
    let cat_lower = category.to_lowercase();

    // 1. Dicionário de Jogos e Franquias Frequentes
    let games: &[(&[&str], &str)] = &[
        (&["zelda", "botw", "totk", "hyrule"], "Zelda"),
        (&["minecraft", "mcbuild", "mc_"], "Minecraft"),
        (&["gta", "grand_theft_auto", "san_andreas", "vice_city"], "GTA"),
        (&["valorant", "val_"], "Valorant"),
        (&["csgo", "counter_strike", "cs2"], "Counter-Strike"),
        (&["pokemon", "pokémon", "pokemmo"], "Pokémon"),
        (&["mario", "super_mario", "luigi", "bowser"], "Super Mario"),
        (&["fortnite", "fort_"], "Fortnite"),
        (&["elden_ring", "eldenring", "tarnished"], "Elden Ring"),
        (&["dark_souls", "darksouls", "darksouls3"], "Dark Souls"),
        (&["cyberpunk", "cp2077"], "Cyberpunk 2077"),
        (&["skyrim", "elder_scrolls", "tesv"], "Skyrim"),
        (&["witcher", "witcher3", "geralt"], "The Witcher"),
        (&["god_of_war", "gow", "kratos"], "God of War"),
        (&["league_of_legends", "leagueoflegends", "lol_"], "League of Legends"),
        (&["overwatch", "ow2"], "Overwatch"),
        (&["roblox"], "Roblox"),
        (&["genshin", "genshin_impact"], "Genshin Impact"),
        (&["red_dead", "rdr2", "reddead"], "Red Dead Redemption"),
        (&["fifa", "ea_fc", "eafc", "pes20"], "Futebol & FIFA"),
        (&["the_sims", "sims4", "sims3"], "The Sims"),
        (&["hollow_knight", "silksong"], "Hollow Knight"),
        (&["resident_evil", "re2", "re4", "re8"], "Resident Evil"),
        (&["assassins_creed", "ac_valhalla", "ac_odyssey"], "Assassin's Creed"),
    ];

    if cat_lower.contains("foto") || cat_lower.contains("imagem") || cat_lower.contains("vídeo") || cat_lower.contains("jogo") {
        for (aliases, proper_name) in games {
            for alias in *aliases {
                if lower.contains(alias) {
                    return Some(format!("Jogos/{}", proper_name));
                }
            }
        }
    }

    // 2. Dicionário de Empresas, Bancos e Concessionárias (Boletos, Faturas, Comprovantes)
    let companies: &[(&[&str], &str)] = &[
        (&["enel"], "Enel"),
        (&["sabesp"], "Sabesp"),
        (&["sanepar"], "Sanepar"),
        (&["cemig"], "Cemig"),
        (&["copel"], "Copel"),
        (&["cpfl"], "CPFL"),
        (&["claro", "net_claro"], "Claro"),
        (&["vivo", "telefonica"], "Vivo"),
        (&["tim"], "TIM"),
        (&["oi_fibra", "oi_telecom"], "Oi"),
        (&["nubank", "nu_pagamentos"], "Nubank"),
        (&["itau", "itaú"], "Itaú"),
        (&["bradesco"], "Bradesco"),
        (&["santander"], "Santander"),
        (&["inter", "banco_inter"], "Banco Inter"),
        (&["c6", "c6_bank"], "C6 Bank"),
        (&["caixa", "cef"], "Caixa Econômica"),
        (&["banco_do_brasil", "bb_"], "Banco do Brasil"),
        (&["neon"], "Neon"),
        (&["picpay"], "PicPay"),
        (&["mercado_pago", "mercadopago"], "Mercado Pago"),
    ];

    if cat_lower.contains("boleto") || cat_lower.contains("fatura") || cat_lower.contains("comprovante") || cat_lower.contains("extrato") {
        for (aliases, proper_name) in companies {
            for alias in *aliases {
                if lower.contains(alias) {
                    return Some(proper_name.to_string());
                }
            }
        }
    }

    // 3. Dicionário de Assuntos Temáticos Pessoais / Eventos
    let subjects: &[(&[&str], &str)] = &[
        (&["viagem", "ferias", "férias", "trip", "vacation"], "Viagens e Férias"),
        (&["praia", "beach", "mar", "litoral"], "Praia"),
        (&["aniversario", "aniversário", "birthday", "bday"], "Aniversários"),
        (&["casamento", "wedding"], "Casamentos"),
        (&["formatura", "graduation"], "Formaturas"),
        (&["trabalho", "projeto", "project"], "Projetos"),
        (&["screenshot", "captura", "print"], "Capturas de Tela"),
        (&["scan", "scanner", "digitalizado"], "Digitalizados"),
        (&["imovel", "aluguel", "locacao", "locação", "condominio"], "Imóvel e Aluguel"),
        (&["carro", "veiculo", "veículo", "ipva", "crlv", "multa"], "Veículo e Transporte"),
        (&["saude", "saúde", "exame", "medico", "médico", "receita"], "Saúde e Medicina"),
    ];

    for (aliases, proper_name) in subjects {
        for alias in *aliases {
            if lower.contains(alias) {
                return Some(proper_name.to_string());
            }
        }
    }

    // 4. Extração de Prefixo / Token Identificador Comum
    // Ex: "ProjetoAlpha_v1", "ProjetoAlpha_v2" -> "ProjetoAlpha"
    let clean_stem = clean_leading_noise(&stem);
    let tokens: Vec<&str> = clean_stem
        .split(|c: char| c == '_' || c == '-' || c == ' ' || c == '.')
        .filter(|t| t.len() >= 3 && !is_noise_token(t))
        .collect();

    if let Some(first_meaningful_token) = tokens.first() {
        let cap = capitalize_token(first_meaningful_token);
        if cap.len() >= 3 {
            return Some(cap);
        }
    }

    None
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

fn is_noise_token(t: &str) -> bool {
    let lower = t.to_lowercase();
    let noise = [
        "final", "edit", "copia", "copy", "novo", "new", "temp", "tmp",
        "versao", "version", "v1", "v2", "v3", "v4", "v5", "page", "pag",
        "part", "parte", "doc", "file", "arquivo", "img", "foto", "photo",
        "ano", "mes", "dia", "2020", "2021", "2022", "2023", "2024", "2025", "2026",
    ];
    noise.contains(&lower.as_str()) || t.chars().all(|c| c.is_ascii_digit())
}

fn capitalize_token(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
