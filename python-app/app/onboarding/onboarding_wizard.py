from pathlib import Path
from PySide6.QtCore import Qt
from PySide6.QtGui import QFont
from PySide6.QtWidgets import (
    QDialog, QVBoxLayout, QHBoxLayout, QLabel, QPushButton,
    QFileDialog, QFrame
)
from app.i18n.language_manager import tr

class OnboardingWizard(QDialog):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.selected_folder: str = ""
        self.setWindowTitle(tr("app.title"))
        self.setFixedSize(520, 360)
        self.init_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(24, 24, 24, 24)
        layout.setSpacing(16)

        # Mascot / Title
        lbl_title = QLabel("Bem-vindo ao Indexo")
        lbl_title.setFont(QFont("Inter", 16, QFont.Weight.Bold))
        lbl_title.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(lbl_title)

        lbl_slogan = QLabel(tr("app.slogan"))
        lbl_slogan.setFont(QFont("Inter", 10))
        lbl_slogan.setStyleSheet("color: #878580;")
        lbl_slogan.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(lbl_slogan)

        # Mental model illustration card
        card = QFrame()
        card.setStyleSheet("background: #F2F0E5; border-radius: 8px; padding: 16px;")
        card_layout = QVBoxLayout(card)

        lbl_model = QLabel("Pasta de Origem   ➔   [Organizar]   ➔   Estrutura Organizada")
        lbl_model.setFont(QFont("Inter", 11, QFont.Weight.Bold))
        lbl_model.setAlignment(Qt.AlignmentFlag.AlignCenter)
        card_layout.addWidget(lbl_model)

        lbl_safe = QLabel(
            "• Visão sem aplicação: por padrão, seus arquivos NÃO saem do lugar.\n"
            "• Tudo é 100% reversível e você tem controle total."
        )
        lbl_safe.setFont(QFont("Inter", 9))
        card_layout.addWidget(lbl_safe)

        layout.addWidget(card)

        # Folder selection
        btn_folder = QPushButton("Selecionar Pasta Padrão (Opcional)")
        btn_folder.clicked.connect(self.select_folder)
        layout.addWidget(btn_folder)

        self.lbl_selected = QLabel("Nenhuma pasta escolhida (você poderá escolher depois)")
        self.lbl_selected.setFont(QFont("Inter", 8))
        self.lbl_selected.setStyleSheet("color: #6F6E69;")
        self.lbl_selected.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(self.lbl_selected)

        layout.addStretch()

        # Start button
        btn_start = QPushButton("Começar a Organizar")
        btn_start.setFont(QFont("Inter", 10, QFont.Weight.Bold))
        btn_start.setStyleSheet("background: #205EA6; color: white; padding: 10px; border-radius: 6px;")
        btn_start.clicked.connect(self.accept)
        layout.addWidget(btn_start)

    def select_folder(self):
        folder = QFileDialog.getExistingDirectory(self, "Selecionar Pasta Padrão")
        if folder:
            self.selected_folder = folder
            self.lbl_selected.setText(folder)
