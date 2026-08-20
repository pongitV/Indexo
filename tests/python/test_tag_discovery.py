import pytest
import tempfile
from pathlib import Path
from PySide6.QtWidgets import QApplication
from app.classification.tag_discovery import TagDiscoveryEngine
from app.workers.index_worker import IndexWorker
from app.config.settings_manager import SettingsManager

@pytest.fixture(scope="session")
def qapp():
    app = QApplication.instance()
    if app is None:
        app = QApplication([])
    return app

def test_zero_start_clean_state():
    mgr = SettingsManager()
    # Ensure fresh state can have empty tags
    mgr.data["tags"] = []
    mgr.save_data()
    assert len(mgr.get_user_tags()) == 0

def test_tag_discovery_from_filename_patterns():
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        engine = TagDiscoveryEngine()

        entries = [
            {"rel_path": "Holerite_2024_01.pdf", "abs_path": str(root / "Holerite_2024_01.pdf"), "file_type": "document"},
            {"rel_path": "Holerite_2024_02.pdf", "abs_path": str(root / "Holerite_2024_02.pdf"), "file_type": "document"},
            {"rel_path": "Fatura_Enel_Jan.pdf", "abs_path": str(root / "Fatura_Enel_Jan.pdf"), "file_type": "document"},
            {"rel_path": "Fatura_Enel_Fev.pdf", "abs_path": str(root / "Fatura_Enel_Fev.pdf"), "file_type": "document"},
        ]

        discovered = engine.discover_tags(root, entries, existing_tags=[])
        tag_names = {t["nome"].lower() for t in discovered}

        assert "holerite" in tag_names
        assert any("enel" in name for name in tag_names)

        # Verify category inference
        holerite_tag = next(t for t in discovered if t["nome"].lower() == "holerite")
        assert "holerite" in holerite_tag["categoria"].lower() or "renda" in holerite_tag["categoria"].lower()

def test_tag_discovery_from_folder_names():
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        engine = TagDiscoveryEngine()

        entries = [
            {"rel_path": "Viagem_Paris/foto1.jpg", "abs_path": str(root / "Viagem_Paris" / "foto1.jpg"), "file_type": "image"},
            {"rel_path": "Viagem_Paris/foto2.jpg", "abs_path": str(root / "Viagem_Paris" / "foto2.jpg"), "file_type": "image"},
        ]

        discovered = engine.discover_tags(root, entries, existing_tags=[])
        tag_names = {t["nome"].lower() for t in discovered}

        assert "viagem paris" in tag_names
        paris_tag = next(t for t in discovered if t["nome"].lower() == "viagem paris")
        assert "viagens" in paris_tag["categoria"].lower() or "foto" in paris_tag["categoria"].lower()

def test_index_worker_auto_creates_and_persists_tags(qapp):
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        mgr = SettingsManager()
        mgr.data["tags"] = []
        mgr.save_data()
        assert len(mgr.get_user_tags()) == 0

        # Create multiple files sharing a pattern
        (root / "Relatorio_Vendas_2024_01.pdf").write_text("Vendas primeiro mes", encoding="utf-8")
        (root / "Relatorio_Vendas_2024_02.pdf").write_text("Vendas segundo mes", encoding="utf-8")

        worker = IndexWorker(root)
        classified_items = []
        worker.file_classified.connect(lambda it: classified_items.append(it))
        worker.run()

        assert len(classified_items) == 2
        
        # Check that user tags were automatically created and persisted in settings!
        saved_tags = mgr.get_user_tags()
        assert len(saved_tags) >= 1
        
        saved_names = [t["nome"].lower() for t in saved_tags]
        assert any("relatorio" in name or "vendas" in name for name in saved_names)

        # Check classified items received the auto-created tag
        for it in classified_items:
            assert it["status"] == "identificado"
            assert it["tag_name"] is not None
