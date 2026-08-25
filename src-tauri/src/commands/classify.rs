use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tauri::Emitter;

use crate::commands::scan::FileMeta;
use crate::db::models::CategoryRecord;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClassifiedFile {
    pub file_id: String,
    pub path: String,
    pub filename: String,
    pub suggested_category: String,
    pub category_id: String,
    pub category_color: Option<String>,
    pub confidence: f32,
    pub tier_used: u8, // 1 = heuristica, 2 = embedding/cluster, 3 = LLM local
    #[serde(default)]
    pub size_bytes: u64,
    #[serde(default)]
    pub is_already_organized: bool,
    #[serde(default)]
    pub original_relative_folder: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ClassifyProgressPayload {
    pub processed: usize,
    pub total: usize,
    pub current_phase: String, // "heuristics" | "extracting" | "clustering" | "done"
    pub item: Option<ClassifiedFile>,
}

/// Roda o pipeline de 3 camadas de classificação
/// sobre os arquivos de uma sessao de varredura.
#[tauri::command]
pub async fn classify_scanned_files(
    session_id: String,
    window: tauri::Window,
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<ClassifiedFile>, String> {
    let (files, rules, language, root_path) = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let files = db.get_files_by_session(&session_id).map_err(|e| e.to_string())?;
        let rules = db.get_classification_rules().map_err(|e| e.to_string())?;
        let lang = db.get_setting("language").unwrap_or(None).unwrap_or_else(|| "pt-BR".to_string());
        let root = db.get_session_root_path(&session_id).unwrap_or(None).unwrap_or_default();
        (files, rules, lang, root)
    };

    let total_files = files.len();
    if total_files == 0 {
        return Ok(Vec::new());
    }

    let _ = window.emit(
        "classify://progress",
        ClassifyProgressPayload {
            processed: 0,
            total: total_files,
            current_phase: "heuristics".to_string(),
            item: None,
        },
    );

    // =========================================================================
    // ETAPA 0: Detecção de Pastas Já Organizadas / Estruturadas
    // =========================================================================
    let mut already_organized_map: HashMap<usize, String> = HashMap::new();
    for (idx, f) in files.iter().enumerate() {
        if !root_path.is_empty() {
            if let Some(rel_folder) = crate::engine::heuristics::detect_already_organized_folder(&f.original_path, &root_path) {
                already_organized_map.insert(idx, rel_folder);
            }
        }
    }

    // =========================================================================
    // CAMADA 1: Heurísticas instantâneas em paralelo (Rayon)
    // =========================================================================
    let file_metas: Vec<FileMeta> = files
        .iter()
        .map(|f| FileMeta {
            path: f.original_path.clone(),
            filename: f.filename.clone(),
            extension_declared: f.extension_declared.clone(),
            extension_detected: f.extension_detected.clone(),
            size_bytes: f.size_bytes as u64,
            created_at: f.created_at.clone().unwrap_or_default(),
            modified_at: f.modified_at.clone().unwrap_or_default(),
        })
        .collect();

    let heuristic_results: Vec<(usize, crate::engine::heuristics::HeuristicResult)> = file_metas
        .par_iter()
        .enumerate()
        .map(|(idx, meta)| {
            let res = crate::engine::heuristics::classify_by_heuristics(meta, &rules);
            (idx, res)
        })
        .collect();

    let mut resolved_map: HashMap<usize, (String, f32, u8)> = HashMap::new();
    let mut ambiguous_indices: Vec<usize> = Vec::new();

    // Prioridade 0: Pastas já organizadas são preservadas intactas
    for (&idx, rel_folder) in &already_organized_map {
        resolved_map.insert(idx, (rel_folder.clone(), 1.0, 1));
    }

    for (idx, res) in &heuristic_results {
        if already_organized_map.contains_key(idx) {
            continue;
        }

        if !res.needs_deeper_analysis && res.category_guess.is_some() {
            resolved_map.insert(
                *idx,
                (res.category_guess.clone().unwrap(), res.confidence, 1),
            );
        } else {
            ambiguous_indices.push(*idx);
        }
    }

    // =========================================================================
    // CAMADA 2 & 3: Conteúdo, Embeddings e Naming Semântico para os ambíguos
    // =========================================================================
    if !ambiguous_indices.is_empty() {
        let _ = window.emit(
            "classify://progress",
            ClassifyProgressPayload {
                processed: resolved_map.len(),
                total: total_files,
                current_phase: "extracting".to_string(),
                item: None,
            },
        );

        // Extrai snippets e gera embeddings em paralelo
        let extracted_and_embedded: Vec<(usize, String, Vec<f32>)> = ambiguous_indices
            .par_iter()
            .filter_map(|&idx| {
                let f = &files[idx];
                let path = Path::new(&f.original_path);
                let snippet = crate::engine::content_extract::extract_text_snippet(path, 2000)
                    .unwrap_or_default();

                if snippet.trim().is_empty() {
                    return None;
                }

                let embedding = crate::engine::embeddings::compute_embedding(&snippet)
                    .unwrap_or_default();

                Some((idx, snippet, embedding))
            })
            .collect();

        if !extracted_and_embedded.is_empty() {
            let _ = window.emit(
                "classify://progress",
                ClassifyProgressPayload {
                    processed: resolved_map.len(),
                    total: total_files,
                    current_phase: "clustering".to_string(),
                    item: None,
                },
            );

            // Monta lista para agrupamento
            let cluster_inputs: Vec<(String, Vec<f32>)> = extracted_and_embedded
                .iter()
                .map(|(idx, _, emb)| (idx.to_string(), emb.clone()))
                .collect();

            let clusters = crate::engine::embeddings::cluster_files(&cluster_inputs);

            // Mapeamento de idx para snippet
            let snippet_by_idx: HashMap<usize, String> = extracted_and_embedded
                .into_iter()
                .map(|(idx, snip, _)| (idx, snip))
                .collect();

            // Camada 3: Nomeação semântica 1x por cluster
            for cluster in clusters {
                let sample_snippets: Vec<String> = cluster
                    .iter()
                    .take(5)
                    .filter_map(|id_str| id_str.parse::<usize>().ok())
                    .filter_map(|idx| snippet_by_idx.get(&idx).cloned())
                    .collect();

                let cluster_category_name = crate::engine::llm_local::name_cluster(
                    &sample_snippets,
                    &language,
                ).unwrap_or_else(|_| {
                    if language.starts_with("en") {
                        "Analyzed Documents".to_string()
                    } else {
                        "Documentos Analisados".to_string()
                    }
                });

                for id_str in cluster {
                    if let Ok(idx) = id_str.parse::<usize>() {
                        resolved_map.insert(
                            idx,
                            (cluster_category_name.clone(), 0.86, 2),
                        );
                    }
                }
            }
        }
    }

    // =========================================================================
    // ETAPA 4: Refinamento de Subcategorias Hierárquicas Inteligentes
    // =========================================================================
    // Para arquivos não pré-organizados, subdivide em subcategorias (ex: "Fotos e Imagens/Jogos/Zelda")
    let mut unorganized_items: Vec<(String, String, String)> = Vec::new();
    for (idx, f) in files.iter().enumerate() {
        if !already_organized_map.contains_key(&idx) {
            let cat_name = match resolved_map.get(&idx) {
                Some((c, _, _)) => c.clone(),
                None => heuristic_results[idx].1.category_guess.clone().unwrap_or_else(|| {
                    if language.starts_with("en") {
                        "Other Files".to_string()
                    } else {
                        "Outros Arquivos".to_string()
                    }
                }),
            };
            unorganized_items.push((f.id.clone(), f.filename.clone(), cat_name));
        }
    }

    let subcategory_map = crate::engine::subcategories::refine_hierarchical_subcategories(&unorganized_items);

    // =========================================================================
    // PERSISTÊNCIA E CONSTRUÇÃO DA RESPOSTA FINAL
    // =========================================================================
    let mut final_results: Vec<ClassifiedFile> = Vec::with_capacity(total_files);
    let mut category_cache: HashMap<String, CategoryRecord> = HashMap::new();

    let db = state.db.lock().map_err(|e| e.to_string())?;

    for (idx, f) in files.iter().enumerate() {
        let is_organized = already_organized_map.contains_key(&idx);
        let orig_rel = already_organized_map.get(&idx).cloned();

        let (mut cat_name, confidence, tier) = resolved_map
            .remove(&idx)
            .unwrap_or_else(|| {
                if let Some(guess) = &heuristic_results[idx].1.category_guess {
                    (guess.clone(), heuristic_results[idx].1.confidence.max(0.60), 1)
                } else {
                    let default_name = if language.starts_with("en") {
                        "Other Files".to_string()
                    } else {
                        "Outros Arquivos".to_string()
                    };
                    (default_name, 0.50, 1)
                }
            });

        // Se não for pré-organizado e tiver subcategoria refinada calculada
        if !is_organized {
            if let Some(refined_cat) = subcategory_map.get(&f.id) {
                cat_name = refined_cat.clone();
            }
        }

        let category = match category_cache.get(&cat_name) {
            Some(c) => c.clone(),
            None => {
                let cat = db
                    .get_or_create_category(&cat_name, "auto", None)
                    .map_err(|e| e.to_string())?;
                category_cache.insert(cat_name.clone(), cat.clone());
                cat
            }
        };

        // Salva associação no SQLite
        let _ = db.assign_file_category(&f.id, &category.id, confidence, "heuristic");

        let classified = ClassifiedFile {
            file_id: f.id.clone(),
            path: f.original_path.clone(),
            filename: f.filename.clone(),
            suggested_category: category.name.clone(),
            category_id: category.id.clone(),
            category_color: category.color.clone(),
            confidence,
            tier_used: tier,
            size_bytes: f.size_bytes.max(0) as u64,
            is_already_organized: is_organized,
            original_relative_folder: orig_rel,
        };

        final_results.push(classified.clone());

        if (idx + 1) % 25 == 0 || idx + 1 == total_files {
            let _ = window.emit(
                "classify://progress",
                ClassifyProgressPayload {
                    processed: idx + 1,
                    total: total_files,
                    current_phase: if idx + 1 == total_files { "done".to_string() } else { "classifying".to_string() },
                    item: Some(classified),
                },
            );
        }
    }

    Ok(final_results)
}

