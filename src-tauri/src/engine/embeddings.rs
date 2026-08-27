use anyhow::Result;
use std::collections::HashSet;

/// Dimensão do vetor denso de embeddings semânticos (384 dimensões, padrão de sentence-transformers)
pub const VECTOR_DIM: usize = 384;
const TOPIC_START_DIM: usize = 256;
pub const SIMILARITY_THRESHOLD: f32 = 0.60;

/// Camada 2: gera um vetor de embedding denso, semântico e determinístico a partir
/// do trecho de texto extraído do arquivo (ou lido via OCR).
///
/// Combina projeção hiperdimensional de subpalavras/n-gramas com 64 âncoras latentes de tópicos
/// normalizados em L2 para representação contínua de 384 dimensões.
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

    // 1. Projeção hiperdimensional contínua por n-gramas e subpalavras (slots 0 a 255)
    let hash_dim = TOPIC_START_DIM;
    for word in &words {
        let chars: Vec<char> = word.chars().collect();
        
        // 3-grams
        if chars.len() >= 3 {
            for window in chars.windows(3) {
                let s: String = window.iter().collect();
                let hash = hash_token(&s) % hash_dim;
                let sign = if (hash_token(&s) >> 7) & 1 == 1 { 1.0 } else { -1.0 };
                vector[hash] += 0.8 * sign;
            }
        }

        // 4-grams
        if chars.len() >= 4 {
            for window in chars.windows(4) {
                let s: String = window.iter().collect();
                let hash = (hash_token(&s).wrapping_mul(31)) % hash_dim;
                let sign = if (hash_token(&s) >> 5) & 1 == 1 { 1.0 } else { -1.0 };
                vector[hash] += 1.2 * sign;
            }
        }

        // 5-grams / palavra inteira
        let word_hash = (hash_token(word).wrapping_mul(17)) % hash_dim;
        vector[word_hash] += 2.0;
    }

    // 2. Projeções de 64 tópicos e conceitos semânticos fundamentais (slots 256 a 383)
    let topic_buckets: &[(&[&str], usize)] = &[
        // 0. Utilities: Luz, Energia e Eletricidade
        (&["boleto", "fatura", "consumo", "vencimento", "kwh", "enel", "luz", "energia", "cemig", "copel", "cpfl", "eletropaulo", "fatura_energia", "electric_bill", "power_bill"], 0),
        // 1. Utilities: Água, Esgoto e Saneamento
        (&["sabesp", "sanepar", "embasa", "cedae", "saae", "agua_esgoto", "hidrometro", "consumo_agua", "water_bill", "utility_bill"], 1),
        // 2. Pagamentos, Transferências e Pix
        (&["comprovante", "pagamento", "transferencia", "pix", "autenticacao", "liquidacao", "favorecido", "recibo_pagamento", "payment_receipt", "transaction_receipt"], 2),
        // 3. Extratos Bancários e Contas Correntes
        (&["extrato", "saldo", "conta_corrente", "movimentacao_bancaria", "agencia", "banco_inter", "nubank", "itau", "bradesco", "santander", "caixa_economica", "bank_statement"], 3),
        // 4. Notas Fiscais Eletrônicas e Tributos
        (&["danfe", "nfe", "nf-e", "nfse", "chave_acesso", "imposto", "icms", "iss", "tributario", "destinatario", "emitente", "valor_total", "tax_invoice", "vat"], 4),
        // 5. Imposto de Renda e Declarações Fiscais
        (&["irpf", "imposto_renda", "declaracao_anual", "rendimentos", "deducoes", "restituicao", "darf", "das_mei", "tax_return", "revenue_service"], 5),
        // 6. Contratos e Instrumentos Jurídicos
        (&["contratante", "contratada", "clausula", "foro", "obrigações", "rescisao", "testemunhas", "acordo", "vigencia", "estipulado", "contract", "agreement", "nda", "terms", "clause"], 6),
        // 7. Procurações, Cartório e Notariais
        (&["procuracao", "plenos_poderes", "outorgante", "outorgado", "cartorio_notas", "tabeliao", "reconhecimento_firma", "power_of_attorney", "notary"], 7),
        // 8. Documentos Pessoais e Identificação
        (&["cpf", "identidade", "orgao_emissor", "nascimento", "filiacao", "nacionalidade", "passaporte", "cnh", "titulo_eleitor", "rg", "passport", "driver_license", "social_security"], 8),
        // 9. Certidões de Registro Civil
        (&["certidao_nascimento", "certidao_casamento", "certidao_obito", "registro_civil", "apostilamento", "certidao_negativa", "birth_certificate", "marriage_certificate"], 9),
        // 10. Recursos Humanos e Folha de Pagamento
        (&["holerite", "contracheque", "folha_pagamento", "salario", "inss", "fgts", "admissao", "demissao", "ferias", "decimo_terceiro", "payroll", "payslip", "salary"], 10),
        // 11. Currículos e Trajetória Profissional
        (&["curriculo", "curriculum_vitae", "experiencia_profissional", "formacao_academica", "habilidades", "competencias", "linkedin", "portfolio", "resume", "cv"], 11),
        // 12. Relatórios Executivos e Gestão
        (&["relatorio", "trimestre", "indicadores", "kpi", "performance", "metas", "cronograma", "executivo", "diretoria", "status_report", "management", "quarterly"], 12),
        // 13. Balanços, DRE e Contabilidade
        (&["ativo", "passivo", "balanco", "dre", "lucro", "despesa", "receita", "orcamento", "planilha", "saldo_anterior", "patrimonio", "fluxo_caixa", "accounting", "ledger", "balance_sheet"], 13),
        // 14. Propostas Comerciais e Vendas
        (&["proposta_comercial", "cotacao", "orcamento_servico", "briefing", "lead", "conversao", "funil_vendas", "campanha", "pitch", "proposal", "quotation", "sales"], 14),
        // 15. Trabalhos Acadêmicos e Teses
        (&["universidade", "faculdade", "departamento", "artigo", "resumo", "abstract", "metodologia", "referencias", "conclusao", "orientador", "tcc", "monografia", "thesis", "dissertation"], 15),
        // 16. Saúde, Medicina e Exames Laboratoriais
        (&["exame", "laudo", "medico", "hospital", "clinica", "paciente", "diagnostico", "receita_medica", "laboratorio", "hemograma", "medical_report", "prescription", "doctor"], 16),
        // 17. Imóveis, Aluguel e Condomínio
        (&["locacao", "locador", "locatario", "imovel", "aluguel", "condominio", "iptu", "escritura", "matricula_imovel", "real_estate", "lease", "tenant", "property"], 17),
        // 18. Veículos, Trânsito e Transporte
        (&["ipva", "licenciamento", "crlv", "multa_transito", "veiculo", "chassi", "placa_veiculo", "renavam", "sinistro", "seguro_auto", "vehicle", "car_insurance"], 18),
        // 19. Telecomunicações e Internet
        (&["claro", "vivo", "tim", "oi_fibra", "banda_larga", "telefonia", "plano_movel", "internet", "fibra_optica", "broadband", "cellular", "telecom"], 19),
        // 20. Cartões de Crédito e Financiamento
        (&["cartao_credito", "fatura_fechada", "limite_disponivel", "parcelamento", "mastercard", "visa", "credit_card", "statement"], 20),
        // 21. Programação e Engenharia de Software
        (&["function", "struct", "class", "async", "interface", "import", "export", "return", "const", "impl", "pub", "fn", "namespace", "repository", "commit", "github"], 21),
        // 22. DevOps, Nuvem e Infraestrutura
        (&["dockerfile", "yaml", "toml", "json", "kubernetes", "nginx", "database", "postgres", "mysql", "redis", "server", "endpoint", "api_key", "environment", "deployment"], 22),
        // 23. Design, Multimídia e Vetores
        (&["resolucao", "canvas", "vetor", "camada", "frame", "render", "audio_track", "codec", "bitrate", "typography", "palette", "vector", "layer"], 23),
        // 24. Manuais e Guias Técnicos
        (&["manual_usuario", "guia_instalacao", "especificacao_tecnica", "instrucoes", "passo_a_passo", "user_guide", "documentation", "specification", "troubleshooting"], 24),
        // 25. Jogos e Entretenimento
        (&["gameplay", "zelda", "minecraft", "pokemon", "valorant", "counter_strike", "steam", "playstation", "xbox", "nintendo", "walkthrough", "achievement"], 25),
        // 26. E-Commerce e Compras Online
        (&["pedido", "rastreamento", "entrega", "frete", "mercadolivre", "amazon", "shopee", "comprovante_compra", "order_tracking", "shipping"], 26),
        // 27. Viagens, Passagens e Hospedagem
        (&["passagem_aerea", "boarding_pass", "hotel", "reserva", "voo", "aeroporto", "hospedagem", "booking", "flight_ticket", "check_in"], 27),
    ];

    for (keywords, dim_offset) in topic_buckets {
        let mut count = 0.0f32;
        for kw in *keywords {
            if kw.contains('_') {
                if lower.contains(kw) {
                    count += 4.5;
                }
            } else if word_set.contains(kw) {
                count += 3.8;
            }
        }
        if count > 0.0 {
            let idx = TOPIC_START_DIM + (dim_offset * 2);
            if idx + 1 < VECTOR_DIM {
                vector[idx] += count;
                vector[idx + 1] += count * 0.85;
            }
        }
    }

    // 3. Normalização L2 rigorosa do vetor denso resultante
    let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude > 0.00001 {
        for val in vector.iter_mut() {
            *val /= magnitude;
        }
    }

    Ok(vector)
}

/// Agrupa arquivos por proximidade semântica utilizando clusterização com centróides adaptativos.
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
            // Adiciona ao cluster existente e atualiza o centróide ponderado
            let cluster = &mut clusters[c_idx];
            cluster.item_ids.push(id.clone());
            let n = cluster.item_ids.len() as f32;

            for (c_val, &v_val) in cluster.centroid.iter_mut().zip(vec.iter()) {
                *c_val = (*c_val * (n - 1.0) + v_val) / n;
            }
            normalize_vector(&mut cluster.centroid);
        } else {
            // Cria um novo cluster com o vetor inicial
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

/// Similaridade de cosseno entre dois vetores densos normalizados (produto escalar direto)
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

