import os
import json
import shutil
from pathlib import Path
from typing import Optional

from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QFont
from PySide6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QPushButton,
    QFrame, QComboBox, QFileDialog, QMessageBox
)
from app.config.settings_manager import SettingsManager, get_app_dir, get_user_rules_path
from app.i18n.language_manager import tr, LanguageManager
from app.widgets.smooth_scroll import SmoothScrollArea
from app.classification.entity_regex import generate_standard_filename
from loguru import logger

class SettingsView(QWidget):
    back_requested = Signal()
    theme_changed = Signal(str)
    language_changed = Signal(str)
    font_size_changed = Signal(int)
    confidence_threshold_changed = Signal(float)
    tag_manager_requested = Signal()
    shortcuts_requested = Signal()
    power_user_toggled = Signal(bool)

    def __init__(self, parent=None):
        super().__init__(parent)
        self.settings_mgr = SettingsManager()
        self.init_ui()

    def init_ui(self):
        self.setObjectName("settings_panel")
        layout = QVBoxLayout(self)
        layout.setContentsMargins(24, 16, 24, 16)
        layout.setSpacing(14)

        # Top Header with "← Back" button and Title
        header_layout = QHBoxLayout()
        self.btn_back = QPushButton(tr("action.back"))
        self.btn_back.setFont(QFont("Inter", 10, QFont.Weight.Bold))
        self.btn_back.setStyleSheet("padding: 6px 16px; border-radius: 4px;")
        self.btn_back.clicked.connect(self.back_requested.emit)
        header_layout.addWidget(self.btn_back)

        self.lbl_title = QLabel(tr('nav.settings'))
        self.lbl_title.setFont(QFont("Inter", 14, QFont.Weight.Bold))
        header_layout.addWidget(self.lbl_title)

        header_layout.addStretch()
        layout.addLayout(header_layout)

        # Scrollable container for settings cards
        self.settings_scroll = SmoothScrollArea()
        self.settings_scroll.setObjectName("settings_scroll")

        container = QWidget()
        container.setObjectName("settings_container")
        c_layout = QVBoxLayout(container)
        c_layout.setContentsMargins(0, 0, 0, 0)
        c_layout.setSpacing(12)

        # 1. Theme Configuration Card
        theme_card = QFrame()
        theme_card.setObjectName("card_options")
        theme_layout = QVBoxLayout(theme_card)
        theme_layout.setContentsMargins(14, 14, 14, 14)
        theme_layout.setSpacing(8)

        self.lbl_theme_title = QLabel(tr("settings.theme_title"))
        self.lbl_theme_title.setFont(QFont("Inter", 10, QFont.Weight.Bold))
        theme_layout.addWidget(self.lbl_theme_title)

        theme_btns = QHBoxLayout()
        self.btn_theme_light = QPushButton(tr("settings.theme_light"))
        self.btn_theme_light.clicked.connect(lambda: self.theme_changed.emit("light"))
        theme_btns.addWidget(self.btn_theme_light)

        self.btn_theme_dark = QPushButton(tr("settings.theme_dark"))
        self.btn_theme_dark.clicked.connect(lambda: self.theme_changed.emit("dark"))
        theme_btns.addWidget(self.btn_theme_dark)

        self.btn_theme_sys = QPushButton(tr("settings.theme_system"))
        self.btn_theme_sys.clicked.connect(lambda: self.theme_changed.emit("system"))
        theme_btns.addWidget(self.btn_theme_sys)

        theme_btns.addStretch()
        theme_layout.addLayout(theme_btns)
        c_layout.addWidget(theme_card)

        # 1.5 Universal Font Size Card (Acessibilidade & Legibilidade)
        font_card = QFrame()
        font_card.setObjectName("card_options")
        font_card_layout = QVBoxLayout(font_card)
        font_card_layout.setContentsMargins(14, 14, 14, 14)
        font_card_layout.setSpacing(8)

        self.lbl_font_size_title = QLabel(tr('settings.font_size_label'))
        self.lbl_font_size_title.setFont(QFont("Inter", 11, QFont.Weight.Bold))
        font_card_layout.addWidget(self.lbl_font_size_title)

        font_btns = QHBoxLayout()
        self.btn_fs_small = QPushButton(tr("settings.font_size_small"))
        self.btn_fs_small.clicked.connect(lambda: self.font_size_changed.emit(13))
        font_btns.addWidget(self.btn_fs_small)

        self.btn_fs_normal = QPushButton(tr("settings.font_size_normal"))
        self.btn_fs_normal.clicked.connect(lambda: self.font_size_changed.emit(15))
        font_btns.addWidget(self.btn_fs_normal)

        self.btn_fs_large = QPushButton(tr("settings.font_size_large"))
        self.btn_fs_large.clicked.connect(lambda: self.font_size_changed.emit(17))
        font_btns.addWidget(self.btn_fs_large)

        self.btn_fs_xlarge = QPushButton(tr("settings.font_size_xlarge"))
        self.btn_fs_xlarge.clicked.connect(lambda: self.font_size_changed.emit(19))
        font_btns.addWidget(self.btn_fs_xlarge)

        font_btns.addStretch()
        font_card_layout.addLayout(font_btns)
        c_layout.addWidget(font_card)

        # 2. Language Selection Card
        lang_card = QFrame()
        lang_card.setObjectName("card_options")
        lang_card_layout = QVBoxLayout(lang_card)
        lang_card_layout.setContentsMargins(14, 14, 14, 14)
        lang_card_layout.setSpacing(8)

        self.lbl_lang_title = QLabel(tr("settings.language_label"))
        self.lbl_lang_title.setFont(QFont("Inter", 10, QFont.Weight.Bold))
        lang_card_layout.addWidget(self.lbl_lang_title)

        lang_btns = QHBoxLayout()
        self.btn_en = QPushButton("English (enUS)")
        self.btn_en.clicked.connect(lambda: self.language_changed.emit("enUS"))
        lang_btns.addWidget(self.btn_en)

        self.btn_pt = QPushButton("Português (ptBR)")
        self.btn_pt.clicked.connect(lambda: self.language_changed.emit("ptBR"))
        lang_btns.addWidget(self.btn_pt)

        lang_btns.addStretch()
        lang_card_layout.addLayout(lang_btns)
        c_layout.addWidget(lang_card)

        # 2.5 Minimum Confidence Threshold Card
        self.conf_card = QFrame()
        self.conf_card.setObjectName("card_options")
        conf_layout = QVBoxLayout(self.conf_card)
        conf_layout.setContentsMargins(14, 14, 14, 14)
        conf_layout.setSpacing(8)

        self.lbl_conf_title = QLabel(tr("settings.confidence_title"))
        self.lbl_conf_title.setFont(QFont("Inter", 10, QFont.Weight.Bold))
        conf_layout.addWidget(self.lbl_conf_title)

        self.lbl_conf_desc = QLabel(tr("settings.confidence_desc"))
        self.lbl_conf_desc.setObjectName("lbl_subtext")
        self.lbl_conf_desc.setFont(QFont("Inter", 9))
        self.lbl_conf_desc.setWordWrap(True)
        conf_layout.addWidget(self.lbl_conf_desc)

        self.combo_confidence = QComboBox()
        self.populate_confidence_options()
        self.combo_confidence.currentIndexChanged.connect(self.on_confidence_changed)
        conf_layout.addWidget(self.combo_confidence)
        c_layout.addWidget(self.conf_card)

        # 3. Keyboard Shortcuts Guide Card
        shortcuts_card = QFrame()
        shortcuts_card.setObjectName("card_options")
        sc_layout = QVBoxLayout(shortcuts_card)
        sc_layout.setContentsMargins(14, 14, 14, 14)
        sc_layout.setSpacing(8)

        self.lbl_shortcuts_title = QLabel("Atalhos do Teclado")
        self.lbl_shortcuts_title.setFont(QFont("Inter", 10, QFont.Weight.Bold))
        sc_layout.addWidget(self.lbl_shortcuts_title)

        lbl_sc_desc = QLabel("Consulte o guia visual com todos os atalhos e teclas de ação rápida do Indexo.")
        lbl_sc_desc.setObjectName("lbl_subtext")
        lbl_sc_desc.setFont(QFont("Inter", 9))
        sc_layout.addWidget(lbl_sc_desc)

        btn_show_shortcuts = QPushButton("Ver Todos os Atalhos de Teclado...")
        btn_show_shortcuts.setFont(QFont("Inter", 10, QFont.Weight.Medium))
        btn_show_shortcuts.setStyleSheet("background: #205EA6; color: white; padding: 6px 14px; border-radius: 4px;")
        btn_show_shortcuts.clicked.connect(self.shortcuts_requested.emit)
        sc_layout.addWidget(btn_show_shortcuts, 0, Qt.AlignmentFlag.AlignLeft)
        c_layout.addWidget(shortcuts_card)

        # 4. Tag Manager Shortcut Card
        self.tag_shortcut_card = QFrame()
        self.tag_shortcut_card.setObjectName("card_options")
        tag_sc_layout = QVBoxLayout(self.tag_shortcut_card)
        tag_sc_layout.setContentsMargins(14, 14, 14, 14)
        tag_sc_layout.setSpacing(8)

        self.lbl_tag_sc_title = QLabel(tr('tags.manager_title'))
        self.lbl_tag_sc_title.setFont(QFont("Inter", 10, QFont.Weight.Bold))
        tag_sc_layout.addWidget(self.lbl_tag_sc_title)

        self.btn_open_tag_manager = QPushButton(tr('settings.tag_manager_shortcut'))
        self.btn_open_tag_manager.setFont(QFont("Inter", 10, QFont.Weight.Medium))
        self.btn_open_tag_manager.clicked.connect(self.tag_manager_requested.emit)
        tag_sc_layout.addWidget(self.btn_open_tag_manager, 0, Qt.AlignmentFlag.AlignLeft)
        c_layout.addWidget(self.tag_shortcut_card)

        # 5. File Renaming Pattern Customization Card
        self.rename_card = QFrame()
        self.rename_card.setObjectName("card_options")
        ren_layout = QVBoxLayout(self.rename_card)
        ren_layout.setContentsMargins(14, 14, 14, 14)
        ren_layout.setSpacing(8)

        self.lbl_rename_title = QLabel(tr('settings.rename_title'))
        self.lbl_rename_title.setFont(QFont("Inter", 10, QFont.Weight.Bold))
        ren_layout.addWidget(self.lbl_rename_title)

        grid = QHBoxLayout()
        
        # Separator
        sep_vbox = QVBoxLayout()
        self.lbl_sep = QLabel(tr("settings.rename_sep_label"))
        sep_vbox.addWidget(self.lbl_sep)
        self.combo_sep = QComboBox()
        self.combo_sep.addItem("_ (Underscore)", "_")
        self.combo_sep.addItem("- (Hyphen)", "-")
        self.combo_sep.addItem(" - (Dash with spaces)", " - ")
        self.combo_sep.addItem(". (Dot)", ".")
        self.combo_sep.addItem("  (Space)", " ")
        curr_sep = self.settings_mgr.get("rename_separator", " - ")
        idx_sep = self.combo_sep.findData(curr_sep)
        if idx_sep >= 0:
            self.combo_sep.setCurrentIndex(idx_sep)
        self.combo_sep.currentIndexChanged.connect(self.on_rename_setting_changed)
        sep_vbox.addWidget(self.combo_sep)
        grid.addLayout(sep_vbox)

        # Date Position
        pos_vbox = QVBoxLayout()
        self.lbl_date_pos = QLabel(tr("settings.rename_date_pos_label"))
        pos_vbox.addWidget(self.lbl_date_pos)
        self.combo_date_pos = QComboBox()
        self.combo_date_pos.addItem(tr("settings.date_pos_prefix"), "prefix")
        self.combo_date_pos.addItem(tr("settings.date_pos_suffix"), "suffix")
        self.combo_date_pos.addItem(tr("settings.date_pos_none"), "none")
        curr_pos = self.settings_mgr.get("rename_date_position", "suffix")
        idx_pos = self.combo_date_pos.findData(curr_pos)
        if idx_pos >= 0:
            self.combo_date_pos.setCurrentIndex(idx_pos)
        self.combo_date_pos.currentIndexChanged.connect(self.on_rename_setting_changed)
        pos_vbox.addWidget(self.combo_date_pos)
        grid.addLayout(pos_vbox)

        # Date Format
        fmt_vbox = QVBoxLayout()
        self.lbl_date_fmt = QLabel(tr("settings.rename_date_fmt_label"))
        fmt_vbox.addWidget(self.lbl_date_fmt)
        self.combo_date_fmt = QComboBox()
        self.combo_date_fmt.addItem("DD-MM-YYYY (10-05-2024)", "DD-MM-YYYY")
        self.combo_date_fmt.addItem("YYYY-MM-DD (2024-05-10)", "YYYY-MM-DD")
        self.combo_date_fmt.addItem("YYYY_MM_DD (2024_05_10)", "YYYY_MM_DD")
        self.combo_date_fmt.addItem("YYYYMMDD (20240510)", "YYYYMMDD")
        self.combo_date_fmt.addItem("DD_MM_YYYY (10_05_2024)", "DD_MM_YYYY")
        self.combo_date_fmt.addItem("DDMMYYYY (10052024)", "DDMMYYYY")
        self.combo_date_fmt.addItem("MM-DD-YYYY (05-10-2024)", "MM-DD-YYYY")
        curr_fmt = self.settings_mgr.get("rename_date_format", "DD-MM-YYYY")
        idx_fmt = self.combo_date_fmt.findData(curr_fmt)
        if idx_fmt >= 0:
            self.combo_date_fmt.setCurrentIndex(idx_fmt)
        self.combo_date_fmt.currentIndexChanged.connect(self.on_rename_setting_changed)
        fmt_vbox.addWidget(self.combo_date_fmt)
        grid.addLayout(fmt_vbox)

        # Casing
        case_vbox = QVBoxLayout()
        self.lbl_casing = QLabel(tr("settings.rename_casing_label"))
        case_vbox.addWidget(self.lbl_casing)
        self.combo_casing = QComboBox()
        self.combo_casing.addItem(tr("settings.casing_title"), "title")
        self.combo_casing.addItem(tr("settings.casing_lower"), "lower")
        self.combo_casing.addItem(tr("settings.casing_upper"), "upper")
        self.combo_casing.addItem(tr("settings.casing_original"), "original")
        curr_case = self.settings_mgr.get("rename_casing", "title")
        idx_case = self.combo_casing.findData(curr_case)
        if idx_case >= 0:
            self.combo_casing.setCurrentIndex(idx_case)
        self.combo_casing.currentIndexChanged.connect(self.on_rename_setting_changed)
        case_vbox.addWidget(self.combo_casing)
        grid.addLayout(case_vbox)

        ren_layout.addLayout(grid)

        # Live Preview Label
        self.lbl_rename_preview = QLabel("")
        self.lbl_rename_preview.setStyleSheet("color: #205EA6; font-weight: bold; font-family: Consolas; font-size: 12px; padding: 4px;")
        ren_layout.addWidget(self.lbl_rename_preview)
        self.update_rename_preview()

        c_layout.addWidget(self.rename_card)

        # 6. Privacy & Local Content Card
        privacy_card = QFrame()
        privacy_card.setObjectName("privacy_card")
        priv_layout = QVBoxLayout(privacy_card)
        priv_layout.setContentsMargins(14, 14, 14, 14)
        priv_layout.setSpacing(6)
        
        self.lbl_priv = QLabel(tr("settings.privacy_title"))
        self.lbl_priv.setFont(QFont("Inter", 10, QFont.Weight.Bold))
        priv_layout.addWidget(self.lbl_priv)

        self.lbl_priv_desc = QLabel(tr("settings.privacy_desc"))
        self.lbl_priv_desc.setObjectName("lbl_subtext")
        self.lbl_priv_desc.setFont(QFont("Inter", 9))
        priv_layout.addWidget(self.lbl_priv_desc)

        self.btn_clear_content = QPushButton(tr('action.clear_content'))
        self.btn_clear_content.clicked.connect(self.clear_privacy_content)
        priv_layout.addWidget(self.btn_clear_content, 0, Qt.AlignmentFlag.AlignLeft)

        c_layout.addWidget(privacy_card)

        # 7. Profile Backup (Export / Import Rules)
        self.prof_card = QFrame()
        self.prof_card.setObjectName("prof_card")
        prof_layout = QVBoxLayout(self.prof_card)
        prof_layout.setContentsMargins(14, 14, 14, 14)
        prof_layout.setSpacing(6)
        
        self.lbl_prof = QLabel(tr("settings.profile_backup_title"))
        self.lbl_prof.setFont(QFont("Inter", 10, QFont.Weight.Bold))
        prof_layout.addWidget(self.lbl_prof)

        prof_btns = QHBoxLayout()
        self.btn_export_prof = QPushButton(tr('action.export_profile'))
        self.btn_export_prof.clicked.connect(self.export_profile)
        prof_btns.addWidget(self.btn_export_prof)

        self.btn_import_prof = QPushButton(tr('action.import_profile'))
        self.btn_import_prof.clicked.connect(self.import_profile)
        prof_btns.addWidget(self.btn_import_prof)

        prof_btns.addStretch()
        prof_layout.addLayout(prof_btns)
        c_layout.addWidget(self.prof_card)

        # 8. Export Log
        self.btn_export_log = QPushButton(tr('action.export_log'))
        self.btn_export_log.clicked.connect(self.export_log_file)
        c_layout.addWidget(self.btn_export_log, 0, Qt.AlignmentFlag.AlignLeft)

        c_layout.addStretch()
        self.settings_scroll.setWidget(container)
        layout.addWidget(self.settings_scroll)

    def on_rename_setting_changed(self):
        sep = self.combo_sep.currentData() or "_"
        pos = self.combo_date_pos.currentData() or "prefix"
        fmt = self.combo_date_fmt.currentData() or "YYYY-MM-DD"
        case = self.combo_casing.currentData() or "title"

        self.settings_mgr.set("rename_separator", sep)
        self.settings_mgr.set("rename_date_position", pos)
        self.settings_mgr.set("rename_date_format", fmt)
        self.settings_mgr.set("rename_casing", case)

        self.update_rename_preview()

    def update_rename_preview(self):
        cfg = {
            "rename_separator": self.combo_sep.currentData() or "_",
            "rename_date_position": self.combo_date_pos.currentData() or "prefix",
            "rename_date_format": self.combo_date_fmt.currentData() or "YYYY-MM-DD",
            "rename_casing": self.combo_casing.currentData() or "title"
        }
        ex_name = generate_standard_filename("2024-05-10", "Enel", "Conta Luz", ".pdf", "fatura_luz", cfg)
        self.lbl_rename_preview.setText(f"{tr('settings.rename_preview_label', default='Exemplo:')}  {ex_name}")

    def clear_privacy_content(self):
        reply = QMessageBox.question(
            self,
            tr("settings.privacy_title"),
            "Deseja realmente limpar todos os textos e miniaturas extraídos da memória e do banco local?",
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No
        )
        if reply == QMessageBox.StandardButton.Yes:
            try:
                app_dir = get_app_dir()
                cache_dir = app_dir / "cache"
                if cache_dir.exists():
                    shutil.rmtree(cache_dir, ignore_errors=True)
                QMessageBox.information(self, "Indexo", "Cache e textos locais limpos com sucesso!")
            except Exception as e:
                logger.error("Failed to clear local cache: {}", e)
                QMessageBox.warning(self, "Indexo", f"Falha ao limpar cache: {e}")

    def export_profile(self):
        file_path, _ = QFileDialog.getSaveFileName(self, tr("action.export_profile"), "indexo_rules_profile.json", "JSON (*.json)")
        if file_path:
            try:
                user_rules_path = get_user_rules_path()
                if user_rules_path.exists():
                    shutil.copy2(user_rules_path, file_path)
                else:
                    rules = self.settings_mgr.get_user_tags()
                    with open(file_path, "w", encoding="utf-8") as f:
                        json.dump(rules, f, indent=2, ensure_ascii=False)
                QMessageBox.information(self, "Indexo", "Perfil de regras exportado com sucesso!")
            except Exception as e:
                logger.error("Failed to export profile: {}", e)
                QMessageBox.warning(self, "Indexo", f"Falha ao exportar perfil: {e}")

    def import_profile(self):
        file_path, _ = QFileDialog.getOpenFileName(self, tr("action.import_profile"), "", "JSON (*.json)")
        if file_path:
            try:
                with open(file_path, "r", encoding="utf-8") as f:
                    rules = json.load(f)
                if isinstance(rules, list):
                    for r in rules:
                        if isinstance(r, dict) and "nome" in r:
                            self.settings_mgr.add_user_tag(r)
                    QMessageBox.information(self, "Indexo", "Perfil de regras importado com sucesso!")
                else:
                    QMessageBox.warning(self, "Indexo", "Arquivo de perfil inválido.")
            except Exception as e:
                logger.error("Failed to import profile: {}", e)
                QMessageBox.warning(self, "Indexo", f"Falha ao importar perfil: {e}")

    def export_log_file(self):
        log_dir = get_app_dir() / "logs"
        log_file = log_dir / "indexo.log"
        if not log_file.exists():
            QMessageBox.information(self, "Indexo", "Nenhum arquivo de log encontrado para exportar.")
            return

        dest_path, _ = QFileDialog.getSaveFileName(self, tr("action.export_log"), "indexo_diagnostic.log", "Log (*.log *.txt)")
        if dest_path:
            try:
                shutil.copy2(log_file, dest_path)
                QMessageBox.information(self, "Indexo", "Arquivo de diagnóstico exportado com sucesso!")
            except Exception as e:
                logger.error("Failed to export log file: {}", e)
                QMessageBox.warning(self, "Indexo", f"Falha ao exportar log: {e}")

    def populate_confidence_options(self):
        curr_val = round(float(self.settings_mgr.get("confidence_threshold", 0.65)), 2)
        self.combo_confidence.blockSignals(True)
        self.combo_confidence.clear()

        levels = [
            (0.50, tr("settings.confidence_50")),
            (0.60, tr("settings.confidence_60")),
            (0.65, tr("settings.confidence_65")),
            (0.70, tr("settings.confidence_70")),
            (0.75, tr("settings.confidence_75")),
            (0.80, tr("settings.confidence_80")),
            (0.85, tr("settings.confidence_85")),
            (0.90, tr("settings.confidence_90")),
            (0.95, tr("settings.confidence_95")),
        ]

        selected_idx = 2  # Default to 0.65
        for idx, (val, label) in enumerate(levels):
            self.combo_confidence.addItem(label, val)
            if abs(val - curr_val) < 0.01:
                selected_idx = idx

        self.combo_confidence.setCurrentIndex(selected_idx)
        self.combo_confidence.blockSignals(False)

    def on_confidence_changed(self):
        val = float(self.combo_confidence.currentData() or 0.65)
        self.settings_mgr.set("confidence_threshold", val)
        self.confidence_threshold_changed.emit(val)

    def retranslate_ui(self):
        self.btn_back.setText(tr("action.back"))
        self.lbl_title.setText(tr('nav.settings'))
        self.lbl_theme_title.setText(tr("settings.theme_title"))
        self.btn_theme_light.setText(tr("settings.theme_light"))
        self.btn_theme_dark.setText(tr("settings.theme_dark"))
        self.btn_theme_sys.setText(tr("settings.theme_system"))
        self.lbl_font_size_title.setText(tr('settings.font_size_label'))
        self.btn_fs_small.setText(tr("settings.font_size_small"))
        self.btn_fs_normal.setText(tr("settings.font_size_normal"))
        self.btn_fs_large.setText(tr("settings.font_size_large"))
        self.btn_fs_xlarge.setText(tr("settings.font_size_xlarge"))
        self.lbl_lang_title.setText(tr("settings.language_label"))
        self.lbl_conf_title.setText(tr("settings.confidence_title"))
        self.lbl_conf_desc.setText(tr("settings.confidence_desc"))
        self.populate_confidence_options()
        self.lbl_tag_sc_title.setText(tr('tags.manager_title'))
        self.btn_open_tag_manager.setText(tr('settings.tag_manager_shortcut'))
        self.lbl_rename_title.setText(tr('settings.rename_title'))
        self.lbl_sep.setText(tr("settings.rename_sep_label"))
        self.lbl_date_pos.setText(tr("settings.rename_date_pos_label"))
        self.lbl_date_fmt.setText(tr("settings.rename_date_fmt_label"))
        self.lbl_casing.setText(tr("settings.rename_casing_label"))
        self.lbl_priv.setText(tr("settings.privacy_title"))
        self.lbl_priv_desc.setText(tr("settings.privacy_desc"))
        self.btn_clear_content.setText(tr('action.clear_content'))
        self.lbl_prof.setText(tr("settings.profile_backup_title"))
        self.btn_export_prof.setText(tr('action.export_profile'))
        self.btn_import_prof.setText(tr('action.import_profile'))
        self.btn_export_log.setText(tr('action.export_log'))
        self.update_rename_preview()
