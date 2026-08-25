pub mod heuristics;
pub mod content_extract;
pub mod embeddings;
pub mod llm_local;
pub mod rules;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use crate::commands::scan::FileMeta;
    use crate::db::models::ClassificationRule;

    #[test]
    fn test_excluded_paths() {
        assert!(heuristics::is_excluded_path(Path::new(r"C:\Windows\System32\cmd.exe")));
        assert!(heuristics::is_excluded_path(Path::new(r"C:\Users\User\AppData\Local\Temp")));
        assert!(heuristics::is_excluded_path(Path::new(r"E:\Project\node_modules\lodash\index.js")));
        assert!(heuristics::is_excluded_path(Path::new(r"E:\Project\.git\config")));
        assert!(heuristics::is_excluded_path(Path::new(r"E:\Downloads\desktop.ini")));

        assert!(!heuristics::is_excluded_path(Path::new(r"E:\Downloads\fatura_enel_maio.pdf")));
        assert!(!heuristics::is_excluded_path(Path::new(r"E:\Downloads\contrato_locacao.docx")));
    }

    #[test]
    fn test_heuristic_classification_finance() {
        let meta = FileMeta {
            path: r"E:\Downloads\fatura_energia_enel.pdf".to_string(),
            filename: "fatura_energia_enel.pdf".to_string(),
            extension_declared: Some("pdf".to_string()),
            extension_detected: Some("pdf".to_string()),
            size_bytes: 125000,
            created_at: "".to_string(),
            modified_at: "".to_string(),
        };
        let rules: Vec<ClassificationRule> = Vec::new();
        let res = heuristics::classify_by_heuristics(&meta, &rules);

        assert!(res.category_guess.is_some());
        assert_eq!(res.category_guess.unwrap(), "Boletos e Faturas");
        assert!(res.confidence >= 0.75);
    }

    #[test]
    fn test_heuristic_classification_receipt() {
        let meta = FileMeta {
            path: r"E:\Downloads\comprovante_pix_aluguel.pdf".to_string(),
            filename: "comprovante_pix_aluguel.pdf".to_string(),
            extension_declared: Some("pdf".to_string()),
            extension_detected: Some("pdf".to_string()),
            size_bytes: 45000,
            created_at: "".to_string(),
            modified_at: "".to_string(),
        };
        let rules: Vec<ClassificationRule> = Vec::new();
        let res = heuristics::classify_by_heuristics(&meta, &rules);

        assert!(res.category_guess.is_some());
        assert_eq!(res.category_guess.unwrap(), "Comprovantes e Recibos");
        assert!(res.confidence >= 0.75);
    }

    #[test]
    fn test_heuristic_classification_learned_rule_priority() {
        let meta = FileMeta {
            path: r"E:\Downloads\relatorio_anual.pdf".to_string(),
            filename: "relatorio_anual.pdf".to_string(),
            extension_declared: Some("pdf".to_string()),
            extension_detected: Some("pdf".to_string()),
            size_bytes: 500000,
            created_at: "".to_string(),
            modified_at: "".to_string(),
        };
        let custom_rule = ClassificationRule {
            id: "r1".to_string(),
            pattern_type: "filename_regex".to_string(),
            pattern_value: "relatorio_anual".to_string(),
            category_id: "cat_diretoria".to_string(),
            confidence_weight: 0.95,
            created_from: "user".to_string(),
            hit_count: 5,
            created_at: "".to_string(),
            updated_at: "".to_string(),
        };
        let rules = vec![custom_rule];
        let res = heuristics::classify_by_heuristics(&meta, &rules);

        assert_eq!(res.category_guess.unwrap(), "cat_diretoria");
        assert!(res.confidence >= 0.95);
        assert!(!res.needs_deeper_analysis);
    }

    #[test]
    fn test_embeddings_and_clustering() {
        let text1 = "Enel fatura de energia eletrica consumo kwh mes vencimento codigo de barras";
        let text2 = "Enel conta de luz energia eletrica valor total vencimento linha digitavel";
        let text3 = "Contrato de locacao de imovel residencial clausula primeira locador locatario foro";

        let emb1 = embeddings::compute_embedding(text1).unwrap();
        let emb2 = embeddings::compute_embedding(text2).unwrap();
        let emb3 = embeddings::compute_embedding(text3).unwrap();

        let sim_1_2 = embeddings::cosine_similarity(&emb1, &emb2);
        let sim_1_3 = embeddings::cosine_similarity(&emb1, &emb3);

        assert!(sim_1_2 > 0.60, "Contas de energia parecidas devem ter alta similaridade: {}", sim_1_2);
        assert!(sim_1_2 > sim_1_3, "Contas de energia devem ter similaridade maior entre si do que com contrato");

        let files = vec![
            ("f1".to_string(), emb1),
            ("f2".to_string(), emb2),
            ("f3".to_string(), emb3),
        ];
        let clusters = embeddings::cluster_files(&files);
        assert_eq!(clusters.len(), 2);
        assert!(clusters[0].contains(&"f1".to_string()) && clusters[0].contains(&"f2".to_string()));
    }

    #[test]
    fn test_llm_cluster_naming() {
        let snippets_energy = vec![
            "Fatura Enel Energia SA consumo 250 kWh vencimento 15/05/2024".to_string(),
            "Segunda via conta de luz Enel cliente codigo barras".to_string(),
        ];
        let name_pt = llm_local::name_cluster(&snippets_energy, "pt-BR").unwrap();
        assert_eq!(name_pt, "Boletos de Luz e Energia");

        let name_en = llm_local::name_cluster(&snippets_energy, "en-US").unwrap();
        assert_eq!(name_en, "Electricity & Energy Bills");
    }
}
