import pytest
from PySide6.QtWidgets import QApplication

@pytest.fixture(scope="session", autouse=True)
def qapp():
    app = QApplication.instance()
    if app is None:
        app = QApplication([])
    return app
