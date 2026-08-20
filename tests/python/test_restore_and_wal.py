import pytest
import tempfile
from pathlib import Path
from app.utils.file_ops import (
    move_file_safe, restore_session, get_restore_path
)

def test_atomic_move_and_restore_session():
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        src = root / "downloads" / "scan1.pdf"
        src.parent.mkdir(parents=True, exist_ok=True)
        src.write_text("dummy test content", encoding="utf-8")

        dest = root / "Indexo_Files" / "Faturas" / "2026-08-16_Conta-Luz.pdf"

        # Execute safe atomic move
        assert move_file_safe(src, dest, root) is True
        assert not src.exists()
        assert dest.exists()

        # Check WAL restore file
        restore_file = get_restore_path(root)
        assert restore_file.exists()

        # Revert / Undo session
        count, errors = restore_session(root)
        assert count == 1
        assert len(errors) == 0
        assert src.exists()
        assert not dest.exists()
        assert not restore_file.exists()
