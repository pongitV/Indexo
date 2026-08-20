from pathlib import Path
from typing import Optional
from PySide6.QtCore import Qt
from PySide6.QtGui import QFont, QIcon, QPixmap
from PySide6.QtWidgets import (
    QDialog, QVBoxLayout, QHBoxLayout, QLabel, QPushButton,
    QFileDialog, QFrame, QWidget
)
from app.i18n.language_manager import tr
from app.utils.theme_manager import get_app_icon_path, get_effective_theme
from app.config.settings_manager import SettingsManager

class OnboardingWizard(QDialog):
    def __init__(self, parent=None):
        super().__init__(parent)
        self.selected_folder: str = ""
        self.setWindowTitle(tr("app.title"))
        self.setFixedSize(620, 500)

        # Set Window Icon
        icon_path = get_app_icon_path()
        if icon_path.exists():
            self.setWindowIcon(QIcon(str(icon_path)))

        self.init_ui()

    def init_ui(self):
        sm = SettingsManager()
        is_dark = get_effective_theme(sm.get("theme", "system")) == "dark"

        layout = QVBoxLayout(self)
        layout.setContentsMargins(28, 22, 28, 22)
        layout.setSpacing(12)

        # 1. Header with App Icon & Title
        header_layout = QVBoxLayout()
        header_layout.setSpacing(4)
        header_layout.setAlignment(Qt.AlignmentFlag.AlignCenter)

        icon_path = get_app_icon_path()
        if icon_path.exists():
            lbl_icon = QLabel()
            pix = QPixmap(str(icon_path))
            if not pix.isNull():
                lbl_icon.setPixmap(pix.scaled(52, 52, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation))
                lbl_icon.setAlignment(Qt.AlignmentFlag.AlignCenter)
                header_layout.addWidget(lbl_icon)

        lbl_title = QLabel("INDEXO")
        lbl_title.setObjectName("lbl_before")
        lbl_title.setFont(QFont("Inter", 18, QFont.Weight.Bold))
        lbl_title.setAlignment(Qt.AlignmentFlag.AlignCenter)
        header_layout.addWidget(lbl_title)

        lbl_slogan = QLabel(tr("app.slogan"))
        lbl_slogan.setObjectName("lbl_subtext")
        lbl_slogan.setFont(QFont("Inter", 11, QFont.Weight.Medium))
        lbl_slogan.setAlignment(Qt.AlignmentFlag.AlignCenter)
        header_layout.addWidget(lbl_slogan)

        layout.addLayout(header_layout)

        # 2. Mental Model & Safety Principles Card (Theme-Adaptive)
        card = QFrame()
        card.setObjectName("card_top")

        card_bg = "#1C1B1A" if is_dark else "#F4F2EA"
        card_border = "#282726" if is_dark else "#DDD9CE"
        step_bg = "#242321" if is_dark else "#EAE7DC"
        step_border = "#343331" if is_dark else "#CECABF"
        text_primary = "#CECDC3" if is_dark else "#100F0F"
        text_muted = "#878580" if is_dark else "#5C5A55"
        accent_color = "#4385BE" if is_dark else "#205EA6"
        success_color = "#879A39" if is_dark else "#2B7A4B"

        card.setStyleSheet(f"""
            QFrame#card_top {{
                background-color: {card_bg};
                border: 1px solid {card_border};
                border-radius: 8px;
            }}
        """)

        card_layout = QVBoxLayout(card)
        card_layout.setContentsMargins(18, 14, 18, 14)
        card_layout.setSpacing(12)

        # Mental Model: 3 Spacious Step Columns with Clean Separator Arrows
        step_container = QWidget()
        step_layout = QHBoxLayout(step_container)
        step_layout.setContentsMargins(0, 0, 0, 0)
        step_layout.setSpacing(6)

        def create_step_badge(text: str, color: str) -> QWidget:
            box = QFrame()
            box.setStyleSheet(f"""
                QFrame {{
                    background-color: {step_bg};
                    border: 1px solid {step_border};
                    border-radius: 6px;
                    padding: 6px 4px;
                }}
            """)
            b_layout = QVBoxLayout(box)
            b_layout.setContentsMargins(6, 4, 6, 4)
            b_layout.setAlignment(Qt.AlignmentFlag.AlignCenter)

            lbl = QLabel(text)
            lbl.setFont(QFont("Inter", 9, QFont.Weight.DemiBold))
            lbl.setStyleSheet(f"color: {color}; border: none; background: transparent;")
            lbl.setAlignment(Qt.AlignmentFlag.AlignCenter)
            lbl.setWordWrap(True)
            b_layout.addWidget(lbl)
            return box

        box1 = create_step_badge(tr("onboarding.step_1"), text_primary)
        box2 = create_step_badge(tr("onboarding.step_2"), accent_color)
        box3 = create_step_badge(tr("onboarding.step_3"), success_color)

        def create_arrow() -> QLabel:
            arrow = QLabel("→")
            arrow.setFont(QFont("Inter", 12, QFont.Weight.Bold))
            arrow.setStyleSheet(f"color: {accent_color}; border: none; background: transparent;")
            arrow.setAlignment(Qt.AlignmentFlag.AlignCenter)
            arrow.setFixedWidth(20)
            return arrow

        step_layout.addWidget(box1, 1)
        step_layout.addWidget(create_arrow())
        step_layout.addWidget(box2, 1)
        step_layout.addWidget(create_arrow())
        step_layout.addWidget(box3, 1)

        card_layout.addWidget(step_container)

        # Safety Assurance Bullets (Shield Icon Preserved)
        lbl_safe_header = QLabel(f"🛡️ <b>{tr('onboarding.safe_title')}</b>")
        lbl_safe_header.setFont(QFont("Inter", 10))
        lbl_safe_header.setStyleSheet(f"color: {text_primary}; border: none; background: transparent;")
        card_layout.addWidget(lbl_safe_header)

        lbl_safe_bullets = QLabel(
            f"• {tr('onboarding.safe_1')}<br>"
            f"• {tr('onboarding.safe_2')}"
        )
        lbl_safe_bullets.setFont(QFont("Inter", 9))
        lbl_safe_bullets.setStyleSheet(f"color: {text_muted}; line-height: 140%; border: none; background: transparent;")
        card_layout.addWidget(lbl_safe_bullets)

        layout.addWidget(card)

        # 3. Folder Selection (No Emojis)
        btn_folder = QPushButton(tr("onboarding.btn_select_default"))
        btn_folder.setFont(QFont("Inter", 10, QFont.Weight.DemiBold))
        btn_folder.setCursor(Qt.CursorShape.PointingHandCursor)
        btn_folder.clicked.connect(self.select_folder)
        layout.addWidget(btn_folder)

        self.lbl_selected = QLabel(tr("onboarding.none_selected"))
        self.lbl_selected.setFont(QFont("Inter", 8))
        self.lbl_selected.setObjectName("lbl_subtext")
        self.lbl_selected.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(self.lbl_selected)

        layout.addStretch()

        # 4. Primary CTA Button (No Emojis)
        btn_start = QPushButton(tr("onboarding.btn_start"))
        btn_start.setFont(QFont("Inter", 11, QFont.Weight.Bold))
        btn_start.setCursor(Qt.CursorShape.PointingHandCursor)
        btn_start.setStyleSheet(f"""
            QPushButton {{
                background-color: {accent_color};
                color: #FFFFFF;
                border: none;
                border-radius: 6px;
                padding: 10px 20px;
            }}
            QPushButton:hover {{
                background-color: #2A70B8;
            }}
            QPushButton:pressed {{
                background-color: #1A4D85;
            }}
        """)
        btn_start.clicked.connect(self.accept)
        layout.addWidget(btn_start)

    def select_folder(self):
        folder = QFileDialog.getExistingDirectory(self, tr("action.select_folder"))
        if folder:
            self.selected_folder = folder
            self.lbl_selected.setText(folder)
            self.lbl_selected.setStyleSheet("color: #4385BE; font-weight: bold;")
