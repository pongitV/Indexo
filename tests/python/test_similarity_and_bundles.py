import pytest
import tempfile
from pathlib import Path
from app.classification.similarity_engine import SimilarityEngine, CohesiveBundle
from app.utils.file_ops import move_folder_safe, restore_session, get_restore_path
from app.widgets.organization_view import OrganizationSplitView
from PySide6.QtWidgets import QApplication

@pytest.fixture(scope="session")
def qapp():
    app = QApplication.instance()
    if app is None:
        app = QApplication([])
    return app

def test_similarity_engine_game_bundle_detection():
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        
        # Create a game folder "PEAK" with PEAK.exe and supporting files
        peak_dir = root / "PEAK"
        peak_dir.mkdir(parents=True)
        (peak_dir / "PEAK.exe").write_text("binary", encoding="utf-8")
        (peak_dir / "data.pak").write_text("assets", encoding="utf-8")
        (peak_dir / "steam_api64.dll").write_text("dll", encoding="utf-8")
        (peak_dir / "config.ini").write_text("fullscreen=1", encoding="utf-8")

        entries = [
            {"rel_path": "PEAK/PEAK.exe", "abs_path": str(peak_dir / "PEAK.exe"), "size": 1000, "mtime": 100, "file_type": "binary"},
            {"rel_path": "PEAK/data.pak", "abs_path": str(peak_dir / "data.pak"), "size": 5000, "mtime": 100, "file_type": "binary"},
            {"rel_path": "PEAK/steam_api64.dll", "abs_path": str(peak_dir / "steam_api64.dll"), "size": 200, "mtime": 100, "file_type": "binary"},
            {"rel_path": "PEAK/config.ini", "abs_path": str(peak_dir / "config.ini"), "size": 50, "mtime": 100, "file_type": "text"},
        ]

        engine = SimilarityEngine()
        bundles, file_to_bundle, discovered_tags = engine.analyze_scan_results(root, entries)

        assert len(bundles) == 1
        bundle = bundles[0]
        assert bundle.folder_name == "PEAK"
        assert bundle.bundle_type == "game"
        assert bundle.primary_executable == "PEAK.exe"
        assert bundle.category in ["Jogos", "Games"]
        assert bundle.file_count == 4

        # Test hierarchical classification of individual files within the bundle
        for entry in entries:
            rel = entry["rel_path"]
            parent_bundle = file_to_bundle.get(rel)
            assert parent_bundle is not None
            hier = engine.classify_by_hierarchy(
                rel_path=rel,
                abs_path=entry["abs_path"],
                file_type=entry["file_type"],
                extracted_text="",
                candidate=None,
                parent_bundle=parent_bundle
            )
            assert hier["is_in_bundle"] is True
            assert hier["category"] in ["Jogos", "Games"]
            assert hier["tag_name"] == "PEAK"
            assert hier["status"] == "identificado"

def test_move_folder_safe_and_restore_session():
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        
        # Create game folder
        game_dir = root / "PEAK"
        game_dir.mkdir(parents=True)
        (game_dir / "PEAK.exe").write_text("game binary", encoding="utf-8")
        (game_dir / "level1.pak").write_text("level data", encoding="utf-8")
        sub_dir = game_dir / "Save"
        sub_dir.mkdir()
        (sub_dir / "save1.dat").write_text("save state", encoding="utf-8")

        dest_dir = root / "Indexo_Files" / "Jogos" / "PEAK"

        # Execute safe atomic folder move
        assert move_folder_safe(game_dir, dest_dir, root) is True
        assert not game_dir.exists()
        assert dest_dir.exists()
        assert (dest_dir / "PEAK.exe").exists()
        assert (dest_dir / "level1.pak").exists()
        assert (dest_dir / "Save" / "save1.dat").exists()

        # Check WAL restore file was created with entries for all 3 files
        restore_file = get_restore_path(root)
        assert restore_file.exists()

        # Revert / Undo session
        restored_count, errors = restore_session(root)
        assert restored_count == 3
        assert len(errors) == 0
        assert game_dir.exists()
        assert (game_dir / "PEAK.exe").exists()
        assert (game_dir / "level1.pak").exists()
        assert (game_dir / "Save" / "save1.dat").exists()
        assert not dest_dir.exists()
        assert not restore_file.exists()

def test_organization_view_cohesive_bundles_table(qapp):
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        view = OrganizationSplitView()
        
        items = [
            {
                "rel_path": "PEAK/PEAK.exe",
                "abs_path": str(root / "PEAK" / "PEAK.exe"),
                "size": 1024,
                "status": "identificado",
                "category": "Jogos",
                "tag_name": "PEAK",
                "caminho_fisico": "Jogos/PEAK",
                "confidence": 0.98,
                "is_in_bundle": True,
                "bundle_folder": "PEAK",
                "bundle_type": "game",
                "suggested_filename": "PEAK.exe"
            }
        ]

        bundles = [
            {
                "folder_rel": "PEAK",
                "folder_name": "PEAK",
                "abs_path": str(root / "PEAK"),
                "category": "Jogos",
                "category_key": "cat.jogos",
                "bundle_type": "game",
                "primary_executable": "PEAK.exe",
                "file_count": 1,
                "total_size": 1024,
                "action": "move_parent",
                "confidence": 0.98,
                "reason": "Executável do jogo detectado (PEAK.exe)",
                "file_rel_paths": ["PEAK/PEAK.exe"]
            }
        ]

        view.populate_results(items, root, set(), [], bundles)
        view.show()

        assert not view.card_cohesive.isHidden()
        assert view.table_cohesive.rowCount() == 1
        assert "PEAK" in view.table_cohesive.item(0, 0).text()
        assert "Jogos" in view.table_cohesive.item(0, 1).text()
        assert "PEAK.exe" in view.table_cohesive.item(0, 2).text()

def test_low_confidence_files_not_organized_in_preview(qapp):
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        view = OrganizationSplitView()

        items = [
            {
                "rel_path": "texture_diffuse.png",
                "abs_path": str(root / "texture_diffuse.png"),
                "size": 2048,
                "status": "pendente",
                "category": "Fotos e Imagens",
                "tag_name": "Fotos e Imagens",
                "confidence": 0.50,
                "is_in_bundle": False,
                "bundle_folder": None
            },
            {
                "rel_path": "conta_energia.pdf",
                "abs_path": str(root / "conta_energia.pdf"),
                "size": 51200,
                "status": "identificado",
                "category": "Faturas e Boletos",
                "tag_name": "Conta de Luz",
                "confidence": 0.95,
                "is_in_bundle": False,
                "bundle_folder": None
            }
        ]

        view.populate_results(items, root, set(), [], [])

        # Check tree_after root node: total organized should only be 1 (the 95% one), NOT the 50% texture image!
        root_after = view.tree_after.topLevelItem(0)
        assert root_after is not None
        assert "1" in root_after.text(0)  # Only 1 file organized
        
        # Check that only Faturas e Boletos is in the tree, and Fotos e Imagens is NOT in the tree
        cat_names = [root_after.child(i).text(0) for i in range(root_after.childCount())]
        assert any("Faturas e Boletos" in c for c in cat_names)
        assert not any("Fotos e Imagens" in c for c in cat_names)

def test_image_without_content_does_not_match_document_rules():
    engine = SimilarityEngine()
    
    # Candidate returned by kernel with only extension match (scores: conteudo=0.0, tipo=1.0)
    candidate_low_score = {
        "categoria": "Bancário e Financeiro",
        "nome": "Comprovantes",
        "confianca": 0.20,
        "scores": {"conteudo": 0.0, "tipo": 1.0}
    }

    hier = engine.classify_by_hierarchy(
        rel_path="texture_rock.png",
        abs_path="C:/games/texture_rock.png",
        file_type="image",
        extracted_text="",
        candidate=candidate_low_score,
        parent_bundle=None
    )

    # Must NOT be classified as Comprovantes! Must be marked pendente with low confidence
    assert hier["tag_name"] != "Comprovantes"
    assert hier["category"] != "Bancário e Financeiro"
    assert hier["status"] == "pendente"
    assert hier["confidence"] < 0.65

