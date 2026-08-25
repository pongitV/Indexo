use anyhow::Result;

const VECTOR_DIM: usize = 128;
const SIMILARITY_THRESHOLD: f32 = 0.65;

/// Camada 2: gera um vetor de embedding local e deterministico a partir
/// do trecho de texto extraido.
///
/// Vetoriza termos semanticos, n-gramas e frequencias de termos em um vetor
/// normalizado L2 de 128 dimensoes.
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

    // 1. Projecao semantica por n-gramas de caracteres e hashing deterministico
    for word in &words {
        let chars: Vec<char> = word.chars().collect();
        if chars.len() >= 3 {
            for window in chars.windows(3) {
                let s: String = window.iter().collect();
                let hash = hash_token(&s) % (VECTOR_DIM / 2);
                vector[hash] += 1.0;
            }
        }
        let word_hash = (hash_token(word) % (VECTOR_DIM / 2)) + (VECTOR_DIM / 2);
        vector[word_hash] += 2.0;
    }

    // 2. Projecoes de topicos semanticos fundamentais
    let topic_buckets: &[(&[&str], usize)] = &[
        // Boletos / Energia / Contas
        (&["boleto", "fatura", "consumo", "vencimento", "kwh", "enel", "luz", "energia", "sabesp", "agua", "mes_referencia", "codigo_barras", "linha_digitavel"], 0),
        // Pagamentos / Comprovantes / Bancos
        (&["comprovante", "pagamento", "transferencia", "pix", "autenticacao", "banco", "saldo", "extrato", "agencia", "conta_corrente", "liquidacao", "favorecido"], 1),
        // Notas Fiscais / Tributos
        (&["danfe", "nfe", "chave_acesso", "imposto", "icms", "iss", "tributario", "destinatario", "emitente", "valor_total", "discriminacao"], 2),
        // Contratos / Juridico
        (&["contratante", "contratada", "clausula", "foro", "obrigações", "rescisao", "testemunhas", "acordo", "vigencia", "estipulado"], 3),
        // Documentos Pessoais
        (&["cpf", "identidade", "orgao_emissor", "nascimento", "filiacao", "nacionalidade", "naturalidade", "passaporte", "cnh", "eleitor"], 4),
        // Relatorios / Trabalho / Empresas
        (&["relatorio", "trimestre", "indicadores", "kpi", "performance", "metas", "cronograma", "executivo", "diretoria", "status_report"], 5),
        // Estudos / Academico
        (&["universidade", "faculdade", "departamento", "artigo", "resumo", "abstract", "metodologia", "referencias", "conclusao", "orientador"], 6),
        // Planilhas / Balancos
        (&["ativo", "passivo", "balanco", "dre", "lucro", "despesa", "receita", "orcamento", "planilha", "total", "saldo_anterior"], 7),
    ];

    for (keywords, dim_offset) in topic_buckets {
        let mut count = 0.0f32;
        for kw in *keywords {
            if lower.contains(kw) {
                count += 3.0;
            }
        }
        if count > 0.0 {
            let idx = (VECTOR_DIM - 16) + dim_offset;
            vector[idx] += count;
        }
    }

    // 3. Normalizacao L2 do vetor resultante
    let magnitude: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude > 0.00001 {
        for val in vector.iter_mut() {
            *val /= magnitude;
        }
    }

    Ok(vector)
}

/// Agrupa arquivos por proximidade de embeddings (similaridade de cosseno)
/// com seed deterministica e estabilidade entre execucoes.
pub fn cluster_files(embeddings: &[(String, Vec<f32>)]) -> Vec<Vec<String>> {
    if embeddings.is_empty() {
        return Vec::new();
    }

    let mut visited = vec![false; embeddings.len()];
    let mut clusters: Vec<Vec<String>> = Vec::new();

    for i in 0..embeddings.len() {
        if visited[i] {
            continue;
        }

        let mut current_cluster = vec![embeddings[i].0.clone()];
        visited[i] = true;

        for j in (i + 1)..embeddings.len() {
            if visited[j] {
                continue;
            }

            let sim = cosine_similarity(&embeddings[i].1, &embeddings[j].1);
            if sim >= SIMILARITY_THRESHOLD {
                visited[j] = true;
                current_cluster.push(embeddings[j].0.clone());
            }
        }

        clusters.push(current_cluster);
    }

    clusters
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
