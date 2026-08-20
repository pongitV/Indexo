import os
import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Dict, List, Any, Optional, Set, Tuple
from loguru import logger
from app.i18n.language_manager import tr, LanguageManager
from app.classification.tag_discovery import TagDiscoveryEngine

GAME_EXTENSIONS = {
    ".exe", ".pak", ".pck", ".vpk", ".unity3d", ".wad", ".sav", ".dat",
    ".rom", ".iso", ".nds", ".gba", ".nsp", ".xci", ".cso", ".mod",
    ".bsp", ".vmf", ".gma", ".esp", ".esm", ".ba2", ".bsa"
}

GAME_INDICATOR_FILENAMES = {
    "steam_api.dll", "steam_api64.dll", "steam_appid.txt",
    "unityplayer.dll", "unitycrashhandler64.exe", "unitycrashhandler32.exe",
    "gamestate.bin", "savedata.bin", "dxgi.dll", "d3d11.dll", "d3d9.dll",
    "xinput1_3.dll", "xinput1_4.dll", "openvr_api.dll", "galaxy.dll",
    "galaxy64.dll", "fmod.dll", "fmodstudio.dll", "binkw32.dll", "binkw64.dll"
}

GAME_FOLDER_KEYWORDS = {
    "game", "games", "jogo", "jogos", "steam", "steamapps", "epic", "gog",
    "roms", "emulator", "saves", "mods", "binaries", "content", "shaders"
}

CODE_PROJECT_INDICATORS = {
    "package.json", "cargo.toml", "pyproject.toml", "cmakelists.txt",
    "makefile", "pom.xml", "build.gradle", ".git", "requirements.txt",
    "go.mod", "solution.sln", ".csproj", ".vcxproj"
}

@dataclass
class CohesiveBundle:
    folder_rel: str
    folder_name: str
    abs_path: str
    category: str
    category_key: str
    bundle_type: str  # "game", "application", "project", "media_album", "homogeneous_group"
    primary_executable: Optional[str] = None
    file_count: int = 0
    total_size: int = 0
    action: str = "move_parent"  # "move_parent", "keep", "disassemble"
    confidence: float = 0.95
    reason: str = ""
    file_rel_paths: List[str] = field(default_factory=list)

    def to_dict(self) -> Dict[str, Any]:
        return {
            "folder_rel": self.folder_rel,
            "folder_name": self.folder_name,
            "abs_path": self.abs_path,
            "category": self.category,
            "category_key": self.category_key,
            "bundle_type": self.bundle_type,
            "primary_executable": self.primary_executable,
            "file_count": self.file_count,
            "total_size": self.total_size,
            "action": self.action,
            "confidence": self.confidence,
            "reason": self.reason,
            "file_rel_paths": self.file_rel_paths,
        }


class SimilarityEngine:
    """
    Hierarchical Similarity & Cohesive Structure Engine.
    Evaluates similarities strictly in order:
    1. Nome do arquivo e da pasta (Folder context, matching executables, stem patterns)
    2. Conteúdo e Metadados (Text signatures, EXIF, tags, keywords)
    3. Tipo / Extensão (File format categorization)
    """

    def __init__(self):
        self.tag_discovery = TagDiscoveryEngine()

    def analyze_scan_results(
        self,
        root_dir: Path,
        entries: List[Dict[str, Any]],
        existing_tags: Optional[List[Dict[str, Any]]] = None
    ) -> Tuple[List[CohesiveBundle], Dict[str, CohesiveBundle], List[Dict[str, Any]]]:
        """
        Analyzes all scanned files in the target directory, detecting cohesive bundles
        (such as game packages, software installations, code projects, or albums),
        mapping file relative paths to their parent bundle, and synthesizing new semantic tags.
        """
        bundles: List[CohesiveBundle] = []
        file_to_bundle: Dict[str, CohesiveBundle] = {}
        discovered_tags: List[Dict[str, Any]] = []

        if not entries or not root_dir.exists():
            return bundles, file_to_bundle, discovered_tags

        # Group entries by top-level subfolders
        folder_groups: Dict[str, List[Dict[str, Any]]] = {}
        for entry in entries:
            rel = entry.get("rel_path", "").replace("\\", "/")
            parts = rel.split("/")
            if len(parts) > 1:
                top_folder = parts[0]
                if top_folder not in folder_groups:
                    folder_groups[top_folder] = []
                folder_groups[top_folder].append(entry)

        lang = LanguageManager.get_instance().current_language

        for folder_name, f_list in folder_groups.items():
            folder_abs = str(root_dir / folder_name).replace("\\", "/")
            bundle = self._evaluate_folder_cohesion(folder_name, folder_abs, f_list, lang)
            if bundle:
                bundles.append(bundle)
                for f in f_list:
                    rel_p = f.get("rel_path", "").replace("\\", "/")
                    file_to_bundle[rel_p] = bundle

        # Dynamic tag discovery from files, folder patterns and text
        discovered_tags = self.tag_discovery.discover_tags(root_dir, entries, existing_tags)

        logger.info("Detected {} cohesive bundles and {} discovered tags in {}", len(bundles), len(discovered_tags), root_dir)
        return bundles, file_to_bundle, discovered_tags

    def _evaluate_folder_cohesion(
        self,
        folder_name: str,
        folder_abs: str,
        files: List[Dict[str, Any]],
        lang: str
    ) -> Optional[CohesiveBundle]:
        """
        Determines if a directory is a cohesive unit (game, app, project, media album)
        using the hierarchy: Name -> Content -> Type.
        """
        total_files = len(files)
        if total_files == 0:
            return None

        total_size = sum(f.get("size", 0) for f in files)
        file_rels = [f.get("rel_path", "").replace("\\", "/") for f in files]

        # 1. HIERARCHY STEP 1: NAME ANALYSIS (Folder name, Executable names, Token similarity)
        folder_lower = folder_name.lower()
        folder_tokens = set(re.findall(r"\w+", folder_lower))

        executables = []
        exact_matching_exe = None
        game_indicator_hits = []

        for f in files:
            p = Path(f.get("rel_path", ""))
            f_name_lower = p.name.lower()
            ext = p.suffix.lower()

            if ext == ".exe":
                executables.append(p.name)
                # Check if executable stem matches folder name (e.g. PEAK / PEAK.exe)
                if p.stem.lower() == folder_lower or p.stem.lower().replace(" ", "") == folder_lower.replace(" ", ""):
                    exact_matching_exe = p.name
                elif folder_lower in p.stem.lower() or p.stem.lower() in folder_lower:
                    if not exact_matching_exe:
                        exact_matching_exe = p.name

            if f_name_lower in GAME_INDICATOR_FILENAMES:
                game_indicator_hits.append(f_name_lower)

        # Check if folder name itself contains game keywords
        has_game_folder_kw = any(kw in folder_tokens for kw in GAME_FOLDER_KEYWORDS)

        # Count game extensions
        game_ext_count = sum(1 for f in files if Path(f.get("rel_path", "")).suffix.lower() in GAME_EXTENSIONS)
        game_ext_ratio = game_ext_count / total_files if total_files > 0 else 0.0

        # A. GAME PACKAGE DETECTION
        if exact_matching_exe or game_indicator_hits or (len(executables) > 0 and (game_ext_ratio >= 0.20 or has_game_folder_kw)):
            primary_exe = exact_matching_exe or (executables[0] if executables else None)
            cat_name = "Jogos" if lang == "ptBR" else "Games"
            cat_key = "cat.jogos"
            reason = f"Executável do jogo detectado ({primary_exe})" if primary_exe else "Estrutura e arquivos de jogo detectados"
            
            return CohesiveBundle(
                folder_rel=folder_name,
                folder_name=folder_name,
                abs_path=folder_abs,
                category=cat_name,
                category_key=cat_key,
                bundle_type="game",
                primary_executable=primary_exe,
                file_count=total_files,
                total_size=total_size,
                action="move_parent",
                confidence=0.98 if exact_matching_exe else 0.90,
                reason=reason,
                file_rel_paths=file_rels,
            )

        # B. APPLICATION / SOFTWARE PACKAGE DETECTION
        if len(executables) > 0 and total_files >= 2:
            primary_exe = executables[0]
            cat_name = "Aplicativos" if lang == "ptBR" else "Applications"
            cat_key = "cat.aplicativos"
            return CohesiveBundle(
                folder_rel=folder_name,
                folder_name=folder_name,
                abs_path=folder_abs,
                category=cat_name,
                category_key=cat_key,
                bundle_type="application",
                primary_executable=primary_exe,
                file_count=total_files,
                total_size=total_size,
                action="move_parent",
                confidence=0.88,
                reason=f"Pacote de aplicativo com executável ({primary_exe})",
                file_rel_paths=file_rels,
            )

        # C. CODE / DEVELOPMENT PROJECT DETECTION
        code_indicators = [
            Path(f.get("rel_path", "")).name.lower()
            for f in files
            if Path(f.get("rel_path", "")).name.lower() in CODE_PROJECT_INDICATORS
        ]
        if code_indicators:
            cat_name = "Projetos e Código" if lang == "ptBR" else "Projects & Code"
            cat_key = "cat.projetos_codigo"
            return CohesiveBundle(
                folder_rel=folder_name,
                folder_name=folder_name,
                abs_path=folder_abs,
                category=cat_name,
                category_key=cat_key,
                bundle_type="project",
                primary_executable=None,
                file_count=total_files,
                total_size=total_size,
                action="move_parent",
                confidence=0.92,
                reason=f"Estrutura de projeto ({', '.join(code_indicators[:2])})",
                file_rel_paths=file_rels,
            )

        # D. HOMOGENEOUS / ALBUM FOLDER (E.g. Audio Album, Video Series)
        type_counts: Dict[str, int] = {}
        for f in files:
            ft = f.get("file_type", "other")
            type_counts[ft] = type_counts.get(ft, 0) + 1

        for ft, count in type_counts.items():
            if count / total_files >= 0.85 and total_files >= 3:
                if ft == "audio":
                    cat_name = "Áudio e Música" if lang == "ptBR" else "Audio and Music"
                    cat_key = "cat.midia_audio"
                    return CohesiveBundle(
                        folder_rel=folder_name,
                        folder_name=folder_name,
                        abs_path=folder_abs,
                        category=cat_name,
                        category_key=cat_key,
                        bundle_type="media_album",
                        primary_executable=None,
                        file_count=total_files,
                        total_size=total_size,
                        action="move_parent",
                        confidence=0.85,
                        reason="Álbum/Coleção de áudio unificada",
                        file_rel_paths=file_rels,
                    )
                elif ft == "video":
                    cat_name = "Vídeos" if lang == "ptBR" else "Videos"
                    cat_key = "cat.midia_video"
                    return CohesiveBundle(
                        folder_rel=folder_name,
                        folder_name=folder_name,
                        abs_path=folder_abs,
                        category=cat_name,
                        category_key=cat_key,
                        bundle_type="media_album",
                        primary_executable=None,
                        file_count=total_files,
                        total_size=total_size,
                        action="move_parent",
                        confidence=0.85,
                        reason="Coleção de vídeos/série unificada",
                        file_rel_paths=file_rels,
                    )

        return None

    def classify_by_hierarchy(
        self,
        rel_path: str,
        abs_path: str,
        file_type: str,
        extracted_text: str,
        candidate: Optional[Dict[str, Any]],
        parent_bundle: Optional[CohesiveBundle] = None,
        matching_tag: Optional[Dict[str, Any]] = None
    ) -> Dict[str, Any]:
        """
        Classifies an individual file using strict hierarchy:
        1. Name / Folder Context (Parent bundle & file name tokens)
        2. Dynamic Discovered / User Tag Match
        3. Content / Metadata (Text/rules match)
        4. Type / Format fallback
        """
        path_obj = Path(abs_path)
        ext = path_obj.suffix.lower()
        stem = path_obj.stem
        lang = LanguageManager.get_instance().current_language

        # 1. If file belongs to a cohesive parent bundle (e.g. PEAK/PEAK.exe, PEAK/data.pak)
        if parent_bundle:
            return {
                "category": parent_bundle.category,
                "category_key": parent_bundle.category_key,
                "tag_name": parent_bundle.folder_name,
                "caminho_fisico": f"{parent_bundle.category.replace(' ', '_')}/{parent_bundle.folder_name.replace(' ', '_')}",
                "confidence": parent_bundle.confidence,
                "status": "identificado",
                "is_in_bundle": True,
                "bundle_folder": parent_bundle.folder_rel,
                "bundle_type": parent_bundle.bundle_type,
                "hierarchy_source": "name_folder_bundle"
            }

        # 2. If matched a dynamically synthesized or user tag (by Name / Cluster / Entity)
        if matching_tag:
            return {
                "category": matching_tag.get("categoria", tr(f"type.{file_type}")),
                "category_key": matching_tag.get("categoria_key"),
                "tag_name": matching_tag.get("nome", tr(f"type.{file_type}")),
                "caminho_fisico": matching_tag.get("caminho_fisico", tr(f"type.{file_type}").replace(" ", "_")),
                "confidence": float(matching_tag.get("confianca_base", 0.85)),
                "status": "identificado",
                "is_in_bundle": False,
                "bundle_folder": None,
                "bundle_type": None,
                "hierarchy_source": "dynamic_discovered_tag"
            }

        # 3. Rule candidate match (Content / Specific Regex)
        if candidate and candidate.get("confianca", 0.0) >= 0.65:
            scores = candidate.get("scores", {})
            score_conteudo = scores.get("conteudo", 0.0)
            # A specific rule (e.g. boleto, comprovante, imposto) MUST have content match > 0.0
            if score_conteudo > 0.0 or candidate.get("origem") == "user":
                return {
                    "category": candidate.get("categoria", tr(f"type.{file_type}")),
                    "category_key": candidate.get("categoria_key"),
                    "tag_name": candidate.get("nome", tr(f"type.{file_type}")),
                    "caminho_fisico": candidate.get("caminho_fisico", tr(f"type.{file_type}").replace(" ", "_")),
                    "confidence": candidate.get("confianca", 0.0),
                    "status": "identificado",
                    "is_in_bundle": False,
                    "bundle_folder": None,
                    "bundle_type": None,
                    "hierarchy_source": "content_rule"
                }

        # 4. Name-based Game / Executable / Media detection
        if ext in GAME_EXTENSIONS:
            cat_name = "Jogos" if lang == "ptBR" else "Games"
            cat_key = "cat.jogos"
            return {
                "category": cat_name,
                "category_key": cat_key,
                "tag_name": stem,
                "caminho_fisico": f"{cat_name.replace(' ', '_')}/{stem.replace(' ', '_')}",
                "confidence": 0.88,
                "status": "identificado",
                "is_in_bundle": False,
                "bundle_folder": None,
                "bundle_type": "game",
                "hierarchy_source": "name_game_extension"
            }

        # 5. Type / Format fallback (Confidence < 0.65 -> Pendente, não organiza no preview)
        type_cat = tr(f"type.{file_type}")
        return {
            "category": type_cat,
            "category_key": f"type.{file_type}",
            "tag_name": type_cat,
            "caminho_fisico": type_cat.replace(" ", "_"),
            "confidence": 0.50,
            "status": "pendente",
            "is_in_bundle": False,
            "bundle_folder": None,
            "bundle_type": None,
            "hierarchy_source": "type_fallback"
        }
