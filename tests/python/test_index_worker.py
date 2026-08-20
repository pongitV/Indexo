import pytest
import time
from pathlib import Path
from PySide6.QtWidgets import QApplication
from app.workers.index_worker import IndexWorker
from app.i18n.language_manager import LanguageManager, tr

def test_index_worker_execution(tmp_path):
    # Ensure QApplication exists
    app = QApplication.instance()
    if app is None:
        app = QApplication([])


    # Create dummy files
    f1 = tmp_path / "documento_teste.pdf"
    f1.write_text("Extrato bancario conta corrente saldo", encoding="utf-8")
    
    f2 = tmp_path / "foto.jpg"
    f2.write_bytes(b"dummy image data")

    worker = IndexWorker(tmp_path)
    classified_items = []
    errors = []

    worker.file_classified.connect(lambda it: classified_items.append(it))
    worker.error_occurred.connect(lambda err: errors.append(err))

    worker.run()

    assert len(errors) == 0, f"Worker had errors: {errors}"
    assert len(classified_items) == 2
    for it in classified_items:
        assert "category" in it
        assert "suggested_filename" in it
        assert "status" in it

def test_index_worker_game_cohesive_folder(tmp_path):
    # Create game folder PEAK with executable and assets
    peak = tmp_path / "PEAK"
    peak.mkdir()
    (peak / "PEAK.exe").write_bytes(b"PEAK binary")
    (peak / "PEAK.pck").write_bytes(b"package data")
    (peak / "steam_api64.dll").write_bytes(b"steam dll")

    worker = IndexWorker(tmp_path)
    classified_items = []
    finished_summary = []
    errors = []

    worker.file_classified.connect(lambda it: classified_items.append(it))
    worker.scan_finished.connect(lambda s: finished_summary.append(s))
    worker.error_occurred.connect(lambda err: errors.append(err))

    worker.run()

    assert len(errors) == 0
    assert len(classified_items) == 3
    assert len(finished_summary) == 1

    summary = finished_summary[0]
    bundles = summary.get("cohesive_bundles", [])
    assert len(bundles) == 1
    assert bundles[0]["folder_name"] == "PEAK"
    assert bundles[0]["primary_executable"] == "PEAK.exe"
    assert bundles[0]["category"] in ["Jogos", "Games"]

    for it in classified_items:
        assert it["is_in_bundle"] is True
        assert it["bundle_folder"] == "PEAK"
        assert it["category"] in ["Jogos", "Games"]
        assert it["tag_name"] == "PEAK"
        # Suggested filename preserves original name for bundle files
        assert it["suggested_filename"] in ["PEAK.exe", "PEAK.pck", "steam_api64.dll"]

