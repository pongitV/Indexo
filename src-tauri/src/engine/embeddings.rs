use anyhow::Result;
use std::collections::HashSet;

pub const VECTOR_DIM: usize = 256;
const TOPIC_START_DIM: usize = 192;
pub const SIMILARITY_THRESHOLD: f32 = 0.62;

/// Camada 2: gera um vetor de embedding local, denso e determinístico a partir
/// do trecho de texto extraído do arquivo.
///
/// Vetoriza termos semânticos, n-gramas e frequências em um vetor normalizado L2 de 256 dimensões.
pub fn compute_embedding(text: &str) -> Result<Vec<f32>> {
    let mut vector = vec![0.0f32; VECTOR_DIM];
    let lower = text.to_lowercase();
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| w.len() >= 2)
        .collect();

    if words.is_empty() {
        return Ok(vector);
    }

    let word_set: HashSet<&str> = words.iter().cloned().collect();

    // 1. Projeção semântica por n-gramas de caracteres e hashing determinístico nos primeiros 192 slots
    let hash_dim = TOPIC_START_DIM;
    for word in &words {
        let chars: Vec<char> = word.chars().collect();
        if chars.len() >= 3 {
            for window in chars.windows(3) {
                let s: String = window.iter().collect();
                let hash = hash_token(&s) % (hash_dim / 2);
                vector[hash] += 1.0;
            }
        }
        if chars.len() >= 4 {
            for window in chars.windows(4) {
                let s: String = window.iter().collect();
                let hash = (hash_token(&s).wrapping_mul(31)) % (hash_dim / 2);
                vector[hash] += 1.5;
            }
        }
        let word_hash = (hash_token(word) % (hash_dim / 2)) + (hash_dim / 2);
        vector[word_hash] += 2.5;
    }

    // 2. Projeções de 32 tópicos semânticos fundamentais (dimensões 192 a 255)
    // Cobertura completa de domínios pessoais, corporativos, fiscais, técnicos e educacionais
    let topic_buckets: &[(&[&str], usize)] = &[
        // 0. Boletos e Energia / Utilities
        (&["boleto", "fatura", "consumo", "vencimento", "kwh", "enel", "luz", "energia", "sabesp", "agua", "mes_referencia", "codigo_barras", "linha_digitavel", "electric", "utility", "power_bill"], 0),
        // 1. Pagamentos, Bancos e Pix / Receipts
        (&["comprovante", "pagamento", "transferencia", "pix", "autenticacao", "banco", "saldo", "extrato", "agencia", "conta_corrente", "liquidacao", "favorecido", "receipt", "bank_statement", "payment"], 1),
        // 2. Notas Fiscais e Tributos / Tax Invoices
        (&["danfe", "nfe", "nf-e", "nfse", "chave_acesso", "imposto", "icms", "iss", "tributario", "destinatario", "emitente", "valor_total", "discriminacao", "invoice", "tax_invoice", "vat"], 2),
        // 3. Contratos e Jurídico / Legal
        (&["contratante", "contratada", "clausula", "foro", "obrigações", "rescisao", "testemunhas", "acordo", "vigencia", "estipulado", "contract", "agreement", "nda", "terms", "clause", "lawsuit"], 3),
        // 4. Documentos Pessoais e Identificação / Personal IDs
        (&["cpf", "identidade", "orgao_emissor", "nascimento", "filiacao", "nacionalidade", "naturalidade", "passaporte", "cnh", "eleitor", "rg", "passport", "driver_license", "social_security"], 4),
        // 5. Relatórios e Gestão / Reports & Management
        (&["relatorio", "trimestre", "indicadores", "kpi", "performance", "metas", "cronograma", "executivo", "diretoria", "status_report", "management", "quarterly", "executive_summary"], 5),
        // 6. Estudos e Acadêmico / Academic & Research
        (&["universidade", "faculdade", "departamento", "artigo", "resumo", "abstract", "metodologia", "referencias", "conclusao", "orientador", "thesis", "dissertation", "academic", "journal", "curriculum"], 6),
        // 7. Planilhas, Balanços e Finanças / Accounting & Sheets
        (&["ativo", "passivo", "balanco", "dre", "lucro", "despesa", "receita", "orcamento", "planilha", "saldo_anterior", "patrimonio", "fluxo_caixa", "accounting", "ledger", "balance_sheet", "ebitda"], 7),
        // 8. Imposto de Renda e Declarações Fiscais / Tax Returns
        (&["irpf", "imposto_renda", "declaracao_anual", "rendimentos", "deducoes", "restituicao", "darf", "das_mei", "tax_return", "revenue_service"], 8),
        // 9. Recursos Humanos e Folha de Pagamento / Payroll & HR
        (&["holerite", "contracheque", "folha_pagamento", "salario", "inss", "fgts", "admissao", "demissao", "ferias", "decimo_terceiro", "payroll", "payslip", "salary", "employee"], 9),
        // 10. Currículos e Carreira / Resumes & Careers
        (&["curriculo", "curriculum_vitae", "experiencia_profissional", "formacao_academica", "habilidades", "competencias", "linkedin", "portfolio", "resume", "cv", "job_history"], 10),
        // 11. Telecomunicações e Internet / Telecom
        (&["claro", "vivo", "tim", "oi_fibra", "banda_larga", "telefonia", "plano_movel", "internet", "fibra_optica", "broadband", "cellular", "telecom"], 11),
        // 12. Cartão de Crédito e Financiamentos / Credit Cards & Loans
        (&["cartao_credito", "nubank", "itau", "bradesco", "santander", "inter", "c6_bank", "limite_disponivel", "fatura_fechada", "parcelamento", "credit_card", "statement", "mastercard", "visa"], 12),
        // 13. Saúde, Medicina e Exames / Healthcare & Medical
        (&["exame", "laudo", "medico", "hospital", "clinica", "paciente", "diagnostico", "receita_medica", "laboratorio", "hemograma", "medical_report", "prescription", "doctor", "health"], 13),
        // 14. Imóveis, Aluguel e Cartório / Real Estate
        (&["locacao", "locador", "locatario", "imovel", "aluguel", "condominio", "iptu", "escritura", "cartorio", "matricula_imovel", "real_estate", "lease", "tenant", "landlord", "property"], 14),
        // 15. Veículos, Trânsito e Transporte / Automotive
        (&["ipva", "licenciamento", "crlv", "multa_transito", "veiculo", "chassi", "placa_veiculo", "renavam", "sinistro", "seguro_auto", "vehicle", "car_insurance", "traffic_ticket"], 15),
        // 16. Código, Programação e Desenvolvimento / Software Dev
        (&["function", "struct", "class", "async", "interface", "import", "export", "return", "const", "let", "impl", "pub", "fn", "namespace", "algorithm", "repository", "commit", "github"], 16),
        // 17. Configurações, DevOps e Infraestrutura / DevOps & Configs
        (&["dockerfile", "yaml", "toml", "json", "kubernetes", "nginx", "database", "postgres", "mysql", "redis", "server", "endpoint", "api_key", "environment", "deployment"], 17),
        // 18. Marketing, Vendas e Comercial / Marketing & Sales
        (&["proposta_comercial", "cotacao", "orcamento_servico", "briefing", "lead", "conversao", "funil_vendas", "campanha", "pitch", "proposal", "quotation", "sales", "marketing"], 18),
        // 19. Design, Áudio, Vídeo e Multimídia / Media & Design
        (&["resolucao", "canvas", "vetor", "camada", "frame", "render", "audio_track", "codec", "bitrate", "typography", "palette", "vector", "layer", "render"], 19),
        // 20. Certidões e Registros Civis / Civil Certificates
        (&["certidao_nascimento", "certidao_casamento", "certidao_obito", "registro_civil", "tabeliao", "reconhecimento_firma", "apostilamento", "public_record", "certificate"], 20),
        // 21. Manuais, Guias e Documentações / Manuals & Docs
        (&["manual_usuario", "guia_instalacao", "especificacao_tecnica", "instrucoes", "passo_a_passo", "user_guide", "documentation", "specification", "troubleshooting"], 21),
    ];

    for (keywords, dim_offset) in topic_buckets {
        let mut count = 0.0f32;
        for kw in *keywords {
            if kw.contains('_') {
                if lower.contains(kw) {
                    count += 4.0;
                }
            } else if word_set.contains(kw) {
                count += 3.5;
            }
        }
        if count > 0.0 {
            let idx = TOPIC_START_DIM + (dim_offset * 2);
            if idx < VECTOR_DIM {
                vector[idx] += count;
                vector[idx + 1] += count * 0.75;
            }
        }
    }

    // 3. Normalização L2 do vetor resultante
    let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude > 0.00001 {
        for val in vector.iter_mut() {
            *val /= magnitude;
        }
    }

    Ok(vector)
}

/// Agrupa arquivos por proximidade de embeddings utilizando clusterização com centróides.
/// Evita encadeamento assimétrico e aglutina arquivos coerentes.
pub fn cluster_files(embeddings: &[(String, Vec<f32>)]) -> Vec<Vec<String>> {
    if embeddings.is_empty() {
        return Vec::new();
    }

    struct Cluster {
        item_ids: Vec<String>,
        centroid: Vec<f32>,
    }

    let mut clusters: Vec<Cluster> = Vec::new();

    for (id, vec) in embeddings {
        if vec.iter().all(|&v| v == 0.0) {
            continue;
        }

        let mut best_match_idx: Option<usize> = None;
        let mut best_similarity = SIMILARITY_THRESHOLD;

        for (idx, cluster) in clusters.iter().enumerate() {
            let sim = cosine_similarity(&cluster.centroid, vec);
            if sim >= best_similarity {
                best_similarity = sim;
                best_match_idx = Some(idx);
            }
        }

        if let Some(c_idx) = best_match_idx {
            // Adiciona ao cluster existente e atualiza o centróide
            let cluster = &mut clusters[c_idx];
            cluster.item_ids.push(id.clone());
            let n = cluster.item_ids.len() as f32;

            for (c_val, &v_val) in cluster.centroid.iter_mut().zip(vec.iter()) {
                *c_val = (*c_val * (n - 1.0) + v_val) / n;
            }
            normalize_vector(&mut cluster.centroid);
        } else {
            // Cria um novo cluster
            clusters.push(Cluster {
                item_ids: vec![id.clone()],
                centroid: vec.clone(),
            });
        }
    }

    clusters.into_iter().map(|c| c.item_ids).collect()
}

/// Normaliza um vetor in-place para norma unitária L2
fn normalize_vector(v: &mut [f32]) {
    let mag: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag > 0.00001 {
        for x in v.iter_mut() {
            *x /= mag;
        }
    }
}

/// Similaridade de cosseno entre dois vetores normalizados (produto escalar)
pub fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
    if v1.len() != v2.len() || v1.is_empty() {
        return 0.0;
    }
    v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum()
}

fn hash_token(token: &str) -> usize {
    let mut h: usize = 5381;
    for b in token.bytes() {
        h = ((h << 5).wrapping_add(h)).wrapping_add(b as usize);
    }
    h
}

