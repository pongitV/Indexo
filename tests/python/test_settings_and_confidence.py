import pytest
import tempfile
from pathlib import Path
from PySide6.QtWidgets import QApplication
from app.widgets.settings_view import SettingsView
from app.widgets.organization_view import OrganizationSplitView
from app.config.settings_manager import SettingsManager

@pytest.fixture(scope="session")
def qapp():
    app = QApplication.instance()
    if app is None:
        app = QApplication([])
    return app

def test_settings_view_confidence_threshold_selection(qapp):
    mgr = SettingsManager()
    mgr.set("confidence_threshold", 0.65)

    view = SettingsView()
    assert hasattr(view, "combo_confidence")
    assert view.combo_confidence.count() >= 5

    # Check default selection is 0.65
    assert abs(float(view.combo_confidence.currentData()) - 0.65) < 0.01

    # Change to 0.75
    emitted_values = []
    view.confidence_threshold_changed.connect(lambda v: emitted_values.append(v))

    idx_75 = view.combo_confidence.findData(0.75)
    assert idx_75 >= 0
    view.combo_confidence.setCurrentIndex(idx_75)

    assert len(emitted_values) == 1
    assert abs(emitted_values[0] - 0.75) < 0.01
    assert abs(float(mgr.get("confidence_threshold")) - 0.75) < 0.01

def test_organization_view_dynamic_confidence_threshold(qapp):
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        mgr = SettingsManager()
        view = OrganizationSplitView()

        items = [
            {
                "rel_path": "manual_game.pdf",
                "abs_path": str(root / "manual_game.pdf"),
                "size": 1024,
                "status": "identificado",
                "category": "Documentos e Contratos",
                "tag_name": "Manual",
                "confidence": 0.60,
                "is_in_bundle": False,
                "bundle_folder": None
            }
        ]

        # Case 1: Threshold set to 0.70 -> Confidence 0.60 is below threshold, must NOT be organized
        mgr.set("confidence_threshold", 0.70)
        view.populate_results(items, root, set(), [], [])
        root_after = view.tree_after.topLevelItem(0)
        assert "0" in root_after.text(0)  # 0 files organized
        assert root_after.childCount() == 0

        # Case 2: Threshold set to 0.50 -> Confidence 0.60 is above threshold, MUST be organized
        mgr.set("confidence_threshold", 0.50)
        view.populate_results(items, root, set(), [], [])
        root_after = view.tree_after.topLevelItem(0)
        assert "1" in root_after.text(0)  # 1 file organized
        assert root_after.childCount() == 1
