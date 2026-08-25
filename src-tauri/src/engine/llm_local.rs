use anyhow::Result;
use std::collections::HashMap;

/// Camada 3: da nome humano e legivel a um cluster formado pela Camada 2.
/// Roda 1x POR CLUSTER, nunca por arquivo individual.
///
/// Analisa os trechos de texto mais representativos do cluster e sintetiza
/// um nome de categoria claro e curto no idioma selecionado.
pub fn name_cluster(sample_snippets: &[String], language: &str) -> Result<String> {
    if sample_snippets.is_empty() {
        return Ok(if language.starts_with("en") {
            "Miscellaneous Documents".to_string()
        } else {
            "Documentos Diversos".to_string()
        });
    }

    let is_en = language.starts_with("en");
    let combined_text = sample_snippets.join(" ").to_lowercase();

    // Regras de nomeacao semantica refinada por frequencia de dominios
    let category_patterns: &[(&[&str], &str, &str)] = &[
        (&["enel", "cemig", "copel", "cpfl", "energia", "luz", "kwh", "eletropaulo", "fatura de energia"], "Boletos de Luz e Energia", "Electricity & Energy Bills"),
        (&["sabesp", "sanepar", "embasa", "cedae", "saae", "agua e esgoto", "hidrometro", "consumo de agua"], "Contas de Água e Saneamento", "Water & Utility Bills"),
        (&["claro", "vivo", "tim", "oi fibra", "internet banda larga", "fatura telefonica", "telefonia"], "Faturas de Internet e Telefone", "Internet & Phone Bills"),
        (&["fatura de cartao", "fatura nubank", "fatura itau", "fatura bradesco", "fatura santander", "fatura inter", "fatura c6", "cartao de credito"], "Faturas de Cartão de Crédito", "Credit Card Statements"),
        (&["comprovante de pagamento", "comprovante pix", "transferencia bancaria", "ted", "doc bancario", "autenticacao bancaria", "recibo de transferencia"], "Comprovantes de Pagamento", "Payment Receipts"),
        (&["danfe", "documento auxiliar da nota fiscal", "nota fiscal eletronica", "nf-e", "nfse", "chave de acesso"], "Notas Fiscais", "Tax Invoices"),
        (&["imposto de renda", "irpf", "declaracao de ajuste anual", "recibo de entrega irpf", "darf"], "Declarações de Imposto de Renda", "Tax Returns & Declarations"),
        (&["contrato de locacao", "contrato de aluguel", "contrato de prestacao", "contrato social", "aditivo contratual", "rescisao contratual"], "Contratos e Acordos", "Contracts & Agreements"),
        (&["holerite", "demonstrativo de pagamento", "folha de pagamento", "recibo de salario", "contracheque"], "Holerites e Folha de Pagamento", "Pay Slips & Salary"),
        (&["extrato de conta", "extrato bancario", "extrato consolidado", "saldo em conta", "movimentacao bancaria"], "Extratos Bancários", "Bank Statements"),
        (&["curriculo", "curriculum vitae", "experiencia profissional", "formacao academica", "resume"], "Currículos e Perfis Profissionais", "Resumes & CVs"),
        (&["procuracao", "plenos poderes", "outorgante", "outorgado", "cartorio de notas"], "Procurações e Escrituras", "Powers of Attorney"),
        (&["certidao de nascimento", "certidao de casamento", "certidao de obito", "certidao negativa"], "Certidões Civis", "Certificates & Public Records"),
        (&["relatorio de atividades", "relatorio mensal", "relatorio trimestral", "relatorio de vendas", "status report"], "Relatórios e Balanços", "Reports & Status"),
        (&["tcc", "trabalho de conclusao", "artigo cientifico", "monografia", "tese de doutorado", "dissertacao de mestrado"], "Trabalhos e Artigos Acadêmicos", "Academic Papers & Theses"),
        (&["proposta comercial", "orcamento", "cotacao de precos", "briefing do projeto", "pitch comercial"], "Propostas e Orçamentos", "Proposals & Estimates"),
        (&["manual do usuario", "guia de instalacao", "especificacao tecnica", "documentacao tecnica"], "Manuais e Documentações", "Manuals & Specifications"),
        (&["planilha de controle", "fluxo de caixa", "dre", "balanco patrimonial", "orcamento financeiro"], "Planilhas Financeiras", "Financial Spreadsheets"),
    ];

    for (keywords, pt_name, en_name) in category_patterns {
        for kw in *keywords {
            if combined_text.contains(kw) {
                return Ok(if is_en { en_name.to_string() } else { pt_name.to_string() });
            }
        }
    }

    // Extracao de entidades mais frequentes
    let words: Vec<&str> = combined_text
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= 4 && !is_stopword(w))
        .collect();

    let mut freq: HashMap<&str, usize> = HashMap::new();
    for w in words {
        *freq.entry(w).or_insert(0) += 1;
    }

    let mut sorted_words: Vec<(&str, usize)> = freq.into_iter().collect();
    sorted_words.sort_by(|a, b| b.1.cmp(&a.1));

    if let Some((top_word, _)) = sorted_words.first() {
        let capitalized = capitalize(top_word);
        if is_en {
            Ok(format!("Documents — {}", capitalized))
        } else {
            Ok(format!("Documentos — {}", capitalized))
        }
    } else if is_en {
        Ok("Unclassified Documents".to_string())
    } else {
        Ok("Documentos Diversos".to_string())
    }
}

fn is_stopword(w: &str) -> bool {
    let stopwords = [
        "para", "com", "por", "que", "como", "mais", "este", "esta", "esses", "essas",
        "pelo", "pela", "pelos", "pelas", "onde", "quando", "quem", "qual", "quais",
        "sobre", "entre", "depois", "antes", "mesmo", "mesma", "todos", "todas",
        "document", "documento", "arquivo", "file", "page", "pagina", "texto", "snippet",
        "data", "valor", "numero", "codigo", "item", "total", "hora", "dados",
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
