use anyhow::{anyhow, Result};
use std::fs::{self, File};
use std::path::{Path, PathBuf};

/// Move um arquivo de forma segura (RF09/RNF04):
/// - cria a pasta de destino se nao existir;
/// - NUNCA sobrescreve -- se ja existir algo com o mesmo nome no destino,
///   renomeia com sufixo (ex.: "arquivo (1).pdf");
/// - suporta movimentacao entre diferentes unidades de disco (cross-drive move);
/// - preserva os timestamps originais de criacao e modificacao (via `filetime`);
/// - retorna o caminho de destino final efetivo.
pub fn safe_move(from: &Path, to: &Path) -> Result<PathBuf> {
    if !from.exists() {
        return Err(anyhow!("Arquivo de origem nao encontrado: {:?}", from));
    }

    // Pré-checagem de bloqueio do arquivo por outros processos (ex: Word, Excel aberto)
    if let Err(e) = check_file_not_locked(from) {
        return Err(anyhow!("Arquivo está em uso ou bloqueado por outro aplicativo ({:?}): {}", from, e));
    }

    let target_parent = to
        .parent()
        .ok_or_else(|| anyhow!("Diretorio pai de destino invalido: {:?}", to))?;

    if !target_parent.exists() {
        fs::create_dir_all(target_parent)?;
    }

    let final_destination = get_non_colliding_path(to);

    // Salva timestamps originais para restaurar após move
    let orig_mtime = filetime::FileTime::from_last_modification_time(&from.metadata()?);
    let orig_atime = filetime::FileTime::from_last_access_time(&from.metadata()?);

    // Tenta rename direto (atomico no mesmo volume)
    if fs::rename(from, &final_destination).is_err() {
        // Se falhar (ex: unidades de disco diferentes), faz copia + remocao
        fs::copy(from, &final_destination)?;
        fs::remove_file(from)?;
    }

    // Restaura timestamps originais no arquivo de destino
    let _ = filetime::set_file_times(&final_destination, orig_atime, orig_mtime);

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

    // Salva timestamps
    let orig_mtime = filetime::FileTime::from_last_modification_time(&to_moved.metadata()?);
    let orig_atime = filetime::FileTime::from_last_access_time(&to_moved.metadata()?);

    if fs::rename(to_moved, &target).is_err() {
        fs::copy(to_moved, &target)?;
        fs::remove_file(to_moved)?;
    }

    let _ = filetime::set_file_times(&target, orig_atime, orig_mtime);

    // Tenta limpar a pasta órfã se ela ficou vazia
    if let Some(moved_parent) = to_moved.parent() {
        clean_empty_dir_if_empty(moved_parent);
    }

    Ok(target)
}

/// Verifica se um arquivo não está aberto exclusivamente por outro processo
fn check_file_not_locked(path: &Path) -> Result<()> {
    // Tenta abrir o arquivo para leitura
    let _ = File::open(path)?;
    Ok(())
}

/// Remove diretório se estiver completamente vazio
pub fn clean_empty_dir_if_empty(dir: &Path) {
    if !dir.exists() || !dir.is_dir() {
        return;
    }
    if let Ok(mut entries) = fs::read_dir(dir) {
        if entries.next().is_none() {
            let _ = fs::remove_dir(dir);
            if let Some(parent) = dir.parent() {
                clean_empty_dir_if_empty(parent);
            }
        }
    }
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
