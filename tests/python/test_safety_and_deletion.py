import pytest
import tempfile
import gc
from pathlib import Path
import indexo_core

from app.utils.file_ops import (
    move_file_safe, restore_session, get_restore_path
)

def test_delete_on_confirm_policy():
    with tempfile.TemporaryDirectory(ignore_cleanup_errors=True) as tmp:
        db_path = Path(tmp) / "indexo_test.db"
        db = indexo_core.PyIndexoDatabase.open(str(db_path))

        root_id = db.get_or_create_root("C:/TestDir", "TestDir")
        file_id = db.upsert_file(root_id, "doc1.pdf", "C:/TestDir/doc1.pdf", 1024, 1000, "hash1", "identificado")

        # Step 1: Marking for deletion does NOT delete from SQLite or disk
        db.mark_for_deletion(file_id, True)
        marked = db.list_marked_for_deletion()
        assert len(marked) == 1
        assert marked[0]["id"] == file_id

        # Step 2: Unmarking restores normal status
        db.mark_for_deletion(file_id, False)
        unmarked = db.list_marked_for_deletion()
        assert len(unmarked) == 0

        # Step 3: Deleting record removes from database
        db.delete_file_record(file_id)
        assert db.get_file_by_id(file_id) is None

        del db
        gc.collect()
