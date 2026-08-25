use anyhow::Result;
use std::collections::HashMap;

/// Camada 3: sintetiza um nome legível e humano para clusters formados pela Camada 2.
/// Roda 1x POR CLUSTER, agregando o vocabulário mais representativo.
pub fn name_cluster(sample_snippets: &[String], language: &str) -> Result<String> {
    let is_en = language.starts_with("en");

    if sample_snippets.is_empty() {
        return Ok(if is_en {
            "Miscellaneous Documents".to_string()
        } else {
            "Documentos Diversos".to_string()
        });
    }

    let combined_text = sample_snippets.join(" ").to_lowercase();

    // 1. Padrões semânticos por domínio de alta precisão (PT / EN)
    let category_patterns: &[(&[&str], &str, &str)] = &[
        // Utilities & Contas de Consumo
        (&["enel", "cemig", "copel", "cpfl", "energia eletrica", "consumo de energia", "kwh", "eletropaulo", "fatura de energia", "electric bill", "power bill"], "Boletos de Luz e Energia", "Electricity & Energy Bills"),
        (&["sabesp", "sanepar", "embasa", "cedae", "saae", "agua e esgoto", "hidrometro", "consumo de agua", "water bill", "utility bill"], "Contas de Água e Saneamento", "Water & Utility Bills"),
        (&["claro", "vivo", "tim", "oi fibra", "internet banda larga", "fatura telefonica", "telefonia", "broadband", "phone bill"], "Faturas de Internet e Telefone", "Internet & Phone Bills"),
        (&["fatura de cartao", "fatura nubank", "fatura itau", "fatura bradesco", "fatura santander", "fatura inter", "fatura c6", "cartao de credito", "credit card statement"], "Faturas de Cartão de Crédito", "Credit Card Statements"),
        
        // Pagamentos & Fiscal
        (&["comprovante de pagamento", "comprovante pix", "transferencia bancaria", "ted bancario", "doc bancario", "autenticacao bancaria", "recibo de transferencia", "payment receipt", "bank receipt"], "Comprovantes de Pagamento", "Payment Receipts"),
        (&["danfe", "documento auxiliar da nota fiscal", "nota fiscal eletronica", "nf-e", "nfse", "chave de acesso", "tax invoice"], "Notas Fiscais", "Tax Invoices"),
        (&["imposto de renda", "irpf", "declaracao de ajuste anual", "recibo de entrega irpf", "darf", "das-mei", "tax return"], "Declarações de Imposto de Renda", "Tax Returns & Declarations"),
        
        // Jurídico & Cartório
        (&["contrato de locacao", "contrato de aluguel", "contrato de prestacao", "contrato social", "aditivo contratual", "rescisao contratual", "lease agreement", "service contract"], "Contratos e Acordos", "Contracts & Agreements"),
        (&["procuracao", "plenos poderes", "outorgante", "outorgado", "cartorio de notas", "power of attorney", "public notary"], "Procurações e Escrituras", "Powers of Attorney"),
        (&["certidao de nascimento", "certidao de casamento", "certidao de obito", "certidao negativa", "birth certificate", "marriage certificate"], "Certidões Civis", "Certificates & Public Records"),

        // RH & Trabalho
        (&["holerite", "demonstrativo de pagamento", "folha de pagamento", "recibo de salario", "contracheque", "pay slip", "payroll"], "Holerites e Folha de Pagamento", "Pay Slips & Salary"),
        (&["curriculo", "curriculum vitae", "experiencia profissional", "formacao academica", "resume", "work experience"], "Currículos e Perfis Profissionais", "Resumes & CVs"),
        
        // Finanças & Bancos
        (&["extrato de conta", "extrato bancario", "extrato consolidado", "saldo em conta", "movimentacao bancaria", "bank statement"], "Extratos Bancários", "Bank Statements"),
        (&["planilha de controle", "fluxo de caixa", "dre", "balanco patrimonial", "orcamento financeiro", "balance sheet", "financial statement"], "Planilhas Financeiras", "Financial Spreadsheets"),
        (&["proposta comercial", "orcamento", "cotacao de precos", "briefing do projeto", "pitch comercial", "commercial proposal", "price quotation"], "Propostas e Orçamentos", "Proposals & Estimates"),

        // Saúde & Medicina
        (&["laudo medico", "receita medica", "exame de sangue", "hemograma", "atestado medico", "diagnostico clinico", "medical report", "prescription"], "Exames e Laudos Médicos", "Medical Reports & Exams"),

        // Imóveis & Veículos
        (&["escritura de imovel", "matricula do imovel", "taxa condominial", "iptu", "property deed", "condo fee"], "Documentos Imobiliários", "Real Estate Documents"),
        (&["crlv", "licenciamento de veiculo", "ipva", "multa de transito", "chassi", "vehicle registration", "traffic ticket"], "Documentos de Veículos", "Vehicle Documents"),

        // Educação & Pesquisa
        (&["tcc", "trabalho de conclusao", "artigo cientifico", "monografia", "tese de doutorado", "dissertacao de mestrado", "academic paper", "thesis"], "Trabalhos e Artigos Acadêmicos", "Academic Papers & Theses"),
        (&["relatorio de atividades", "relatorio mensal", "relatorio trimestral", "relatorio de vendas", "status report", "executive report"], "Relatórios e Balanços", "Reports & Status"),
        (&["manual do usuario", "guia de instalacao", "especificacao tecnica", "documentacao tecnica", "user manual", "technical spec"], "Manuais e Documentações", "Manuals & Specifications"),
    ];

    for (keywords, pt_name, en_name) in category_patterns {
        for kw in *keywords {
            if combined_text.contains(kw) {
                return Ok(if is_en { en_name.to_string() } else { pt_name.to_string() });
            }
        }
    }

    // 2. Extração semântica de entidades de alta relevância com validação estrita
    let words: Vec<&str> = combined_text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| is_valid_topic_word(w))
        .collect();

    let mut freq: HashMap<&str, usize> = HashMap::new();
    for w in words {
        *freq.entry(w).or_insert(0) += 1;
    }

    // Filtra palavras que aparecem com frequência significativa (>= 2)
    let mut sorted_words: Vec<(&str, usize)> = freq
        .into_iter()
        .filter(|(_, count)| *count >= 2)
        .collect();
    sorted_words.sort_by(|a, b| b.1.cmp(&a.1));

    if let Some((top_word, _)) = sorted_words.first() {
        let capitalized = capitalize(top_word);
        if is_en {
            Ok(format!("Documents — {}", capitalized))
        } else {
            Ok(format!("Documentos — {}", capitalized))
        }
    } else if is_en {
        Ok("Miscellaneous Documents".to_string())
    } else {
        Ok("Documentos Diversos".to_string())
    }
}

/// Valida se a palavra candidata e um termo semantico valido e nao um ruido binario/tecnico
fn is_valid_topic_word(w: &str) -> bool {
    let len = w.len();
    if !(4..=24).contains(&len) {
        return false;
    }

    if is_stopword(w) {
        return false;
    }

    // Nao pode ser puramente numerico ou ano (ex: 2024, 2025, 2026, 12345)
    if w.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Deve conter pelo menos uma vogal (evita siglas e ruidos hex como "dddd", "b0ic", "hchh")
    let has_vowel = w.chars().any(|c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y' | 'á' | 'é' | 'í' | 'ó' | 'ú' | 'ã' | 'õ' | 'â' | 'ê'));
    if !has_vowel {
        return false;
    }

    // Nao pode ser hash hexadecimal
    if w.chars().all(|c| c.is_ascii_hexdigit()) {
        return false;
    }

    // Termos binarios/tecnicos comuns de compiladores e headers que devem ser descartados
    let forbidden_technical = [
        "cannot", "program", "phys", "mode", "rdata", "pdata", "reloc", "rsrc",
        "ihdr", "idat", "srgb", "gama", "iend", "text", "vec4", "threads",
        "undefined", "null", "true", "false", "function", "class", "struct",
        "import", "export", "return", "const", "interface", "projectconfiguration",
        "populate", "dddd", "aaaa", "untitled", "empty", "license",
    ];
    if forbidden_technical.contains(&w) {
        return false;
    }

    true
}

fn is_stopword(w: &str) -> bool {
    let stopwords = [
        // Portugues basico
        "para", "com", "por", "que", "como", "mais", "este", "esta", "esses", "essas",
        "pelo", "pela", "pelos", "pelas", "onde", "quando", "quem", "qual", "quais",
        "sobre", "entre", "depois", "antes", "mesmo", "mesma", "todos", "todas",
        "muito", "muita", "muitos", "muitas", "outro", "outra", "outros", "outras",
        "cada", "algum", "alguma", "alguns", "algumas", "nenhum", "nenhuma",
        "tambem", "assim", "ainda", "entao", "desde", "pode", "podem", "deve", "devem",
        "estao", "foram", "sendo", "seria", "tenha", "havia", "fazer", "feito",
        // Ingles basico
        "about", "above", "after", "again", "against", "all", "and", "any", "because",
        "been", "before", "being", "below", "between", "both", "but", "by", "can",
        "could", "did", "does", "doing", "down", "during", "each", "few", "for",
        "from", "further", "had", "has", "have", "having", "her", "here", "hers",
        "him", "his", "how", "into", "its", "itself", "just", "more", "most",
        "other", "our", "ours", "out", "over", "same", "should", "some", "such",
        "than", "that", "the", "their", "theirs", "them", "then", "there", "these",
        "they", "this", "those", "through", "too", "under", "until", "very", "was",
        "were", "what", "when", "where", "which", "while", "who", "whom", "why",
        "will", "with", "would", "your", "yours",
        // Termos genericos de arquivos e documentos
        "document", "documento", "arquivo", "file", "page", "pagina", "texto", "snippet",
        "data", "valor", "numero", "codigo", "item", "total", "hora", "dados", "name",
        "title", "subject", "format", "type", "size", "bytes", "section", "table",
        "header", "footer", "content", "version", "date", "status",
        // Meses e dias
        "janeiro", "fevereiro", "marco", "abril", "maio", "junho", "julho", "agosto",
        "setembro", "outubro", "novembro", "dezembro", "segunda", "terca", "quarta",
        "quinta", "sexta", "sabado", "domingo", "january", "february", "march", "april",
        "june", "july", "august", "september", "october", "november", "december",
        "monday", "tuesday", "wednesday", "thursday", "friday", "saturday", "sunday",
    ];
    stopwords.contains(&w)
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

