use std::path::Path;
use crate::commands::scan::FileMeta;
use crate::db::models::ClassificationRule;

pub struct HeuristicResult {
    pub category_guess: Option<String>,
    pub confidence: f32, // 0.0 a 1.0
    pub needs_deeper_analysis: bool,
}

const CONFIDENCE_THRESHOLD: f32 = 0.75;

/// Pastas/arquivos que NUNCA devem ser tocados por padrao (RNF05):
/// Windows/, Program Files/, Program Files (x86)/, AppData/, .git/,
/// node_modules/, e arquivos de sistema ocultos.
pub fn is_excluded_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    let lower_path = path_str.to_lowercase();

    // Pastas de sistema e diretorios protegidos no Windows
    let excluded_dirs = [
        "\\windows\\",
        "/windows/",
        "\\program files\\",
        "/program files/",
        "\\program files (x86)\\",
        "/program files (x86)/",
        "\\appdata\\",
        "/appdata/",
        "\\$recycle.bin\\",
        "/$recycle.bin/",
        "\\system volume information\\",
        "/system volume information/",
        "\\node_modules\\",
        "/node_modules/",
        "\\.git\\",
        "/.git/",
        "\\.svn\\",
        "/.svn/",
        "\\.hg\\",
        "/.hg/",
        "\\target\\debug\\",
        "\\target\\release\\",
        "/target/debug/",
        "/target/release/",
    ];

    for exc in excluded_dirs {
        if lower_path.contains(exc) {
            return true;
        }
    }

    // Checar componentes individuais do caminho
    for component in path.components() {
        let name = component.as_os_str().to_string_lossy();
        if name.starts_with('.') && name.len() > 1 && name != ".." {
            return true; // Diretorios ou arquivos ocultos como .git, .vscode, .env
        }
    }

    // Arquivos de sistema especiais
    if let Some(file_name) = path.file_name() {
        let name = file_name.to_string_lossy().to_lowercase();
        let system_files = [
            "desktop.ini",
            "thumbs.db",
            "ntuser.dat",
            "swapfile.sys",
            "pagefile.sys",
            "hiberfil.sys",
            ".ds_store",
        ];
        if system_files.contains(&name.as_str()) {
            return true;
        }
    }

    false
}

/// Camada 1 do motor de classificacao:
/// classificacao instantanea de alta precisao, sem ler o conteudo do arquivo.
pub fn classify_by_heuristics(
    file: &FileMeta,
    known_rules: &[ClassificationRule],
) -> HeuristicResult {
    let filename_lower = file.filename.to_lowercase();
    let ext_declared = file.extension_declared.as_deref().unwrap_or("").to_lowercase();
    let ext_detected = file.extension_detected.as_deref().unwrap_or("").to_lowercase();

    let effective_ext = if !ext_declared.is_empty() {
        ext_declared.as_str()
    } else {
        ext_detected.as_str()
    };

    // -------------------------------------------------------------
    // SINAL 1: Regras aprendidas (classification_rules) - Validação Estrita
    // -------------------------------------------------------------
    for rule in known_rules {
        let pat_val = rule.pattern_value.to_lowercase();
        let matches = match rule.pattern_type.as_str() {
            "extension" => ext_declared == pat_val || ext_detected == pat_val,
            "filename_regex" => {
                // Exige casamento por token de palavra completa (evita falso positivo por substring)
                contains_word_token(&filename_lower, &[&pat_val])
            }
            "content_keyword" => contains_word_token(&filename_lower, &[&pat_val]),
            _ => false,
        };

        if matches {
            // Verifica se a regra aprendida não viola o domínio óbvio de código/scripts
            let is_code_file = is_code_or_script_extension(effective_ext);
            let is_finance_rule = rule.category_id.to_lowercase().contains("boleto")
                || rule.category_id.to_lowercase().contains("fatura")
                || rule.category_id.to_lowercase().contains("recibo")
                || rule.category_id.to_lowercase().contains("comprovante");

            if !(is_code_file && is_finance_rule) {
                let conf = (rule.confidence_weight + (rule.hit_count as f32 * 0.05)).clamp(0.80, 0.99);
                return HeuristicResult {
                    category_guess: Some(rule.category_id.clone()),
                    confidence: conf,
                    needs_deeper_analysis: false,
                };
            }
        }
    }

    // -------------------------------------------------------------
    // SINAL 2: Analise de nome legivel e palavras-chave NLP
    // -------------------------------------------------------------
    let (nlp_category, mut score, is_generic_name) = analyze_filename_nlp(&file.filename, &ext_declared, &ext_detected);

    // -------------------------------------------------------------
    // SINAL 3: Extensao detectada vs declarada (infer magic bytes)
    // -------------------------------------------------------------
    let has_extension_mismatch = !ext_detected.is_empty()
        && !ext_declared.is_empty()
        && !extensions_are_compatible(&ext_declared, &ext_detected);

    if has_extension_mismatch {
        score -= 0.35; // Penalidade por extensao falsa/corrompida
    }

    // -------------------------------------------------------------
    // SINAL 4: Contexto da pasta-pai
    // -------------------------------------------------------------
    if let Some(parent_hint) = get_parent_directory_hint(&file.path) {
        if let Some(ref cat) = nlp_category {
            if cat.to_lowercase().contains(&parent_hint) || parent_hint.contains(&cat.to_lowercase()) {
                score += 0.15;
            }
        }
    }

    let final_confidence = score.clamp(0.0, 0.95);

    // Documentos textuais e imagens com nomes genéricos requerem análise semântica de conteúdo/OCR (Camada 2).
    let is_textual_doc = ["pdf", "docx", "doc", "odt", "rtf", "txt", "csv", "tsv", "xlsx", "xls", "ods"]
        .contains(&effective_ext);

    let is_ocr_candidate_image = crate::engine::ocr::is_ocr_supported_extension(effective_ext) && is_generic_name;

    let is_resolved = if is_textual_doc {
        final_confidence >= CONFIDENCE_THRESHOLD && !is_generic_name && !has_extension_mismatch
    } else if is_ocr_candidate_image {
        // Imagens com nomes genéricos passam pelo OCR na Camada 2
        false
    } else {
        nlp_category.is_some() && !has_extension_mismatch && final_confidence >= 0.60
    };

    HeuristicResult {
        category_guess: nlp_category,
        confidence: final_confidence,
        needs_deeper_analysis: !is_resolved,
    }
}

/// Verifica se a extensão pertence estritamente ao domínio de código e desenvolvimento
pub fn is_code_or_script_extension(ext: &str) -> bool {
    let code_exts = [
        "bat", "cmd", "sh", "ps1", "rs", "ts", "js", "py", "c", "cpp", "h", "hpp",
        "java", "go", "php", "sql", "html", "css", "scss", "json", "yaml", "yml",
        "toml", "xml", "dockerfile", "vue", "svelte", "swift", "kt", "cs", "lua",
        "rb", "dart", "wasm", "vbs", "asm", "hex", "ini", "cfg", "conf", "env",
    ];
    code_exts.contains(&ext)
}

/// Analisa o nome do arquivo com tokenização estrita por limites de palavra e matriz de extensões
fn analyze_filename_nlp(filename: &str, ext_declared: &str, ext_detected: &str) -> (Option<String>, f32, bool) {
    let lower = filename.to_lowercase();
    let effective_ext = if !ext_declared.is_empty() {
        ext_declared
    } else {
        ext_detected
    };

    let is_generic = is_generic_or_random_name(filename);

    // =========================================================================
    // DOMÍNIO 1: Código, Scripts e Desenvolvimento (Extensões Protegidas)
    // NUNCA podem ser classificados como Boletos, Faturas, Fotos ou Mídia
    // =========================================================================
    if is_code_or_script_extension(effective_ext) {
        return (Some("Código e Desenvolvimento".to_string()), 0.90, is_generic);
    }

    // =========================================================================
    // DOMÍNIO 2: Documentação Técnica e Markdown (.md, .rst, .tex)
    // =========================================================================
    let markdown_exts = ["md", "markdown", "rst", "adoc", "tex", "log"];
    if markdown_exts.contains(&effective_ext) {
        return (Some("Documentos e Documentação".to_string()), 0.85, is_generic);
    }

    // =========================================================================
    // DOMÍNIO 3: Mídias Nativas (Extensões com Tipologia Inegociável)
    // =========================================================================
    let image_exts = ["jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "tiff", "ico", "raw", "heic", "psd", "ai", "eps", "xcf", "cr2", "nef", "arw"];
    if image_exts.contains(&effective_ext) {
        let conf = if is_generic { 0.65 } else { 0.85 };
        return (Some("Fotos e Imagens".to_string()), conf, is_generic);
    }

    let video_exts = ["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "3gp", "ts", "mpg", "mpeg"];
    if video_exts.contains(&effective_ext) {
        let conf = if is_generic { 0.70 } else { 0.85 };
        return (Some("Vídeos".to_string()), conf, is_generic);
    }

    let audio_exts = ["mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "mid", "midi", "opus", "aiff", "alac"];
    if audio_exts.contains(&effective_ext) {
        return (Some("Áudios e Músicas".to_string()), 0.85, is_generic);
    }

    let archive_exts = ["zip", "rar", "7z", "tar", "gz", "bz2", "xz", "iso", "tgz", "cab", "lz", "zst"];
    if archive_exts.contains(&effective_ext) {
        return (Some("Arquivos Compactados".to_string()), 0.88, is_generic);
    }

    let installer_exts = ["exe", "msi", "dmg", "pkg", "appimage", "deb", "rpm", "apk", "jar", "dll", "so", "dylib", "sys"];
    if installer_exts.contains(&effective_ext) {
        return (Some("Instaladores e Programas".to_string()), 0.90, is_generic);
    }

    let ebook_exts = ["epub", "mobi", "azw3", "cbr", "cbz", "fb2"];
    if ebook_exts.contains(&effective_ext) {
        return (Some("Livros e E-books".to_string()), 0.88, is_generic);
    }

    let sheet_exts = ["xlsx", "xls", "csv", "ods", "tsv", "xlsm", "numbers"];
    if sheet_exts.contains(&effective_ext) {
        let conf = if is_generic { 0.60 } else { 0.80 };
        return (Some("Planilhas e Dados".to_string()), conf, is_generic);
    }

    // =========================================================================
    // DOMÍNIO 4: Dicionários Semânticos por Palavras-Chave (Apenas Docs/PDFs)
    // =========================================================================

    // 1. Financeiro / Boletos / Faturas (Tokens Exatos e Expressões Compostas)
    let finance_keywords = [
        "boleto", "fatura", "conta_de_luz", "conta_luz", "conta_de_agua", "conta_agua",
        "energia", "enel", "cemig", "copel", "cpfl", "sabesp", "sanepar", "embasa",
        "claro", "vivo", "tim", "oi_fibra", "internet", "iptu", "ipva", "darf", "das_mei",
        "tributo", "fatura_cartao", "fatura_nubank", "fatura_itau", "fatura_bradesco",
        "invoice", "electric_bill", "utility_bill", "phone_bill",
    ];
    if contains_word_token(&lower, &finance_keywords) {
        return (Some("Boletos e Faturas".to_string()), 0.88, is_generic);
    }

    // 2. Comprovantes e Recibos
    let receipt_keywords = [
        "comprovante", "recibo", "pagamento", "transferencia", "pix", "ted", "doc_bancario",
        "extrato", "extrato_bancario", "holerite", "contracheque", "receipt", "payment_receipt",
        "voucher", "statement", "bank_statement",
    ];
    if contains_word_token(&lower, &receipt_keywords) {
        return (Some("Comprovantes e Recibos".to_string()), 0.86, is_generic);
    }

    // 3. Notas Fiscais
    let tax_doc_keywords = [
        "danfe", "nfe", "nf-e", "nfse", "nfs-e", "nota_fiscal", "notafiscal", "cupom_fiscal", "xml_nfe",
    ];
    if contains_word_token(&lower, &tax_doc_keywords) {
        return (Some("Notas Fiscais".to_string()), 0.90, is_generic);
    }

    // 4. Contratos e Documentos Legais
    let legal_keywords = [
        "contrato", "acordo", "termo_de_adesao", "termo_de_uso", "procuracao", "declaracao",
        "estatuto", "certidao", "aditivo", "notificacao", "juridico", "contract", "agreement",
        "nda", "affidavit", "power_of_attorney", "deed", "termo_de_rescisao",
    ];
    if contains_word_token(&lower, &legal_keywords) {
        return (Some("Contratos e Jurídico".to_string()), 0.85, is_generic);
    }

    // 5. Documentos Pessoais e Identidade
    let personal_doc_keywords = [
        "rg", "cpf", "cnh", "passaporte", "identidade", "carteira_de_trabalho", "titulo_eleitor",
        "certidao_nascimento", "certidao_casamento", "curriculo", "curriculum", "resume", "cv",
        "passport", "driver_license",
    ];
    if contains_word_token(&lower, &personal_doc_keywords) {
        return (Some("Documentos Pessoais".to_string()), 0.86, is_generic);
    }

    // 6. Relatorios e Projetos de Trabalho
    let work_keywords = [
        "relatorio", "apresentacao", "slides", "ata_de_reuniao", "projeto", "orcamento",
        "proposta_comercial", "briefing", "cronograma", "planejamento", "report", "presentation",
        "proposal", "budget", "project_plan", "meeting_minutes",
    ];
    if contains_word_token(&lower, &work_keywords) {
        return (Some("Relatórios e Projetos".to_string()), 0.82, is_generic);
    }

    // 7. Estudos e Academico
    let study_keywords = [
        "tcc", "artigo_cientifico", "dissertacao", "tese", "monografia", "apostila",
        "resumo_aula", "prova", "gabarito", "exercicio", "syllabus", "thesis", "essay",
        "academic_paper",
    ];
    if contains_word_token(&lower, &study_keywords) {
        return (Some("Estudos e Acadêmico".to_string()), 0.82, is_generic);
    }

    // 8. Documentos de Texto / PDF sem palavra-chave específica
    let doc_exts = ["pdf", "docx", "doc", "odt", "rtf", "txt"];
    if doc_exts.contains(&effective_ext) {
        return (Some("Documentos".to_string()), 0.50, is_generic);
    }

    (None, 0.20, is_generic)
}

/// Verifica se o nome do arquivo e puramente numerico, timestamp ou aleatorio
fn is_generic_or_random_name(filename: &str) -> bool {
    let name_without_ext = Path::new(filename)
        .file_stem()
        .map(|s| s.to_string_lossy())
        .unwrap_or_default();

    let name = name_without_ext.trim();
    if name.is_empty() {
        return true;
    }

    // Se e puramente numerico (ex: 8291, 1234567)
    if name.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }

    let lower = name.to_lowercase();

    // Padroes de camera / scanner / download generico
    let generic_prefixes = [
        "img_", "dsc_", "photo_", "screenshot_", "captura_", "scanner_", "scan_",
        "doc_", "documento", "arquivo", "file_", "download", "untitled", "sem_titulo",
        "nova_pasta", "copia", "copy",
    ];

    for prefix in generic_prefixes {
        if lower.starts_with(prefix) {
            let rest = &lower[prefix.len()..];
            // Se o restante for numeros, data ou caracteres curtos -> generico
            if rest.chars().all(|c| c.is_ascii_digit() || c == '_' || c == '-' || c == '(' || c == ')' || c == ' ') {
                return true;
            }
        }
    }

    // Padrao de copia automatica: "algo (1)", "algo(2)" com nome base curto
    if name.len() < 8 && name.contains('(') && name.ends_with(')') {
        return true;
    }

    // Hashes hexadecimais longos (MD5, SHA1, SHA256, UUID)
    if (name.len() == 32 || name.len() == 36 || name.len() == 40 || name.len() == 64)
        && name.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
    {
        return true;
    }

    false
}

/// Verifica compatibilidade de extensoes
fn extensions_are_compatible(declared: &str, detected: &str) -> bool {
    if declared.eq_ignore_ascii_case(detected) {
        return true;
    }

    // Sinonimos comuns
    let aliases = [
        ("jpg", "jpeg"),
        ("jpeg", "jpg"),
        ("tif", "tiff"),
        ("tiff", "tif"),
        ("htm", "html"),
        ("html", "htm"),
        ("yaml", "yml"),
        ("yml", "yaml"),
        ("docx", "zip"), // docx/xlsx sao zips
        ("xlsx", "zip"),
        ("pptx", "zip"),
        ("jar", "zip"),
    ];

    for (d, det) in aliases {
        if declared.eq_ignore_ascii_case(d) && detected.eq_ignore_ascii_case(det) {
            return true;
        }
    }

    false
}

/// Extrai possivel dica contextual da pasta pai
fn get_parent_directory_hint(file_path: &str) -> Option<String> {
    let path = Path::new(file_path);
    let parent = path.parent()?;
    let parent_name = parent.file_name()?.to_string_lossy().to_lowercase();

    // Ignorar pastas genericas
    let generic_parents = ["downloads", "desktop", "documents", "documentos", "temp", "tmp", "files", "arquivos"];
    if generic_parents.contains(&parent_name.as_str()) || parent_name.len() < 3 {
        None
    } else {
        Some(parent_name)
    }
}

/// Detecta se um arquivo já está situado em uma subpasta organizada/estruturada em relação à raiz escaneada.
/// Retorna Some(relative_folder_path) se a subpasta possui coerência estrutural e semântica real.
///
/// **PRIMEIRO ATIVO DO SISTEMA**:
/// - Preserva projetos de código (com package.json, Cargo.toml, Makefile, .csproj, etc.).
/// - Preserva pastas de jogos e softwares (com .exe, .dll, assets/, data/, bin/, config.ini, etc.).
/// - Preserva subpastas com prefixos compartilhados ou hierarquias multiníveis.
/// - Apenas pastas de despejo/descarte (downloads, temp, desktop, nova pasta, unsorted, bagunça)
///   e arquivos soltos diretamente na raiz são elegíveis para reorganização.
pub fn detect_already_organized_folder(file_path: &str, root_path: &str) -> Option<String> {
    let p_file = Path::new(file_path);
    let p_root = Path::new(root_path);

    let parent = p_file.parent()?;
    if parent == p_root {
        // Arquivo está solto diretamente na raiz escaneada -> precisa de organização
        return None;
    }

    let relative = match parent.strip_prefix(p_root) {
        Ok(r) => r,
        Err(_) => return None,
    };

    let rel_str = relative.to_string_lossy().replace('\\', "/");
    let segments: Vec<&str> = rel_str
        .split('/')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if segments.is_empty() {
        return None;
    }

    // Pastas de descarte, lixeira ou nomes provisórios que NUNCA devem ser consideradas organizadas
    let generic_dump_folders = [
        "downloads", "download", "desktop", "temp", "tmp", "files", "arquivos", "misc",
        "nova pasta", "nova_pasta", "novapasta", "new folder", "new_folder", "newfolder",
        "outros", "unorganized", "variados", "pasta", "folder", "bagunca", "bagunça",
        "lixeira", "lixo", "descarte", "despejo", "organizar", "para organizar", "para_organizar",
        "to_sort", "unsorted", "loose", "scratch", "test", "teste", "todos", "all",
    ];

    // Se o primeiro segmento da pasta relativa for uma pasta de despejo/descarte conhecida, não é organizada
    let top_segment_lower = segments[0].to_lowercase();
    if generic_dump_folders.contains(&top_segment_lower.as_str()) {
        return None;
    }

    // Verifica padrões como "Nova Pasta (1)", "New Folder 2", "Pasta 3"
    if (top_segment_lower.starts_with("nova pasta")
        || top_segment_lower.starts_with("new folder")
        || top_segment_lower.starts_with("pasta "))
        && top_segment_lower.len() < 20
    {
        return None;
    }

    // Se for um hash hexadecimal de pasta temporária (ex: 32 chars)
    if (top_segment_lower.len() == 32 || top_segment_lower.len() == 36)
        && top_segment_lower.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
    {
        return None;
    }

    Some(segments.join("/"))
}

/// Avalia o score de coerência semântica e tipológica de um grupo de arquivos em uma pasta (0.0 a 1.0)
///
/// Reconhece com alta prioridade:
/// 1. Projetos de software / código (presença de manifestos de projeto).
/// 2. Jogos / aplicações / executáveis com bibliotecas (.exe + .dll / .pak / assets).
/// 3. Prefixo de nome compartilhado entre a pasta e os arquivos.
/// 4. Categorias majoritárias homogêneas.
pub fn calculate_folder_coherence(files: &[&FileMeta], known_rules: &[ClassificationRule]) -> f32 {
    if files.is_empty() {
        return 0.0;
    }
    if files.len() == 1 {
        return 0.85; // Arquivo único já em pasta estruturada
    }

    // 1. Verificação de Projeto de Código ou Software
    let project_marker_files = [
        "package.json", "cargo.toml", "cargo.lock", "makefile", "cmakelists.txt",
        "requirements.txt", "pyproject.toml", "setup.py", "go.mod", "dockerfile",
        "docker-compose.yml", "tsconfig.json", "pom.xml", "build.gradle",
        "solution.sln", ".csproj", ".vcxproj", "readme.md",
    ];

    let has_project_marker = files.iter().any(|f| {
        let name_lower = f.filename.to_lowercase();
        project_marker_files.contains(&name_lower.as_str()) || name_lower.ends_with(".sln") || name_lower.ends_with(".csproj")
    });

    if has_project_marker {
        return 0.98; // É claramente um projeto organizado, manter 100% intacto
    }

    // 2. Verificação de Pasta de Jogo ou Aplicação
    let has_executable = files.iter().any(|f| {
        let ext = f.extension_declared.as_deref().unwrap_or("").to_lowercase();
        ext == "exe" || ext == "msi" || ext == "appimage"
    });
    let has_app_support = files.iter().any(|f| {
        let ext = f.extension_declared.as_deref().unwrap_or("").to_lowercase();
        ["dll", "pak", "dat", "cfg", "ini", "bin", "so", "pck", "rpf"].contains(&ext.as_str())
    });

    if has_executable && has_app_support {
        return 0.98; // É um jogo ou suíte de software, manter 100% intacto
    }

    // 3. Verificação de Prefixo Compartilhado (Nome da pasta presente nos arquivos)
    if let Some(first) = files.first() {
        if let Some(parent_hint) = get_parent_directory_hint(&first.path) {
            let parent_tokens: Vec<&str> = parent_hint
                .split(|c: char| !c.is_alphanumeric())
                .filter(|t| t.len() >= 3)
                .collect();

            if !parent_tokens.is_empty() {
                let matching_files = files.iter().filter(|f| {
                    let f_lower = f.filename.to_lowercase();
                    parent_tokens.iter().any(|&token| f_lower.contains(token))
                }).count();

                let prefix_ratio = matching_files as f32 / files.len() as f32;
                if prefix_ratio >= 0.40 {
                    return (0.75 + (prefix_ratio * 0.20)).clamp(0.0, 1.0);
                }
            }
        }
    }

    // 4. Coerência por Categorias Heurísticas
    let mut category_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut total_classified = 0;

    for f in files {
        let res = classify_by_heuristics(f, known_rules);
        if let Some(cat) = res.category_guess {
            *category_counts.entry(cat).or_insert(0) += 1;
            total_classified += 1;
        }
    }

    if total_classified == 0 {
        return 0.50;
    }

    let max_count = category_counts.values().copied().max().unwrap_or(0);
    let majority_ratio = max_count as f32 / files.len() as f32;

    // Se os arquivos forem homogêneos, alta coerência
    majority_ratio.clamp(0.0, 1.0)
}

/// Tokeniza o texto e verifica se algum dos keywords corresponde a um token de palavra completo.
///
/// Resolve o problema clássico de falsos positivos como:
/// - "declaracao_escopo.md" NÃO dá match em "claro" (operadora de telecom)
/// - "optimizer_runtime.bat" NÃO dá match em "tim" (operadora de telecom)
/// - "survivor_game.bat" NÃO dá match em "vivo" (operadora de telecom)
/// - "united_script.bat" NÃO dá match em "ted" (termo bancário)
/// - "standard_agenda.md" NÃO dá match em "nda" (contrato)
pub fn contains_word_token(text: &str, keywords: &[&str]) -> bool {
    let lower_text = text.to_lowercase();
    
    // Normaliza separadores comuns para sublinhados
    let normalized_text = lower_text.replace(['-', '.', '(', ')', '[', ']', '{', '}', ',', ';', ':', '+', '=', '~', '!', '@', '#', '$', '%', '^', '&', '*', '\'', '"', '/', '\\'], "_");
    
    // Extrai tokens de palavras completas
    let tokens: Vec<&str> = normalized_text
        .split('_')
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .collect();

    for &kw in keywords {
        let kw_lower = kw.to_lowercase();
        
        if kw_lower.contains('_') || kw_lower.contains(' ') {
            // É uma expressão composta (ex: "conta_de_luz", "nota_fiscal", "fatura_cartao")
            let kw_norm = kw_lower.replace(['-', ' '], "_");
            // Verifica se a expressão aparece com limites de palavra
            if normalized_text == kw_norm
                || normalized_text.starts_with(&format!("{}_", kw_norm))
                || normalized_text.ends_with(&format!("_{}", kw_norm))
                || normalized_text.contains(&format!("_{}_", kw_norm))
            {
                return true;
            }
        } else {
            // É uma palavra simples (ex: "tim", "vivo", "claro", "danfe", "boleto")
            if tokens.contains(&kw_lower.as_str()) {
                return true;
            }
        }
    }

    false
}
