import pytest
from pathlib import Path
from PySide6.QtWidgets import QApplication
from app.config.settings_manager import SettingsManager
from app.main_window import MainWindow
from app.widgets.stats_view import StatsView
from app.widgets.shortcuts_dialog import ShortcutsGuideDialog

@pytest.fixture(scope="session")
def qapp():
    app = QApplication.instance()
    if app is None:
        app = QApplication([])
    return app

def test_settings_manager_power_user(tmp_path):
    mgr = SettingsManager()
    mgr.set_power_user(False)
    assert mgr.is_power_user() is False
    assert mgr.get("advanced_mode") is False

    mgr.set_power_user(True)
    assert mgr.is_power_user() is True
    assert mgr.get("advanced_mode") is True

    # Restore default for test isolation
    mgr.set_power_user(False)

def test_main_window_all_tabs_and_settings_visible(qapp):
    win = MainWindow()
    win.update_power_user_ui_visibility()
    assert win.main_tabs.count() == 8
    assert win.rename_card.isHidden() is False
    assert win.prof_card.isHidden() is False
    assert win.tag_shortcut_card.isHidden() is False
    win.close()

def test_stats_view_metrics(qapp):
    view = StatsView()
    items = [
        {"rel_path": "doc1.pdf", "size": 1024 * 1024 * 5, "file_type": "document", "status": "identificado", "category": "Faturas", "confidence": 0.95},
        {"rel_path": "foto.png", "size": 1024 * 1024 * 2, "file_type": "image", "status": "identificado", "category": "Fotos", "confidence": 0.90},
        {"rel_path": "outro.xyz", "size": 1024 * 500, "file_type": "other", "status": "pendente", "category": "Outros", "confidence": 0.10},
    ]
    view.update_stats(items, duplicates_count=1, duplicates_bytes=1024 * 500)
    assert view.card_files["value"].text() == "3"
    assert "7.5" in view.card_size["value"].text() or "MB" in view.card_size["value"].text()

def test_shortcuts_guide_dialog(qapp):
    dlg = ShortcutsGuideDialog()
    assert "Guia Completo de Atalhos" in dlg.windowTitle()
    dlg.close()

def test_universal_font_size_setting(qapp):
    win = MainWindow()
    win.change_font_size(19)
    assert win.settings_mgr.get("font_size") == 19
    win.change_font_size(15)
    assert win.settings_mgr.get("font_size") == 15
    win.close()
