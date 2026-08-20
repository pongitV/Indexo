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

def test_dynamic_category_learning_from_folder_topology():
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        engine = TagDiscoveryEngine()

        entries = [
            {"rel_path": "Viagens/Paris/foto1.jpg", "abs_path": str(root / "Viagens" / "Paris" / "foto1.jpg"), "file_type": "image"},
            {"rel_path": "Viagens/Paris/foto2.jpg", "abs_path": str(root / "Viagens" / "Paris" / "foto2.jpg"), "file_type": "image"},
            {"rel_path": "Projetos/MeuApp/main.py", "abs_path": str(root / "Projetos" / "MeuApp" / "main.py"), "file_type": "text"},
            {"rel_path": "Projetos/MeuApp/utils.py", "abs_path": str(root / "Projetos" / "MeuApp" / "utils.py"), "file_type": "text"},
        ]

        discovered = engine.discover_tags(root, entries, existing_tags=[])
        assert len(discovered) >= 2

        paris_tag = next(t for t in discovered if t["nome"].lower() == "paris")
        assert paris_tag["categoria"] == "Viagens"

        app_tag = next(t for t in discovered if t["nome"].lower() in ["meuapp", "meu app"])
        assert app_tag["categoria"] == "Projetos"

def test_dynamic_category_learning_from_prefix_clusters():
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        engine = TagDiscoveryEngine()

        entries = [
            {"rel_path": "Fatura_Enel_Jan.pdf", "abs_path": str(root / "Fatura_Enel_Jan.pdf"), "file_type": "document"},
            {"rel_path": "Fatura_Sabesp_Fev.pdf", "abs_path": str(root / "Fatura_Sabesp_Fev.pdf"), "file_type": "document"},
            {"rel_path": "Fatura_Claro_Mar.pdf", "abs_path": str(root / "Fatura_Claro_Mar.pdf"), "file_type": "document"},
            {"rel_path": "Relatorio_Vendas_2024.pdf", "abs_path": str(root / "Relatorio_Vendas_2024.pdf"), "file_type": "document"},
            {"rel_path": "Relatorio_Custos_2024.pdf", "abs_path": str(root / "Relatorio_Custos_2024.pdf"), "file_type": "document"},
        ]

        discovered = engine.discover_tags(root, entries, existing_tags=[])
        categories = {t["categoria"] for t in discovered}

        assert "Faturas" in categories
        assert "Relatórios" in categories

def test_settings_manager_get_all_categories():
    mgr = SettingsManager()
    mgr.data["tags"] = [
        {"id": "t1", "nome": "Paris", "categoria": "Viagens"},
        {"id": "t2", "nome": "Enel", "categoria": "Faturas"},
        {"id": "t3", "nome": "Sabesp", "categoria": "Faturas"},
    ]
    mgr.save_data()

    all_cats = mgr.get_all_categories()
    assert "Viagens" in all_cats
    assert "Faturas" in all_cats

def test_index_worker_dynamic_category_and_tag_assignment(qapp):
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        mgr = SettingsManager()
        mgr.data["tags"] = []
        mgr.save_data()

        # Create files under a custom parent directory
        custom_dir = root / "Contabilidade" / "Empresa_Alfa"
        custom_dir.mkdir(parents=True)
        (custom_dir / "Balanco_2024.pdf").write_text("Balanco financeiro", encoding="utf-8")
        (custom_dir / "DRE_2024.pdf").write_text("DRE demonstrativo", encoding="utf-8")

        worker = IndexWorker(root)
        classified_items = []
        worker.file_classified.connect(lambda it: classified_items.append(it))
        worker.run()

        assert len(classified_items) == 2
        for it in classified_items:
            assert it["status"] == "identificado"
            assert it["category"] == "Contabilidade"
            assert it["tag_name"] in ["Empresa Alfa", "Empresa_Alfa"]
