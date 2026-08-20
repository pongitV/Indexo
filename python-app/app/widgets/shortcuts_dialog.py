from typing import List, Tuple
from PySide6.QtCore import Qt
from PySide6.QtGui import QFont, QColor
from PySide6.QtWidgets import (
    QDialog, QVBoxLayout, QHBoxLayout, QLabel, QScrollArea,
    QWidget, QFrame, QPushButton, QGridLayout
)
from app.i18n.language_manager import tr
from app.widgets.smooth_scroll import SmoothScrollArea

class KeyCapWidget(QWidget):
    """Renders a realistic physical keyboard keycap."""
    def __init__(self, key_text: str, parent=None):
        super().__init__(parent)
        self.key_text = key_text
        layout = QHBoxLayout(self)
        layout.setContentsMargins(0, 0, 0, 0)
        
        lbl = QLabel(key_text)
        lbl.setFont(QFont("Consolas", 10, QFont.Weight.Bold))
        lbl.setAlignment(Qt.AlignmentFlag.AlignCenter)
        lbl.setStyleSheet("""
            QLabel {
                background: qlineargradient(x1:0, y1:0, x2:0, y2:1, stop:0 #3A3F4D, stop:1 #232731);
                color: #FFFFFF;
                border: 1px solid #4F5565;
                border-bottom: 3px solid #181A20;
                border-radius: 5px;
                padding: 4px 10px;
                min-width: 24px;
            }
        """)
        layout.addWidget(lbl)


class ShortcutCard(QFrame):
    """Card displaying a shortcut with realistic keyboard keys and description."""
    def __init__(self, keys: List[str], title: str, description: str, parent=None):
        super().__init__(parent)
        self.setObjectName("settings_panel")
        self.setStyleSheet("""
            QFrame {
                border-radius: 8px;
                padding: 10px 14px;
            }
        """)

        layout = QHBoxLayout(self)
        layout.setContentsMargins(12, 10, 12, 10)
        layout.setSpacing(16)

        # Left: Keyboard Keycaps
        keys_box = QHBoxLayout()
        keys_box.setSpacing(6)
        for i, k in enumerate(keys):
            if i > 0:
                plus = QLabel("+")
                plus.setFont(QFont("Inter", 11, QFont.Weight.Bold))
                plus.setStyleSheet("color: #888;")
                keys_box.addWidget(plus)
            keys_box.addWidget(KeyCapWidget(k))
        layout.addLayout(keys_box)

        # Right: Title and description
        text_box = QVBoxLayout()
        text_box.setSpacing(2)
        
        lbl_title = QLabel(title)
        lbl_title.setFont(QFont("Inter", 10, QFont.Weight.Bold))
        text_box.addWidget(lbl_title)

        lbl_desc = QLabel(description)
        lbl_desc.setFont(QFont("Inter", 9))
        lbl_desc.setStyleSheet("color: #888;")
        text_box.addWidget(lbl_desc)

        layout.addLayout(text_box, 1)


class ShortcutsGuideDialog(QDialog):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.setWindowTitle("Guia Completo de Atalhos do Teclado — Indexo")
        self.setMinimumSize(680, 600)
        self.init_ui()

    def init_ui(self):
        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(20, 20, 20, 20)
        main_layout.setSpacing(16)

        # Header
        header = QHBoxLayout()
        title_box = QVBoxLayout()
        lbl_title = QLabel("Atalhos do Teclado do Indexo")
        lbl_title.setFont(QFont("Inter", 14, QFont.Weight.Bold))
        title_box.addWidget(lbl_title)

        lbl_sub = QLabel("Aumente sua produtividade usando o teclado para navegar e organizar arquivos.")
        lbl_sub.setFont(QFont("Inter", 9))
        lbl_sub.setStyleSheet("color: #888;")
        title_box.addWidget(lbl_sub)

        header.addLayout(title_box, 1)
        main_layout.addLayout(header)

        # Scroll Area for Shortcuts
        self.scroll = SmoothScrollArea()

        container = QWidget()
        layout = QVBoxLayout(container)
        layout.setContentsMargins(0, 0, 0, 0)
        layout.setSpacing(16)

        # Section 1: Organização & Navegação
        layout.addWidget(self.create_section_label("Organização & Ações Principais"))
        layout.addWidget(ShortcutCard(["Ctrl", "O"], "Selecionar Pasta", "Abre o diálogo do Windows para escolher a pasta de trabalho."))
        layout.addWidget(ShortcutCard(["Ctrl", "Enter"], "Executar Organização", "Move fisicamente os arquivos das pastas permitidas para Indexo_Files."))
        layout.addWidget(ShortcutCard(["F5"], "Recarregar / Atualizar", "Reescaneia todos os arquivos da pasta ativa na hora."))
        layout.addWidget(ShortcutCard(["Esc"], "Fechar Prévia / Voltar", "Fecha o painel lateral de prévia ou retorna da tela de configurações."))

        # Section 2: Explorador de Arquivos & Árvores
        layout.addWidget(self.create_section_label("Explorador de Arquivos & Árvores"))
        layout.addWidget(ShortcutCard(["F2"], "Renomear Item", "Abre o diálogo para renomear o arquivo, pasta ou categoria selecionada."))
        layout.addWidget(ShortcutCard(["Delete"], "Excluir / Mover para Lixeira", "Marca o arquivo selecionado para envio seguro à lixeira."))
        layout.addWidget(ShortcutCard(["Ctrl", "C"], "Copiar Caminho", "Copia o caminho completo do arquivo ou pasta para a área de transferência."))
        layout.addWidget(ShortcutCard(["Duplo Clique"], "Abrir Programa Padrão", "Abre o arquivo no Word, Acrobat Reader, Excel ou visualizador."))

        # Section 3: Gestão & Configurações
        layout.addWidget(self.create_section_label("Gestão de Tags & Ajustes"))
        layout.addWidget(ShortcutCard(["Ctrl", "M"], "Gerenciador de Tags", "Abre a tela de regras, tags automáticas e manuais do sistema."))
        layout.addWidget(ShortcutCard(["Ctrl", "K"], "Paleta de Busca Rápida", "Busca instantânea com destaque semântico em tempo real."))
        layout.addWidget(ShortcutCard(["Ctrl", ","], "Configurações", "Abre as preferências, tema, idioma e personalizações."))

        layout.addStretch()
        self.scroll.setWidget(container)
        main_layout.addWidget(self.scroll, 1)

        # Bottom close button
        bottom = QHBoxLayout()
        bottom.addStretch()
        btn_close = QPushButton("Entendido")
        btn_close.setFocusPolicy(Qt.FocusPolicy.NoFocus)
        btn_close.setStyleSheet("background: #205EA6; color: white; font-weight: bold; padding: 8px 24px;")
        btn_close.clicked.connect(self.accept)
        bottom.addWidget(btn_close)
        main_layout.addLayout(bottom)

        if self.scroll.verticalScrollBar():
            self.scroll.verticalScrollBar().setValue(0)

    def create_section_label(self, text: str) -> QLabel:
        lbl = QLabel(text)
        lbl.setFont(QFont("Inter", 11, QFont.Weight.Bold))
        lbl.setStyleSheet("color: #205EA6; margin-top: 6px;")
        return lbl
