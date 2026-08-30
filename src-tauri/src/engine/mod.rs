pub mod heuristics;
pub mod content_extract;
pub mod embeddings;
pub mod llm_local;
pub mod ocr;
pub mod rules;
pub mod renamer;
pub mod subcategories;
pub mod duplicates;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use crate::commands::scan::FileMeta;
    use crate::db::models::ClassificationRule;

    #[test]
    fn test_ocr_supported_extensions() {
        assert!(ocr::is_ocr_supported_extension("png"));
        assert!(ocr::is_ocr_supported_extension("jpg"));
        assert!(ocr::is_ocr_supported_extension("JPEG"));
        assert!(ocr::is_ocr_supported_extension("webp"));
        assert!(ocr::is_ocr_supported_extension("bmp"));
        assert!(ocr::is_ocr_supported_extension("tiff"));
        assert!(!ocr::is_ocr_supported_extension("exe"));
        assert!(!ocr::is_ocr_supported_extension("docx"));
        assert!(!ocr::is_ocr_supported_extension("pdf"));
    }

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

        assert_eq!(emb1.len(), embeddings::VECTOR_DIM);
        assert_eq!(emb2.len(), embeddings::VECTOR_DIM);

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

        // Teste de rejeição de ruído binário
        let junk_snippets = vec![
            "ihdr srgb idat 5jcb4k cannot program reloc rdata pdata".to_string(),
        ];
        let junk_name = llm_local::name_cluster(&junk_snippets, "pt-BR").unwrap();
        assert_eq!(junk_name, "Documentos Diversos");
    }

    #[test]
    fn test_subcategories_clustering() {
        let items = vec![
            ("f1".to_string(), "Zelda_Breath_of_the_Wild_01.png".to_string(), "Fotos e Imagens".to_string()),
            ("f2".to_string(), "Zelda_Tears_of_the_Kingdom.png".to_string(), "Fotos e Imagens".to_string()),
            ("f3".to_string(), "Minecraft_House_Build.png".to_string(), "Fotos e Imagens".to_string()),
            ("f4".to_string(), "Minecraft_Redstone.png".to_string(), "Fotos e Imagens".to_string()),
            ("f5".to_string(), "Foto_Aleatoria_Avulsa.jpg".to_string(), "Fotos e Imagens".to_string()),
            ("f6".to_string(), "Fatura_Enel_Janeiro.pdf".to_string(), "Boletos e Faturas".to_string()),
            ("f7".to_string(), "Conta_Luz_Enel_Fevereiro.pdf".to_string(), "Boletos e Faturas".to_string()),
        ];

        let refined = subcategories::refine_hierarchical_subcategories(&items);

        assert_eq!(refined.get("f1").unwrap(), "Fotos e Imagens/Jogos/Zelda");
        assert_eq!(refined.get("f2").unwrap(), "Fotos e Imagens/Jogos/Zelda");
        assert_eq!(refined.get("f3").unwrap(), "Fotos e Imagens/Jogos/Minecraft");
        assert_eq!(refined.get("f4").unwrap(), "Fotos e Imagens/Jogos/Minecraft");
        // Arquivo avulso fica na categoria principal
        assert_eq!(refined.get("f5").unwrap(), "Fotos e Imagens");
        // Boletos da Enel
        assert_eq!(refined.get("f6").unwrap(), "Boletos e Faturas/Enel");
        assert_eq!(refined.get("f7").unwrap(), "Boletos e Faturas/Enel");
    }

    #[test]
    fn test_detect_already_organized_folder() {
        let root = r"C:\Users\Test\Meus Arquivos";

        // Arquivo dentro de pasta estruturada
        let organized_file = r"C:\Users\Test\Meus Arquivos\Fotos de Férias\Praia\foto1.jpg";
        let detected = heuristics::detect_already_organized_folder(organized_file, root);
        assert_eq!(detected, Some("Fotos de Férias/Praia".to_string()));

        // Arquivo solto na raiz
        let root_file = r"C:\Users\Test\Meus Arquivos\arquivo_solto.pdf";
        let root_detected = heuristics::detect_already_organized_folder(root_file, root);
        assert_eq!(root_detected, None);

        // Arquivo dentro de pasta de lixo/temporária
        let temp_file = r"C:\Users\Test\Meus Arquivos\Temp\teste.txt";
        let temp_detected = heuristics::detect_already_organized_folder(temp_file, root);
        assert_eq!(temp_detected, None);

        // Pasta provisória "Nova Pasta (1)"
        let prov_file = r"C:\Users\Test\Meus Arquivos\Nova Pasta (1)\foto.png";
        let prov_detected = heuristics::detect_already_organized_folder(prov_file, root);
        assert_eq!(prov_detected, None);
    }

    #[test]
    fn test_calculate_folder_coherence() {
        let f1 = FileMeta {
            path: r"C:\Test\Fotos\foto1.jpg".to_string(),
            filename: "foto1.jpg".to_string(),
            extension_declared: Some("jpg".to_string()),
            extension_detected: Some("jpg".to_string()),
            size_bytes: 1000,
            created_at: "".to_string(),
            modified_at: "".to_string(),
        };
        let f2 = FileMeta {
            path: r"C:\Test\Fotos\foto2.jpg".to_string(),
            filename: "foto2.jpg".to_string(),
            extension_declared: Some("jpg".to_string()),
            extension_detected: Some("jpg".to_string()),
            size_bytes: 1000,
            created_at: "".to_string(),
            modified_at: "".to_string(),
        };
        let rules: Vec<ClassificationRule> = Vec::new();
        let coherence = heuristics::calculate_folder_coherence(&[&f1, &f2], &rules);
        assert!(coherence >= 0.70, "Pasta homogênea de fotos deve ter alta coerência");
    }

    #[test]
    fn test_windows_reserved_names_sanitization() {
        assert_eq!(renamer::sanitize_windows_reserved_name("CON"), "CON_File");
        assert_eq!(renamer::sanitize_windows_reserved_name("aux"), "aux_File");
        assert_eq!(renamer::sanitize_windows_reserved_name("NUL"), "NUL_File");
        assert_eq!(renamer::sanitize_windows_reserved_name("Relatorio"), "Relatorio");
    }

    #[test]
    fn test_token_deduplication() {
        let config = renamer::RenameConfig {
            preset: "semantic".to_string(),
            separator: "_".to_string(),
            case_style: "title".to_string(),
            date_format: "none".to_string(),
            include_category: true,
            remove_noise: true,
            custom_template: None,
            structure_order: Some(vec!["subject".to_string(), "clean_name".to_string()]),
        };

        // Arquivo cujo nome original repete o assunto da categoria
        let proposed = renamer::generate_proposed_name(
            "Fatura_Fatura_Enel.pdf",
            "Boletos e Faturas/Enel",
            None,
            None,
            &config,
        );

        // Não deve gerar "Fatura_Fatura_Enel" duplicado
        assert_eq!(proposed, "Enel_Fatura.pdf");
    }

    #[test]
    fn test_extract_exif_date_on_nonexistent() {
        let fake_path = Path::new("C:/fake/path/img.jpg");
        let date = content_extract::extract_exif_date(fake_path);
        assert_eq!(date, None);
    }

    #[test]
    fn test_scripts_and_markdown_not_classified_as_boletos() {
        let rules: Vec<ClassificationRule> = Vec::new();

        // 1. Arquivos de script com nomes que continham substrings de operadoras/bancos
        let bat_runtime = FileMeta {
            path: r"C:\Dev\optimizer_runtime.bat".to_string(),
            filename: "optimizer_runtime.bat".to_string(),
            extension_declared: Some("bat".to_string()),
            extension_detected: Some("bat".to_string()),
            size_bytes: 500,
            created_at: "".to_string(),
            modified_at: "".to_string(),
        };
        let res1 = heuristics::classify_by_heuristics(&bat_runtime, &rules);
        assert_eq!(res1.category_guess, Some("Código e Desenvolvimento".to_string()));

        let bat_estimativa = FileMeta {
            path: r"C:\Dev\estimativa_vendas.bat".to_string(),
            filename: "estimativa_vendas.bat".to_string(),
            extension_declared: Some("bat".to_string()),
            extension_detected: Some("bat".to_string()),
            size_bytes: 500,
            created_at: "".to_string(),
            modified_at: "".to_string(),
        };
        let res2 = heuristics::classify_by_heuristics(&bat_estimativa, &rules);
        assert_eq!(res2.category_guess, Some("Código e Desenvolvimento".to_string()));

        let md_declaracao = FileMeta {
            path: r"C:\Docs\declaracao_escopo.md".to_string(),
            filename: "declaracao_escopo.md".to_string(),
            extension_declared: Some("md".to_string()),
            extension_detected: Some("md".to_string()),
            size_bytes: 1200,
            created_at: "".to_string(),
            modified_at: "".to_string(),
        };
        let res3 = heuristics::classify_by_heuristics(&md_declaracao, &rules);
        assert_eq!(res3.category_guess, Some("Documentos e Documentação".to_string()));

        let bat_survivor = FileMeta {
            path: r"C:\Games\survivor_game.bat".to_string(),
            filename: "survivor_game.bat".to_string(),
            extension_declared: Some("bat".to_string()),
            extension_detected: Some("bat".to_string()),
            size_bytes: 400,
            created_at: "".to_string(),
            modified_at: "".to_string(),
        };
        let res4 = heuristics::classify_by_heuristics(&bat_survivor, &rules);
        assert_eq!(res4.category_guess, Some("Código e Desenvolvimento".to_string()));

        // 2. Boletos reais em PDF continuam sendo perfeitamente detectados
        let pdf_fatura_tim = FileMeta {
            path: r"C:\Docs\fatura_tim_outubro.pdf".to_string(),
            filename: "fatura_tim_outubro.pdf".to_string(),
            extension_declared: Some("pdf".to_string()),
            extension_detected: Some("pdf".to_string()),
            size_bytes: 50000,
            created_at: "".to_string(),
            modified_at: "".to_string(),
        };
        let res5 = heuristics::classify_by_heuristics(&pdf_fatura_tim, &rules);
        assert_eq!(res5.category_guess, Some("Boletos e Faturas".to_string()));

        let pdf_conta_luz = FileMeta {
            path: r"C:\Docs\conta_de_luz_enel_2024.pdf".to_string(),
            filename: "conta_de_luz_enel_2024.pdf".to_string(),
            extension_declared: Some("pdf".to_string()),
            extension_detected: Some("pdf".to_string()),
            size_bytes: 50000,
            created_at: "".to_string(),
            modified_at: "".to_string(),
        };
        let res6 = heuristics::classify_by_heuristics(&pdf_conta_luz, &rules);
        assert_eq!(res6.category_guess, Some("Boletos e Faturas".to_string()));
    }

    #[test]
    fn test_game_and_project_folders_coherence() {
        let rules: Vec<ClassificationRule> = Vec::new();

        // 1. Pasta de Jogo contendo arquivos de tipos diversos (.exe, .dll, .bat, .png, .wav)
        let g_exe = FileMeta {
            path: r"C:\Games\Zelda64\zelda.exe".to_string(),
            filename: "zelda.exe".to_string(),
            extension_declared: Some("exe".to_string()),
            extension_detected: Some("exe".to_string()),
            size_bytes: 5000000,
            created_at: "".to_string(),
            modified_at: "".to_string(),
        };
        let g_dll = FileMeta {
            path: r"C:\Games\Zelda64\engine.dll".to_string(),
            filename: "engine.dll".to_string(),
            extension_declared: Some("dll".to_string()),
            extension_detected: Some("dll".to_string()),
            size_bytes: 2000000,
            created_at: "".to_string(),
            modified_at: "".to_string(),
        };
        let g_bat = FileMeta {
            path: r"C:\Games\Zelda64\launch.bat".to_string(),
            filename: "launch.bat".to_string(),
            extension_declared: Some("bat".to_string()),
            extension_detected: Some("bat".to_string()),
            size_bytes: 100,
            created_at: "".to_string(),
            modified_at: "".to_string(),
        };
        let g_png = FileMeta {
            path: r"C:\Games\Zelda64\splash.png".to_string(),
            filename: "splash.png".to_string(),
            extension_declared: Some("png".to_string()),
            extension_detected: Some("png".to_string()),
            size_bytes: 400000,
            created_at: "".to_string(),
            modified_at: "".to_string(),
        };

        let game_coherence = heuristics::calculate_folder_coherence(&[&g_exe, &g_dll, &g_bat, &g_png], &rules);
        assert_eq!(game_coherence, 0.98, "Pasta de jogo com executável e dll deve ter coerência máxima 0.98");

        // 2. Projeto de código com package.json
        let p_pkg = FileMeta {
            path: r"C:\Projects\Indexo\package.json".to_string(),
            filename: "package.json".to_string(),
            extension_declared: Some("json".to_string()),
            extension_detected: Some("json".to_string()),
            size_bytes: 400,
            created_at: "".to_string(),
            modified_at: "".to_string(),
        };
        let p_md = FileMeta {
            path: r"C:\Projects\Indexo\README.md".to_string(),
            filename: "README.md".to_string(),
            extension_declared: Some("md".to_string()),
            extension_detected: Some("md".to_string()),
            size_bytes: 2000,
            created_at: "".to_string(),
            modified_at: "".to_string(),
        };

        let project_coherence = heuristics::calculate_folder_coherence(&[&p_pkg, &p_md], &rules);
        assert_eq!(project_coherence, 0.98, "Projeto de código com package.json deve ter coerência máxima 0.98");
    }
}
