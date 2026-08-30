use crate::db::models::{DuplicateGroup, DuplicateItem};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Escaneia uma pasta e agrupa arquivos duplicados de forma rápida em múltiplos estágios
pub fn find_duplicates_in_folder(folder_path: &str) -> anyhow::Result<Vec<DuplicateGroup>> {
    let root = Path::new(folder_path);
    if !root.exists() || !root.is_dir() {
        return Ok(Vec::new());
    }

    // 1. Agrupamento inicial por tamanho exato em bytes
    let mut size_map: HashMap<u64, Vec<PathBuf>> = HashMap::new();

    for entry in WalkDir::new(root).follow_links(false).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Ok(metadata) = entry.metadata() {
                let size = metadata.len();
                if size > 0 {
                    size_map.entry(size).or_default().push(entry.into_path());
                }
            }
        }
    }

    // Filtrar apenas tamanhos com 2 ou mais arquivos
    let candidate_sizes: Vec<(u64, Vec<PathBuf>)> = size_map
        .into_iter()
        .filter(|(_, paths)| paths.len() >= 2)
        .collect();

    // 2. Hash rápido de prefixo (64 KB) para descarte de arquivos com conteúdos diferentes
    let mut prefix_map: HashMap<(u64, String), Vec<PathBuf>> = HashMap::new();

    for (size, paths) in candidate_sizes {
        for path in paths {
            if let Ok(prefix_hash) = compute_prefix_hash(&path, 64 * 1024) {
                prefix_map.entry((size, prefix_hash)).or_default().push(path);
            }
        }
    }

    let candidate_prefixes: Vec<Vec<PathBuf>> = prefix_map
        .into_iter()
        .filter_map(|(_, paths)| if paths.len() >= 2 { Some(paths) } else { None })
        .collect();

    // 3. Hash SHA-256 completo para confirmação exata
    let mut full_hash_map: HashMap<String, (u64, Vec<PathBuf>)> = HashMap::new();

    for paths in candidate_prefixes {
        for path in paths {
            if let Ok((hash, size)) = compute_full_hash(&path) {
                let entry = full_hash_map.entry(hash).or_insert((size, Vec::new()));
                entry.1.push(path);
            }
        }
    }

    // 4. Montar os grupos de duplicatas com heurísticas de recomendação do melhor arquivo para manter
    let mut groups: Vec<DuplicateGroup> = Vec::new();
    let mut group_idx = 1;

    for (hash, (size, paths)) in full_hash_map {
        if paths.len() < 2 {
            continue;
        }

        let mut items = Vec::new();
        for path in paths {
            let filename = path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            let metadata = std::fs::metadata(&path).ok();
            let modified_at = metadata
                .and_then(|m| m.modified().ok())
                .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339());

            let resolution = get_image_resolution(&path);

            items.push(DuplicateItem {
                path: path.to_string_lossy().to_string(),
                filename,
                size_bytes: size,
                modified_at,
                resolution,
                is_recommended_to_keep: false,
                is_selected_to_keep: false,
            });
        }

        // Determinar o melhor arquivo para manter:
        // 1º: Nome mais limpo (sem 'copy', '(1)', '- cópia')
        // 2º: Data de modificação mais recente
        // 3º: Menor profundidade de caminho
        let mut best_idx = 0;
        let mut best_score = -100;

        for (idx, item) in items.iter().enumerate() {
            let mut score = 0;
            let lower_name = item.filename.to_lowercase();
            if !lower_name.contains("copy") && !lower_name.contains("copia") && !lower_name.contains("(") {
                score += 50;
            }
            if item.resolution.is_some() {
                score += 20;
            }
            // Caminho mais curto normalmente é o diretório original
            score -= (item.path.len() / 10) as i32;

            if score > best_score {
                best_score = score;
                best_idx = idx;
            }
        }

        if !items.is_empty() {
            items[best_idx].is_recommended_to_keep = true;
            items[best_idx].is_selected_to_keep = true;
        }

        let potential_savings = (items.len().saturating_sub(1) as u64) * size;

        groups.push(DuplicateGroup {
            group_id: format!("dup-group-{}", group_idx),
            hash,
            size_bytes: size,
            items,
            potential_savings_bytes: potential_savings,
        });

        group_idx += 1;
    }

    // Ordenar grupos pelo maior potencial de economia de espaço
    groups.sort_by(|a, b| b.potential_savings_bytes.cmp(&a.potential_savings_bytes));

    Ok(groups)
}

fn compute_prefix_hash(path: &Path, max_bytes: usize) -> anyhow::Result<String> {
    let mut file = File::open(path)?;
    let mut buffer = vec![0u8; max_bytes];
    let bytes_read = file.read(&mut buffer)?;
    let mut hasher = Sha256::new();
    hasher.update(&buffer[..bytes_read]);
    Ok(format!("{:x}", hasher.finalize()))
}

fn compute_full_hash(path: &Path) -> anyhow::Result<(String, u64)> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut total_bytes = 0u64;

    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
        total_bytes += n as u64;
    }

    Ok((format!("{:x}", hasher.finalize()), total_bytes))
}

fn get_image_resolution(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_string_lossy().to_lowercase();
    if matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp") {
        if let Ok((w, h)) = image::image_dimensions(path) {
            return Some(format!("{}x{}", w, h));
        }
    }
    None
}
