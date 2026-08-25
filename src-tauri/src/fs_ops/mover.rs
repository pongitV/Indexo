use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Move um arquivo de forma segura (RF09/RNF04):
/// - cria a pasta de destino se nao existir;
/// - NUNCA sobrescreve -- se ja existir algo com o mesmo nome no destino,
///   renomeia com sufixo (ex.: "arquivo (1).pdf");
/// - suporta movimentacao entre diferentes unidades de disco (cross-drive move);
/// - retorna o caminho de destino final efetivo.
pub fn safe_move(from: &Path, to: &Path) -> Result<PathBuf> {
    if !from.exists() {
        return Err(anyhow!("Arquivo de origem nao encontrado: {:?}", from));
    }

    let target_parent = to
        .parent()
        .ok_or_else(|| anyhow!("Diretorio pai de destino invalido: {:?}", to))?;

    if !target_parent.exists() {
        fs::create_dir_all(target_parent)?;
    }

    let final_destination = get_non_colliding_path(to);

    // Tenta rename direto (atomico no mesmo volume)
    if let Err(_) = fs::rename(from, &final_destination) {
        // Se falhar (ex: unidades de disco diferentes), faz copia + remocao
        fs::copy(from, &final_destination)?;
        fs::remove_file(from)?;
    }

    Ok(final_destination)
}

/// Reverte um arquivo movido de volta para o caminho de origem
pub fn undo_single_move(from_original: &Path, to_moved: &Path) -> Result<PathBuf> {
    if !to_moved.exists() {
        return Err(anyhow!("Arquivo movido nao encontrado no destino: {:?}", to_moved));
    }

    let original_parent = from_original
        .parent()
        .ok_or_else(|| anyhow!("Diretorio pai de origem invalido: {:?}", from_original))?;

    if !original_parent.exists() {
        fs::create_dir_all(original_parent)?;
    }

    let target = get_non_colliding_path(from_original);

    if let Err(_) = fs::rename(to_moved, &target) {
        fs::copy(to_moved, &target)?;
        fs::remove_file(to_moved)?;
    }

    Ok(target)
}

/// Gera um caminho que nao colide com arquivos ja existentes, adicionando (1), (2)...
fn get_non_colliding_path(path: &Path) -> PathBuf {
    if !path.exists() {
        return path.to_path_buf();
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    let mut counter = 1;
    loop {
        let candidate_name = format!("{} ({}){}", stem, counter, ext);
        let candidate_path = parent.join(candidate_name);
        if !candidate_path.exists() {
            return candidate_path;
        }
        counter += 1;
    }
}
