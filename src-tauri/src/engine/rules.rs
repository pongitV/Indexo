use anyhow::Result;
use rusqlite::params;
use std::path::Path;

use crate::db::models::UserCorrection;
use crate::db::Database;

/// Toda vez que o usuario corrige uma classificacao (RF08/RF15), isso
/// reforca (ou cria) uma regra em `classification_rules`, aumentando
/// `confidence_weight` e `hit_count`.
///
/// Da proxima vez que um arquivo parecido aparecer, a camada 1
/// (`heuristics::classify_by_heuristics`) resolve direto, sem precisar
/// reprocessar com embeddings/IA -- e assim que o app fica mais rapido
/// E mais preciso especificamente para o vocabulario daquele usuario.
pub fn learn_from_correction(correction: &UserCorrection, db: &Database) -> Result<()> {
    // 1. Obter informacoes do arquivo original
    let file_info: Option<(String, Option<String>, Option<String>)> = db.conn.query_row(
        "SELECT filename, extension_declared, extension_detected FROM files WHERE id = ?1",
        params![correction.file_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).ok();

    if let Some((filename, ext_decl, ext_det)) = file_info {
        let ext = ext_decl.or(ext_det).unwrap_or_default().to_lowercase();
        let stem = Path::new(&filename)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or(filename.clone());

        // 2. Extrair tokens semânticos significativos do nome do arquivo
        let generic_rule_blacklist = [
            "teste", "test", "novo", "new", "script", "app", "main", "index", "file",
            "arquivo", "doc", "documento", "document", "dados", "data", "info", "temp",
            "tmp", "copia", "copy", "untitled", "sem_titulo", "backup", "padrao", "default",
            "final", "versao", "version", "v1", "v2", "patch", "update", "run", "start",
            "exec", "sample", "exemplo", "code", "codigo", "item", "lista", "tudo", "all",
        ];

        let tokens: Vec<&str> = stem
            .split(|c: char| !c.is_alphanumeric())
            .filter(|t| {
                let lower_t = t.to_lowercase();
                t.len() >= 4
                    && !t.chars().all(|c| c.is_ascii_digit())
                    && !generic_rule_blacklist.contains(&lower_t.as_str())
            })
            .collect();

        // Se houver tokens claros e específicos no nome, criar regra de palavra-chave
        if let Some(&primary_keyword) = tokens.first() {
            db.upsert_learned_rule(
                "filename_regex",
                &primary_keyword.to_lowercase(),
                &correction.new_category_id,
                0.85,
                "learned",
            )?;
        }

        // Se a extensao for representativa e compatível com a categoria, aprender associacao
        let is_code_ext = crate::engine::heuristics::is_code_or_script_extension(&ext);
        let is_finance_cat = correction.new_category_id.to_lowercase().contains("boleto")
            || correction.new_category_id.to_lowercase().contains("fatura")
            || correction.new_category_id.to_lowercase().contains("recibo")
            || correction.new_category_id.to_lowercase().contains("comprovante");

        if !ext.is_empty() && ext.len() <= 6 && !(is_code_ext && is_finance_cat) {
            db.upsert_learned_rule(
                "extension",
                &ext,
                &correction.new_category_id,
                0.75,
                "learned",
            )?;
        }
    }

    // 3. Atualizar a associacao na tabela `file_categories`
    db.assign_file_category(
        &correction.file_id,
        &correction.new_category_id,
        1.0,
        "user",
    )?;

    Ok(())
}
