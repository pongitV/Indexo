import re
from pathlib import Path
from typing import Dict, List, Any, Optional, Set, Tuple
from collections import Counter, defaultdict
from loguru import logger
from app.i18n.language_manager import tr, LanguageManager

# Common stopwords to ignore when discovering name tokens
STOPWORDS = {
    "de", "do", "da", "dos", "das", "em", "no", "na", "nos", "nas",
    "por", "para", "com", "sem", "e", "ou", "a", "o", "as", "os",
    "um", "uma", "uns", "umas", "the", "a", "an", "and", "or", "of",
    "for", "with", "without", "in", "on", "at", "to", "by", "from",
    "copy", "copia", "copie", "final", "novo", "new", "v1", "v2", "v3",
    "scan", "img", "doc", "file", "arquivo", "temp", "tmp", "backup",
    "null", "undefined", "untitled", "sem_titulo", "teste", "test",
    "empresa", "pasta", "diretorio", "item", "dado", "dados", "geral"
}

# Generic root words that often represent categories when appearing as prefixes
ROOT_CATEGORY_MAP = {
    "fatura": "Faturas",
    "faturas": "Faturas",
    "boleto": "Boletos",
    "boletos": "Boletos",
    "conta": "Contas",
    "contas": "Contas",
    "comprovante": "Comprovantes",
    "comprovantes": "Comprovantes",
    "extrato": "Extratos",
    "extratos": "Extratos",
    "contrato": "Contratos",
    "contratos": "Contratos",
    "relatorio": "Relatórios",
    "relatorios": "Relatórios",
    "recibo": "Recibos",
    "recibos": "Recibos",
    "documento": "Documentos",
    "documentos": "Documentos",
    "imposto": "Impostos",
    "impostos": "Impostos",
    "declaracao": "Declarações",
    "declaracoes": "Declarações",
    "holerite": "Holerites e Renda",
    "holerites": "Holerites e Renda",
    "projeto": "Projetos",
    "projetos": "Projetos",
    "project": "Projects",
    "projects": "Projects",
    "game": "Jogos",
    "games": "Jogos",
    "jogo": "Jogos",
    "jogos": "Jogos",
    "viagem": "Viagens",
    "viagens": "Viagens",
    "foto": "Fotos",
    "fotos": "Fotos",
    "musica": "Músicas",
    "musicas": "Músicas",
    "video": "Vídeos",
    "videos": "Vídeos",
    "curso": "Cursos e Aulas",
    "cursos": "Cursos e Aulas",
    "aula": "Cursos e Aulas",
    "aulas": "Cursos e Aulas",
    "trabalho": "Trabalho",
    "pessoal": "Pessoal",
    "clientes": "Clientes",
    "cliente": "Clientes"
}

def clean_token(token: str) -> str:
    """Cleans a single token removing dates, numbers, underscores and non-alphanumerics."""
    t = re.sub(r"[^\w\s-]", "", token).strip().lower()
    t = re.sub(r"^\d+$", "", t)
    return t

def slugify(text: str) -> str:
    """Converts a text string to a clean slug."""
    slug = re.sub(r"[^\w\s-]", "", text).strip().lower()
    slug = re.sub(r"[\s_-]+", "_", slug)
    return slug or "tag"

def format_title(text: str) -> str:
    """Formats text into clean Title Case."""
    words = [w for w in re.split(r"[_\-\s\.]+", text) if w and not w.isdigit()]
    return " ".join(w.capitalize() for w in words) if words else text.strip()


class TagDiscoveryEngine:
    """
    Intelligent dynamic category and tag synthesis engine.
    Discovers, learns, and groups Categories and Tags automatically from:
    1. Folder Depth & Topology (Parent Folder -> Category, Subfolder -> Tag)
    2. Shared Prefix & Root Word Clustering (e.g. Fatura_Enel, Fatura_Sabesp -> Category: Faturas, Tags: Enel, Sabesp)
    3. Contextual Text & Entity Extraction (e.g. CPFL, Banco Inter, Prefeitura)
    4. Adaptive Media / Format grouping
    """

    def __init__(self):
        pass

    def discover_tags(
        self,
        root_dir: Path,
        entries: List[Dict[str, Any]],
        existing_tags: Optional[List[Dict[str, Any]]] = None
    ) -> List[Dict[str, Any]]:
        """
        Discovers new categories and tags dynamically from scan results.
        Returns a list of newly synthesized tag definitions ready for persistence.
        """
        if not entries:
            return []

        existing_tag_names = {t.get("nome", "").lower() for t in (existing_tags or []) if t.get("nome")}
        
        discovered_tags: Dict[str, Dict[str, Any]] = {}
        lang = LanguageManager.get_instance().current_language

        # 1. HIERARCHY STEP 1: DISCOVERY FROM FOLDER TOPOLOGY (Depth 2: Category/Tag, Depth 1: Folder Tag)
        self._discover_from_folder_topology(root_dir, entries, discovered_tags, existing_tag_names, lang)

        # 2. HIERARCHY STEP 1 (CONT.): DISCOVERY FROM PREFIX / ROOT WORD CLUSTERS & RECURRING TOKENS
        self._discover_from_recurring_patterns(entries, discovered_tags, existing_tag_names, lang)

        # 3. HIERARCHY STEP 2: DISCOVERY FROM TEXT CONTENT ENTITIES
        self._discover_from_text_content(entries, discovered_tags, existing_tag_names, lang)

        result = list(discovered_tags.values())
        logger.info("Discovered {} new dynamic tags and categories from scan of {}", len(result), root_dir)
        return result

    def _discover_from_folder_topology(
        self,
        root_dir: Path,
        entries: List[Dict[str, Any]],
        discovered_tags: Dict[str, Dict[str, Any]],
        existing_tag_names: Set[str],
        lang: str
    ):
        """
        Learns categories and tags directly from directory structure:
        - ParentFolder / SubFolder / File -> Category: ParentFolder, Tag: SubFolder
        - ParentFolder / File -> Category derived from folder or format, Tag: ParentFolder
        """
        folder_tree: Dict[str, Dict[str, List[Dict[str, Any]]]] = defaultdict(lambda: defaultdict(list))
        direct_folder_files: Dict[str, List[Dict[str, Any]]] = defaultdict(list)

        for entry in entries:
            rel = entry.get("rel_path", "").replace("\\", "/")
            parts = rel.split("/")
            if len(parts) >= 3:
                parent_folder = parts[0]
                sub_folder = parts[1]
                folder_tree[parent_folder][sub_folder].append(entry)
            elif len(parts) == 2:
                folder_name = parts[0]
                direct_folder_files[folder_name].append(entry)

        # Case A: Nested Structure (Depth >= 2) -> Parent is Category, Sub is Tag
        for parent_folder, subs in folder_tree.items():
            cat_name = format_title(parent_folder)
            for sub_folder, f_list in subs.items():
                tag_name = format_title(sub_folder)
                if not tag_name or tag_name.lower() in existing_tag_names:
                    continue

                tag_id = f"auto_topol_{slugify(cat_name)}_{slugify(tag_name)}"
                file_types = [f.get("file_type", "other") for f in f_list]
                exts = list({Path(f.get("rel_path", "")).suffix.lower() for f in f_list if Path(f.get("rel_path", "")).suffix})
                keywords = [clean_token(w) for w in (tag_name).split() if len(clean_token(w)) > 2 and clean_token(w) not in STOPWORDS]

                cat_slug = slugify(cat_name)
                tag_slug = slugify(tag_name)

                tag_def = {
                    "id": tag_id,
                    "nome": tag_name,
                    "categoria": cat_name,
                    "categoria_key": f"custom.{cat_slug}",
                    "subcategoria": tag_name,
                    "entidade": tag_name,
                    "caminho_fisico": f"{cat_slug}/{tag_slug}",
                    "origem": "user",
                    "idioma": lang,
                    "sinonimos": [sub_folder.lower()],
                    "palavras_chave": keywords if keywords else [tag_name.lower()],
                    "regex": [rf"\b{re.escape(sub_folder.replace('_', ' '))}\b"],
                    "extensoes": exts,
                    "confianca_base": 0.85,
                    "usar_para_automacao": True,
                    "version": 1
                }
                discovered_tags[tag_name.lower()] = tag_def

        # Case B: Single Folder Depth -> Learn category and tag dynamically
        for folder_name, f_list in direct_folder_files.items():
            clean_name = folder_name.replace("_", " ").replace("-", " ").strip()
            if not clean_name or len(clean_name) < 2:
                continue

            tag_name = format_title(clean_name)
            if tag_name.lower() in existing_tag_names or tag_name.lower() in discovered_tags:
                continue

            file_types = [f.get("file_type", "other") for f in f_list]
            exts = list({Path(f.get("rel_path", "")).suffix.lower() for f in f_list if Path(f.get("rel_path", "")).suffix})
            keywords = [clean_token(w) for w in clean_name.split() if len(clean_token(w)) > 2 and clean_token(w) not in STOPWORDS]

            first_word = clean_token(clean_name.split()[0])
            if first_word in ROOT_CATEGORY_MAP and len(clean_name.split()) > 1:
                category_name = ROOT_CATEGORY_MAP[first_word]
                tag_name = format_title(clean_name)
            elif first_word in ROOT_CATEGORY_MAP and len(clean_name.split()) == 1:
                category_name = ROOT_CATEGORY_MAP[first_word]
                tag_name = category_name
            else:
                category_name = self._infer_dynamic_category(clean_name, keywords, file_types, lang)

            cat_slug = slugify(category_name)
            tag_slug = slugify(tag_name)

            tag_def = {
                "id": f"auto_folder_{slugify(tag_name)}",
                "nome": tag_name,
                "categoria": category_name,
                "categoria_key": f"custom.{cat_slug}",
                "subcategoria": tag_name,
                "entidade": tag_name,
                "caminho_fisico": f"{cat_slug}/{tag_slug}",
                "origem": "user",
                "idioma": lang,
                "sinonimos": [clean_name.lower()],
                "palavras_chave": keywords if keywords else [tag_name.lower()],
                "regex": [rf"\b{re.escape(clean_name)}\b"],
                "extensoes": exts,
                "confianca_base": 0.85,
                "usar_para_automacao": True,
                "version": 1
            }
            discovered_tags[tag_name.lower()] = tag_def

    def _discover_from_recurring_patterns(
        self,
        entries: List[Dict[str, Any]],
        discovered_tags: Dict[str, Dict[str, Any]],
        existing_tag_names: Set[str],
        lang: str
    ):
        """
        Discovers recurring prefix clusters and standalone tokens:
        - Fatura_Enel, Fatura_Sabesp -> Category: Faturas, Tags: Enel, Sabesp
        - Holerite_2024_01, Holerite_2024_02 -> Category: Holerites e Renda, Tag: Holerite
        - Relatorio_Vendas, Relatorio_Custos -> Category: Relatórios, Tags: Vendas, Custos
        """
        prefix_groups: Dict[str, Dict[str, List[Dict[str, Any]]]] = defaultdict(lambda: defaultdict(list))
        token_occurrences: Dict[str, List[Dict[str, Any]]] = defaultdict(list)

        for entry in entries:
            stem = Path(entry.get("rel_path", "")).stem
            tokens = [t for t in re.split(r"[_\-\s\.]+", stem) if t]
            clean_tokens = [clean_token(t) for t in tokens if clean_token(t) and clean_token(t) not in STOPWORDS]

            # Track individual recurring tokens
            for ct in set(clean_tokens):
                if len(ct) >= 3:
                    token_occurrences[ct].append(entry)

            # Track prefix + suffix
            if len(clean_tokens) >= 1:
                prefix = clean_tokens[0]
                if len(prefix) >= 3 and prefix not in STOPWORDS:
                    suffix = " ".join(clean_tokens[1:]) if len(clean_tokens) > 1 else ""
                    prefix_groups[prefix][suffix].append(entry)

        # 1. Prefix clusters
        for prefix, suffix_map in prefix_groups.items():
            total_files = sum(len(fl) for fl in suffix_map.values())
            if total_files >= 2:
                cat_name = ROOT_CATEGORY_MAP.get(prefix, format_title(prefix))

                for suffix, f_list in suffix_map.items():
                    # If suffix is empty (meaning all files just share the prefix token, e.g. Holerite_01, Holerite_02)
                    if not suffix:
                        tag_name = format_title(prefix)
                    else:
                        tag_name = format_title(suffix)

                    if tag_name.lower() in existing_tag_names or tag_name.lower() in discovered_tags:
                        continue

                    tag_id = f"auto_cluster_{slugify(cat_name)}_{slugify(tag_name)}"
                    file_types = [f.get("file_type", "other") for f in f_list]
                    exts = list({Path(f.get("rel_path", "")).suffix.lower() for f in f_list if Path(f.get("rel_path", "")).suffix})
                    keywords = [clean_token(w) for w in f"{prefix} {suffix}".split() if clean_token(w) and clean_token(w) not in STOPWORDS]

                    cat_slug = slugify(cat_name)
                    tag_slug = slugify(tag_name)

                    tag_def = {
                        "id": tag_id,
                        "nome": tag_name,
                        "categoria": cat_name,
                        "categoria_key": f"custom.{cat_slug}",
                        "subcategoria": tag_name,
                        "entidade": tag_name,
                        "caminho_fisico": f"{cat_slug}/{tag_slug}",
                        "origem": "user",
                        "idioma": lang,
                        "sinonimos": [f"{prefix} {suffix}".strip().lower()],
                        "palavras_chave": keywords,
                        "regex": [rf"\b{re.escape(tag_name.lower())}\b", rf"\b{re.escape(prefix)}\b"],
                        "extensoes": exts,
                        "confianca_base": 0.85,
                        "usar_para_automacao": True,
                        "version": 1
                    }
                    discovered_tags[tag_name.lower()] = tag_def

        # 2. Standalone recurring tokens (>= 2 files)
        for token, f_list in token_occurrences.items():
            if len(f_list) >= 2:
                tag_name = format_title(token)
                if tag_name.lower() in existing_tag_names or tag_name.lower() in discovered_tags:
                    continue

                category_name = ROOT_CATEGORY_MAP.get(token, self._infer_dynamic_category(token, [token], [f.get("file_type", "other") for f in f_list], lang))
                cat_slug = slugify(category_name)
                tag_slug = slugify(tag_name)
                exts = list({Path(f.get("rel_path", "")).suffix.lower() for f in f_list if Path(f.get("rel_path", "")).suffix})

                tag_def = {
                    "id": f"auto_token_{tag_slug}",
                    "nome": tag_name,
                    "categoria": category_name,
                    "categoria_key": f"custom.{cat_slug}",
                    "subcategoria": tag_name,
                    "entidade": tag_name,
                    "caminho_fisico": f"{cat_slug}/{tag_slug}",
                    "origem": "user",
                    "idioma": lang,
                    "sinonimos": [token],
                    "palavras_chave": [token],
                    "regex": [rf"\b{re.escape(token)}\b"],
                    "extensoes": exts,
                    "confianca_base": 0.85,
                    "usar_para_automacao": True,
                    "version": 1
                }
                discovered_tags[tag_name.lower()] = tag_def

    def _discover_from_text_content(
        self,
        entries: List[Dict[str, Any]],
        discovered_tags: Dict[str, Dict[str, Any]],
        existing_tag_names: Set[str],
        lang: str
    ):
        """Extracts repeated entity patterns from extracted text content."""
        entity_to_files: Dict[str, List[Dict[str, Any]]] = defaultdict(list)

        ENTITY_PATTERNS = [
            (r"\b(nubank|banco inter|itau|itaú|bradesco|santander|caixa economica|banco do brasil)\b", "Bancos e Finanças"),
            (r"\b(cpfl|enel|sabesp|comgas|comgás|copel|cemig|light|neoenergia)\b", "Faturas e Serviços"),
            (r"\b(claro|vivo|tim|oi|net)\b", "Telecomunicações"),
            (r"\b(prefeitura de [\w\s]+|receita federal|governo do estado)\b", "Governo e Tributos"),
            (r"\b(universidade [\w\s]+|faculdade [\w\s]+)\b", "Educação")
        ]

        for entry in entries:
            text = entry.get("extracted_text", "")
            if not text:
                continue

            text_lower = text.lower()
            for pattern, default_cat in ENTITY_PATTERNS:
                matches = re.findall(pattern, text_lower)
                for m in matches:
                    ent_clean = m.strip()
                    entity_to_files[(ent_clean, default_cat)].append(entry)

        for (ent_name, default_cat), f_list in entity_to_files.items():
            if len(f_list) >= 1:
                tag_name = format_title(ent_name)
                if tag_name.lower() in existing_tag_names or tag_name.lower() in discovered_tags:
                    continue

                category_name = default_cat
                cat_slug = slugify(category_name)
                tag_slug = slugify(tag_name)
                exts = list({Path(f.get("rel_path", "")).suffix.lower() for f in f_list if Path(f.get("rel_path", "")).suffix})
                keywords = ent_name.split()

                tag_def = {
                    "id": f"auto_entity_{slugify(ent_name)}",
                    "nome": tag_name,
                    "categoria": category_name,
                    "categoria_key": f"custom.{cat_slug}",
                    "subcategoria": tag_name,
                    "entidade": tag_name,
                    "caminho_fisico": f"{cat_slug}/{tag_slug}",
                    "origem": "user",
                    "idioma": lang,
                    "sinonimos": [ent_name],
                    "palavras_chave": keywords,
                    "regex": [rf"\b{re.escape(ent_name)}\b"],
                    "extensoes": exts,
                    "confianca_base": 0.85,
                    "usar_para_automacao": True,
                    "version": 1
                }
                discovered_tags[tag_name.lower()] = tag_def

    def _infer_dynamic_category(self, name: str, keywords: List[str], file_types: List[str], lang: str) -> str:
        """Infers dynamic category name based on contextual tokens or format domain."""
        combined_text = f"{name} {' '.join(keywords)}".lower()
        tokens = set(re.findall(r"\w+", combined_text))

        for root_kw, cat_name in ROOT_CATEGORY_MAP.items():
            if root_kw in tokens:
                return cat_name

        if file_types:
            most_common_type = Counter(file_types).most_common(1)[0][0]
            type_names = {
                "image": "Fotos e Imagens" if lang == "ptBR" else "Photos and Images",
                "audio": "Áudio e Música" if lang == "ptBR" else "Audio and Music",
                "video": "Vídeos" if lang == "ptBR" else "Videos",
                "document": "Documentos" if lang == "ptBR" else "Documents",
                "text": "Textos e Notas" if lang == "ptBR" else "Texts and Notes",
                "archive": "Arquivos Compactados" if lang == "ptBR" else "Archives",
                "binary": "Aplicativos e Programas" if lang == "ptBR" else "Applications"
            }
            return type_names.get(most_common_type, "Outros" if lang == "ptBR" else "Other")

        return "Geral" if lang == "ptBR" else "General"
