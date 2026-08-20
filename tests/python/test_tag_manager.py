import pytest
from pathlib import Path
from PySide6.QtWidgets import QApplication
from app.widgets.tag_manager_view import TagManagerView, TagEditDialog
from app.widgets.tree_view import VirtualTreeView
from app.widgets.organization_view import OrganizationSplitView
from app.config.settings_manager import SettingsManager
from app.i18n.language_manager import LanguageManager, tr
from app.main_window import MainWindow

def test_tag_manager_creation_and_editing(tmp_path):
    app = QApplication.instance()
    if app is None:
        app = QApplication([])

    sm = SettingsManager()
    
    # 1. Add user tag
    custom_tag = {
        "id": "user_contratos_urgentes",
        "nome": "Contratos Urgentes",
        "categoria": "Documentos e Contratos",
        "subcategoria": "Contratos Urgentes",
        "entidade": "Juridico",
        "caminho_fisico": "Documentos_e_Contratos/Contratos_Urgentes",
        "origem": "user",
        "idioma": "ptBR",
        "sinonimos": ["Urgent Contracts"],
        "palavras_chave": ["urgente", "contrato", "clausula"],
        "confianca_base": 0.95,
        "usar_para_automacao": True,
        "version": 1
    }
    sm.add_user_tag(custom_tag)

    user_tags = sm.get_user_tags()
    assert any(t.get("id") == "user_contratos_urgentes" for t in user_tags)

    # 2. Instantiate TagManagerView
    manager = TagManagerView()
    manager.load_tags()
    assert manager.table_system.rowCount() > 0
    assert manager.table_user.rowCount() > 0
    # Live count in tab titles
    assert "(" in manager.tab_widget.tabText(0)
    assert "(" in manager.tab_widget.tabText(1)

    # 3. Test filter
    manager.txt_filter.setText("urgentes")
    manager.apply_filter()
    assert manager.table_user.rowCount() >= 1

    # 4. Test VirtualTreeView and OrganizationSplitView show user tag even with 0 files
    tree_view = VirtualTreeView()
    tree_view.populate_results([], user_tags)
    assert tree_view.tree.topLevelItemCount() > 0

    org_view = OrganizationSplitView()
    org_view.populate_results([], tmp_path, set(), user_tags)
    assert org_view.tree_after.topLevelItemCount() > 0

    # 5. Clean up
    sm.remove_user_tag("user_contratos_urgentes")
    assert not any(t.get("id") == "user_contratos_urgentes" for t in sm.get_user_tags())


def test_file_reclassification_on_main_window(tmp_path):
    app = QApplication.instance()
    if app is None:
        app = QApplication([])

    win = MainWindow()
    test_file = tmp_path / "fatura_luz.pdf"
    test_file.write_text("dummy")

    item = {
        "file_id": 1,
        "abs_path": str(test_file),
        "rel_path": "fatura_luz.pdf",
        "tag_name": "Outros",
        "category": "Outros",
        "caminho_fisico": "Outros",
        "status": "pendente",
        "confidence": 0.5,
        "suggested_filename": "fatura_luz.pdf",
        "primary_date": "2024-01-01",
        "entity": "Enel"
    }
    win.last_results = [item]
    win.current_folder = tmp_path

    # Reclassify via on_file_reclassified
    win.on_file_reclassified(str(test_file), "Conta de Luz", "Faturas e Boletos")
    
    assert item["tag_name"] == "Conta de Luz"
    assert item["category"] == "Faturas e Boletos"
    assert item["status"] == "identificado"
    assert item["confidence"] == 1.0

    win.close()
