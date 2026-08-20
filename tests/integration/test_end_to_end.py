import os
import shutil
import tempfile
import gc
from pathlib import Path
import indexo_core
from app.config.settings_manager import SettingsManager
from app.classification.rule_loader import RuleLoader
from app.workers.index_worker import IndexWorker
from app.utils.file_ops import move_file_safe, restore_session

def test_full_pipeline_end_to_end():
    with tempfile.TemporaryDirectory(ignore_cleanup_errors=True) as tmp_dir:
        root = Path(tmp_dir) / "test_user_files"
        root.mkdir(parents=True, exist_ok=True)

        # Create sample files
        f1 = root / "fatura_luz.txt"
        f1.write_text("AES Sul Distribuidora de Energia - Conta de Luz Consumo ativo 350 kWh Vencimento 10/08/2026", encoding="utf-8")

        f2 = root / "boleto.txt"
        f2.write_text("Linha digitavel 23793381286008301352856000063307789012345678 Banco do Brasil", encoding="utf-8")

        f3 = root / "foto_praia.txt"
        f3.write_text("Foto simples de ferias", encoding="utf-8")

        # 1. Scan directory
        entries = indexo_core.py_scan_directory(str(root), False)
        assert len(entries) == 3

        # 2. Kernel Classification
        loader = RuleLoader()
        kernel = indexo_core.PyClassificationKernel.from_rules_json(loader.build_kernel_json())

        c1 = kernel.classify(f1.read_text(encoding="utf-8"), ".txt", 0.0)
        assert c1 is not None
        assert "luz" in c1["nome"].lower() or "energia" in c1["nome"].lower()

        # 3. Database operations
        db_path = root / "test.db"
        db = indexo_core.PyIndexoDatabase.open(str(db_path))
        root_id = db.get_or_create_root(str(root), root.name)
        
        file_id = db.upsert_file(root_id, "fatura_luz.txt", str(f1), f1.stat().st_size, int(f1.stat().st_mtime), "hash1", "identificado")
        assert file_id > 0

        # 4. FTS search
        db.update_fts_content("fatura_luz.txt", c1["nome"], "AES Sul", "energia kwh", f1.read_text(encoding="utf-8")[:600], "", "", "")
        search_res = db.search_fts("energia")
        assert len(search_res) == 1
        assert search_res[0]["rel_path"] == "fatura_luz.txt"

        # 5. Atomic physical move & Undo
        dest = root / "Indexo_Files" / "Faturas" / "2026-08-10_Conta-Luz_AES-Sul.txt"
        assert move_file_safe(f1, dest, root) is True
        assert dest.exists()
        assert not f1.exists()

        # Undo session
        count, errors = restore_session(root)
        assert count == 1
        assert f1.exists()
        assert not dest.exists()

        del db
        gc.collect()
