import json
import os
import sys
from pathlib import Path
from typing import Dict, List, Any, Optional

from PySide6.QtCore import Qt, QSize, QPoint
from PySide6.QtGui import QFont, QKeySequence, QShortcut, QIcon, QPixmap
from PySide6.QtWidgets import (
    QMainWindow, QWidget, QVBoxLayout, QHBoxLayout, QLabel,
    QPushButton, QStackedWidget, QFileDialog, QMessageBox,
    QProgressBar, QSplitter, QFrame, QListWidget, QListWidgetItem,
    QTableWidget, QTableWidgetItem, QHeaderView, QCheckBox, QScrollArea,
    QComboBox, QAbstractItemView, QTabWidget, QMenu
)
from loguru import logger
import indexo_core

from app.config.settings_manager import (
    SettingsManager, get_app_dir, get_db_path, get_user_rules_path
)
from app.i18n.language_manager import tr, LanguageManager
from app.workers.index_worker import IndexWorker
from app.widgets.tree_view import VirtualTreeView
from app.widgets.preview_panel import PreviewPanel
from app.widgets.organization_view import OrganizationSplitView
from app.widgets.duplicate_view import DuplicateView
from app.widgets.trash_view import TrashView
from app.widgets.pending_list import PendingListView
from app.widgets.palette import SearchPaletteDialog
from app.widgets.lite_mode import LiteModeView
from app.widgets.folder_review import FolderReviewView
from app.widgets.tag_manager_view import TagManagerView
from app.widgets.settings_view import SettingsView
from app.widgets.stats_view import StatsView
from app.widgets.shortcuts_dialog import ShortcutsGuideDialog
from app.widgets.smooth_scroll import SmoothScrollArea
from app.utils.file_ops import move_file_safe, move_folder_safe, restore_session, get_restore_path
from app.utils.theme_manager import get_app_icon_path, apply_app_theme, is_system_dark_mode
from app.classification.entity_regex import generate_standard_filename

class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.settings_mgr = SettingsManager()
        self.current_folder: Optional[Path] = None
        self.last_results: List[Dict[str, Any]] = []
        self.last_cohesive_bundles: List[Dict[str, Any]] = []
        self.last_duplicates_count: int = 0
        self.last_duplicates_bytes: int = 0
        self.worker: Optional[IndexWorker] = None
        self.allowed_folders: set = set()
        self.previous_root_index: int = 0
        
        self.init_ui()
        self.setup_shortcuts()
        self.apply_theme()
        self.check_restore_availability()
        self.setAcceptDrops(True)
        self.update_power_user_ui_visibility()

    def dragEnterEvent(self, event):
        if event.mimeData().hasUrls():
            event.acceptProposedAction()

    def dropEvent(self, event):
        urls = event.mimeData().urls()
        if urls:
            for url in urls:
                local_path = url.toLocalFile()
                if local_path and os.path.isdir(local_path):
                    self.set_active_folder(Path(local_path))
                    break

    def init_ui(self):
        self.setWindowTitle(tr("app.title"))
        self.resize(1280, 840)
        self.setMinimumSize(980, 680)

        icon_path = get_app_icon_path()
        if icon_path.exists():
            self.setWindowIcon(QIcon(str(icon_path)))

        main_widget = QWidget()
        self.setCentralWidget(main_widget)
        self.main_layout = QVBoxLayout(main_widget)
        self.main_layout.setContentsMargins(12, 12, 12, 12)
        self.main_layout.setSpacing(8)

        # 1. Top Bar
        top_bar = QHBoxLayout()
        self.lbl_logo = QLabel("INDEXO")
        self.lbl_logo.setObjectName("lbl_before")
        self.lbl_logo.setFont(QFont("Inter", 15, QFont.Weight.Bold))
        top_bar.addWidget(self.lbl_logo)

        self.btn_select_dir = QPushButton(f"📁 {tr('action.select_folder')} (Ctrl+O)")
        self.btn_select_dir.setFont(QFont("Inter", 11, QFont.Weight.Bold))
        self.btn_select_dir.clicked.connect(self.select_folder_dialog)
        self.btn_select_dir.setVisible(False)
        top_bar.addWidget(self.btn_select_dir)

        self.lbl_current_folder = QLabel("")
        self.lbl_current_folder.setObjectName("lbl_subtext")
        self.lbl_current_folder.setFont(QFont("Consolas", 11))
        self.lbl_current_folder.setVisible(False)
        top_bar.addWidget(self.lbl_current_folder)

        top_bar.addStretch()

        self.btn_search_palette = QPushButton(f"{tr('search.placeholder')}")
        self.btn_search_palette.setFont(QFont("Inter", 11))
        self.btn_search_palette.clicked.connect(self.open_search_palette)
        top_bar.addWidget(self.btn_search_palette)

        # Settings gear icon button
        self.btn_settings_icon = QPushButton("⚙️")
        self.btn_settings_icon.setToolTip(f"{tr('nav.settings')} (Ctrl+,)")
        self.btn_settings_icon.setFont(QFont("Segoe UI Emoji", 12))
        self.btn_settings_icon.setCursor(Qt.CursorShape.PointingHandCursor)
        self.btn_settings_icon.clicked.connect(self.toggle_settings_view)
        top_bar.addWidget(self.btn_settings_icon)

        self.main_layout.addLayout(top_bar)

        # 2. Main Root View Stack:
        # [0] Clean Welcome Landing Screen
        # [1] Workspace (Split View, Duplicates, Trash, etc.)
        # [2] Dedicated Full-Screen Settings View
        # [3] Dedicated Full-Screen Tag Manager View
        self.root_stack = QStackedWidget()

        # View 0: Clean Welcome Landing Screen
        self.welcome_widget = self.create_welcome_view()
        self.root_stack.addWidget(self.welcome_widget)

        # View 1: Main Workspace Split View
        self.workspace_widget = self.create_workspace_view()
        self.root_stack.addWidget(self.workspace_widget)

        # View 2: Dedicated Clean Settings Screen (Modularized SRP)
        self.settings_widget = SettingsView()
        self.settings_widget.back_requested.connect(self.go_back_from_settings_or_tags)
        self.settings_widget.theme_changed.connect(self.change_theme)
        self.settings_widget.font_size_changed.connect(self.change_font_size)
        self.settings_widget.language_changed.connect(self.change_language)
        self.settings_widget.confidence_threshold_changed.connect(self.on_confidence_threshold_changed)
        self.settings_widget.tag_manager_requested.connect(self.toggle_tag_manager_view)
        self.settings_widget.shortcuts_requested.connect(self.show_shortcuts_guide)
        self.root_stack.addWidget(self.settings_widget)

        # Backward compatibility references for tests
        self.rename_card = self.settings_widget.rename_card
        self.prof_card = self.settings_widget.prof_card
        self.main_layout.addWidget(self.root_stack, 1)

        # 3. Bottom Bar
        self.bottom_bar = QHBoxLayout()
        self.lbl_status = QLabel(tr("status.ready"))
        self.lbl_status.setFont(QFont("Inter", 11))
        self.progress_bar = QProgressBar()
        self.progress_bar.setVisible(False)
        self.progress_bar.setFixedHeight(14)
        self.progress_bar.setMaximumWidth(280)

        self.btn_cancel_scan = QPushButton(tr("action.cancel"))
        self.btn_cancel_scan.setVisible(False)
        self.btn_cancel_scan.setFont(QFont("Inter", 10))
        self.btn_cancel_scan.clicked.connect(self.cancel_scan)

        self.bottom_bar.addWidget(self.lbl_status)
        self.bottom_bar.addStretch()
        self.bottom_bar.addWidget(self.progress_bar)
        self.bottom_bar.addWidget(self.btn_cancel_scan)
        self.main_layout.addLayout(self.bottom_bar)

        # Start on welcome landing screen
        self.root_stack.setCurrentIndex(0)

    def create_welcome_view(self) -> QWidget:
        panel = QWidget()
        layout = QVBoxLayout(panel)
        layout.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.setSpacing(14)

        layout.addStretch(2)

        # App Icon Logo
        icon_path = get_app_icon_path()
        if icon_path.exists():
            lbl_welcome_icon = QLabel()
            pix = QPixmap(str(icon_path))
            if not pix.isNull():
                lbl_welcome_icon.setPixmap(pix.scaled(72, 72, Qt.AspectRatioMode.KeepAspectRatio, Qt.TransformationMode.SmoothTransformation))
                lbl_welcome_icon.setAlignment(Qt.AlignmentFlag.AlignCenter)
                layout.addWidget(lbl_welcome_icon)

        self.lbl_welcome_logo = QLabel("INDEXO")
        self.lbl_welcome_logo.setObjectName("lbl_before")
        self.lbl_welcome_logo.setFont(QFont("Inter", 32, QFont.Weight.Bold))
        self.lbl_welcome_logo.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(self.lbl_welcome_logo)

        self.lbl_welcome_slogan = QLabel(tr("welcome.slogan"))
        self.lbl_welcome_slogan.setFont(QFont("Inter", 14, QFont.Weight.Medium))
        self.lbl_welcome_slogan.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.lbl_welcome_slogan.setObjectName("lbl_subtext")
        layout.addWidget(self.lbl_welcome_slogan)

        layout.addSpacing(16)

        card = QFrame()
        card.setObjectName("card_top")
        card_layout = QVBoxLayout(card)
        card_layout.setContentsMargins(36, 26, 36, 26)
        card_layout.setSpacing(14)
        card_layout.setAlignment(Qt.AlignmentFlag.AlignCenter)

        self.lbl_welcome_prompt = QLabel(tr("welcome.select_prompt"))
        self.lbl_welcome_prompt.setFont(QFont("Inter", 12))
        self.lbl_welcome_prompt.setAlignment(Qt.AlignmentFlag.AlignCenter)
        card_layout.addWidget(self.lbl_welcome_prompt)

        self.btn_welcome_select = QPushButton(tr("welcome.btn_select"))
        self.btn_welcome_select.setFont(QFont("Inter", 13, QFont.Weight.Bold))
        self.btn_welcome_select.setCursor(Qt.CursorShape.PointingHandCursor)
        self.btn_welcome_select.setStyleSheet("background: #205EA6; color: white; padding: 12px 32px; border-radius: 6px;")
        self.btn_welcome_select.clicked.connect(self.select_folder_dialog)
        card_layout.addWidget(self.btn_welcome_select)

        self.lbl_drop_hint = QLabel(tr("welcome.drop_hint"))
        self.lbl_drop_hint.setObjectName("lbl_subtext")
        self.lbl_drop_hint.setFont(QFont("Inter", 11))
        self.lbl_drop_hint.setAlignment(Qt.AlignmentFlag.AlignCenter)
        card_layout.addWidget(self.lbl_drop_hint)

        layout.addWidget(card, 0, Qt.AlignmentFlag.AlignCenter)

        layout.addStretch(3)
        return panel

    def create_workspace_view(self) -> QWidget:
        # Main horizontal splitter with clean Tabs on the left/center and collapsible Preview on the right
        workspace_splitter = QSplitter(Qt.Orientation.Horizontal)

        self.main_tabs = QTabWidget()
        self.main_tabs.setFont(QFont("Inter", 12, QFont.Weight.Medium))
        self.main_tabs.setDocumentMode(True)

        # Tab 0: Primary Central Organization Split View (Antes x Depois — Foco Principal)
        self.organization_view = OrganizationSplitView()
        self.organization_view.file_selected.connect(self.on_file_selected_for_preview)
        self.organization_view.folder_perm_toggled.connect(self.on_folder_perm_toggled)
        self.organization_view.tag_rename_requested.connect(self.on_tag_renamed)
        self.organization_view.file_reclassified.connect(self.on_file_reclassified)
        self.organization_view.file_marked_trash.connect(self.on_file_marked_trash)
        self.organization_view.bundle_action_changed.connect(self.on_bundle_action_changed)
        self.organization_view.refresh_requested.connect(self.start_scan)
        self.organization_view.execute_requested.connect(self.execute_organization)
        self.organization_view.restore_requested.connect(self.restore_last_session_action)

        # Tab 1: Stats & Visual Charts View
        self.stats_view = StatsView()

        # Tab 2: Virtual Categories Tree
        self.tree_view = VirtualTreeView()
        self.tree_view.file_selected.connect(self.on_file_selected_for_preview)
        self.tree_view.tag_rename_requested.connect(self.on_tag_renamed)
        self.tree_view.file_reclassified.connect(self.on_file_reclassified)
        self.tree_view.file_marked_trash.connect(self.on_file_marked_trash)
        self.tree_view.refresh_requested.connect(self.start_scan)
        self.tree_view.tag_manager_requested.connect(self.toggle_tag_manager_view)

        # Tab 3: Duplicates
        self.duplicate_view = DuplicateView()
        self.duplicate_view.file_marked.connect(self.on_trash_or_duplicate_changed)

        # Tab 4: Trash & History
        self.trash_view = TrashView()
        self.trash_view.trash_updated.connect(self.on_trash_or_duplicate_changed)

        # Power User Tabs
        self.lite_mode_view = LiteModeView()
        self.pending_view = PendingListView()
        self.pending_view.file_selected.connect(self.on_file_selected_for_preview)
        self.pending_view.file_reclassified.connect(self.on_file_reclassified)
        self.pending_view.file_marked_trash.connect(self.on_file_marked_trash)
        self.pending_view.promotion_suggested.connect(self.on_promotion_suggested)
        self.pending_view.refresh_requested.connect(self.start_scan)
        self.folder_review_view = FolderReviewView()

        self.repopulate_tabs()
        self.main_tabs.currentChanged.connect(self.on_tab_changed)

        workspace_splitter.addWidget(self.main_tabs)

        # Right/Collapsible Preview Panel
        self.preview_panel = PreviewPanel()
        self.preview_panel.setMaximumWidth(450)
        workspace_splitter.addWidget(self.preview_panel)

        workspace_splitter.setStretchFactor(0, 3)
        workspace_splitter.setStretchFactor(1, 1)

        return workspace_splitter

    def repopulate_tabs(self):
        curr_widget = self.main_tabs.currentWidget() if hasattr(self, 'main_tabs') else None
        self.main_tabs.blockSignals(True)
        self.main_tabs.clear()

        # 1. Primary Organization View
        self.main_tabs.addTab(self.organization_view, f"📁 {tr('nav.organization')}")

        # 2. Stats & Charts
        self.main_tabs.addTab(self.stats_view, "📊 Estatísticas")

        # 3. Semantic Tags View
        self.main_tabs.addTab(self.tree_view, f"🏷️ {tr('nav.tags')}")

        # 4. Duplicates
        self.main_tabs.addTab(self.duplicate_view, f"👯 {tr('nav.duplicates')}")

        # 5. Trash
        self.main_tabs.addTab(self.trash_view, f"🗑️ {tr('nav.trash')}")

        # 6. Rename Only
        self.main_tabs.addTab(self.lite_mode_view, f"✏️ {tr('nav.lite_mode')}")

        # 7. Move Only
        self.main_tabs.addTab(self.folder_review_view, f"📁 {tr('nav.folder_review')}")

        # 8. Pending / Ambiguous Files
        self.main_tabs.addTab(self.pending_view, f"❓ {tr('nav.pending')}")

        self.main_tabs.blockSignals(False)

        if curr_widget is not None:
            idx = self.main_tabs.indexOf(curr_widget)
            if idx >= 0:
                self.main_tabs.setCurrentIndex(idx)
                return
        self.main_tabs.setCurrentIndex(0)

    def on_tab_changed(self, index: int):
        widget = self.main_tabs.widget(index)
        if widget == self.stats_view and self.last_results:
            self.stats_view.update_stats(self.last_results, getattr(self, 'last_duplicates_count', 0), getattr(self, 'last_duplicates_bytes', 0))
        elif widget == self.duplicate_view and self.last_results:
            root_id = self.last_results[0].get("root_id", 1)
            self.duplicate_view.load_duplicates(root_id)
        elif widget == self.trash_view:
            self.trash_view.load_trash()
        elif widget == self.pending_view and self.last_results:
            self.pending_view.populate_pending(self.last_results)
        elif widget == self.folder_review_view and self.current_folder:
            if self.last_results:
                self.folder_review_view.load_from_items(self.last_results, self.current_folder)
            else:
                self.folder_review_view.scan_preorganized_folders(self.current_folder)
        elif widget == self.lite_mode_view and self.current_folder:
            if not self.lite_mode_view.target_dir or self.lite_mode_view.target_dir != self.current_folder:
                self.lite_mode_view.target_dir = self.current_folder
                self.lite_mode_view.lbl_folder_path.setText(str(self.current_folder))
                self.lite_mode_view.scan_and_plan()
        elif widget == self.tree_view and self.last_results:
            user_tags = self.settings_mgr.get_user_tags()
            self.tree_view.populate_results(self.last_results, user_tags)

    def update_power_user_ui_visibility(self):
        if hasattr(self, 'rename_card'):
            self.rename_card.setVisible(True)
        if hasattr(self, 'prof_card'):
            self.prof_card.setVisible(True)
        if hasattr(self, 'tag_shortcut_card'):
            self.tag_shortcut_card.setVisible(True)
        if hasattr(self, 'main_tabs'):
            self.repopulate_tabs()

    def setup_shortcuts(self):
        QShortcut(QKeySequence("Ctrl+O"), self, self.select_folder_dialog)
        QShortcut(QKeySequence("Ctrl+K"), self, self.open_search_palette)
        QShortcut(QKeySequence("Ctrl+M"), self, self.toggle_tag_manager_view)
        QShortcut(QKeySequence("Ctrl+,"), self, self.toggle_settings_view)
        QShortcut(QKeySequence("Escape"), self, self.handle_escape_key)
        QShortcut(QKeySequence("Ctrl+Return"), self, self.execute_organization)
        QShortcut(QKeySequence("Ctrl+Enter"), self, self.execute_organization)

    def handle_escape_key(self):
        if self.root_stack.currentIndex() in [2, 3]:
            self.go_back_from_settings_or_tags()
        elif self.preview_panel.isVisible():
            self.preview_panel.close_preview()

    def toggle_settings_view(self):
        if self.root_stack.currentIndex() == 2:
            self.go_back_from_settings_or_tags()
        else:
            if self.root_stack.currentIndex() not in [2, 3]:
                self.previous_root_index = self.root_stack.currentIndex()
            self.root_stack.setCurrentIndex(2)
            if hasattr(self, 'settings_scroll') and self.settings_scroll.verticalScrollBar():
                self.settings_scroll.verticalScrollBar().setValue(0)

    def navigate_to_tab(self, target_widget: QWidget):
        if self.root_stack.currentIndex() not in [2, 3]:
            self.previous_root_index = self.root_stack.currentIndex()
        self.root_stack.setCurrentIndex(1)
        self.main_tabs.setCurrentWidget(target_widget)
        self.btn_select_dir.setVisible(True)
        if not self.current_folder:
            self.lbl_status.setText(tr("view.select_folder_hint"))

    def toggle_tag_manager_view(self):
        if self.root_stack.currentIndex() == 3:
            self.go_back_from_settings_or_tags()
        else:
            if self.root_stack.currentIndex() not in [2, 3]:
                self.previous_root_index = self.root_stack.currentIndex()
            self.tag_manager_widget.load_tags()
            self.root_stack.setCurrentIndex(3)

    def go_back_from_settings_or_tags(self):
        self.root_stack.setCurrentIndex(self.previous_root_index)

    def on_tags_changed_by_manager(self):
        user_tags = self.settings_mgr.get_user_tags()
        self.tree_view.populate_results(self.last_results, user_tags)
        if self.current_folder:
            self.organization_view.populate_results(self.last_results, self.current_folder, self.allowed_folders, user_tags, self.last_cohesive_bundles)

    def on_confidence_threshold_changed(self, threshold: float):
        user_tags = self.settings_mgr.get_user_tags()
        if self.current_folder:
            self.organization_view.populate_results(
                self.last_results, self.current_folder, self.allowed_folders, user_tags, self.last_cohesive_bundles
            )

    def select_folder_dialog(self):
        folder = QFileDialog.getExistingDirectory(self, tr("action.select_folder"))
        if folder:
            self.set_active_folder(Path(folder))

    def set_active_folder(self, folder: Path):
        self.current_folder = folder
        self.lbl_current_folder.setText(str(folder))
        self.lbl_current_folder.setVisible(True)
        self.btn_select_dir.setVisible(True)
        self.preview_panel.close_preview()
        self.root_stack.setCurrentIndex(1)
        self.main_tabs.setCurrentIndex(0)
        self.start_scan()

    def start_scan(self):
        if not self.current_folder or not self.current_folder.exists():
            return

        self.preview_panel.close_preview()
        self.last_results.clear()
        self.last_cohesive_bundles.clear()
        self.organization_view.clear()
        self.tree_view.clear()
        self.allowed_folders.clear()

        self.progress_bar.setVisible(True)
        self.btn_cancel_scan.setVisible(True)

        self.worker = IndexWorker(self.current_folder)
        self.worker.progress_changed.connect(self.on_scan_progress)
        self.worker.file_classified.connect(self.on_file_classified)
        self.worker.scan_finished.connect(self.on_scan_finished)
        self.worker.error_occurred.connect(self.on_scan_error)
        self.worker.start()

    def cancel_scan(self):
        if self.worker:
            self.worker.cancel()
            self.lbl_status.setText(tr("status.scan_cancelled"))
            self.progress_bar.setVisible(False)
            self.btn_cancel_scan.setVisible(False)

    def on_scan_progress(self, current: int, total: int, file_name: str):
        if total > 0:
            self.progress_bar.setMaximum(total)
            self.progress_bar.setValue(current)
            self.lbl_status.setText(tr("status.scanning_file", current=current, total=total, file=file_name))
        else:
            self.lbl_status.setText(f"{file_name}")

    def on_file_classified(self, item: Dict[str, Any]):
        self.last_results.append(item)

    def on_scan_finished(self, summary: Dict[str, Any]):
        self.progress_bar.setVisible(False)
        self.btn_cancel_scan.setVisible(False)

        total = summary["total_files"]
        self.lbl_status.setText(tr("status.ready_processed", count=total, elapsed=summary["elapsed_seconds"]))

        # Calculate duplicates metrics
        dup_groups = summary.get("duplicates_groups", [])
        dup_bytes = sum(sum(it.get("size", 0) for it in g[1:]) for g in dup_groups) if dup_groups else 0
        self.last_duplicates_count = summary.get("duplicates", 0)
        self.last_duplicates_bytes = dup_bytes
        self.last_cohesive_bundles = summary.get("cohesive_bundles", [])

        # Update stats view
        self.stats_view.update_stats(self.last_results, self.last_duplicates_count, self.last_duplicates_bytes)

        user_tags = self.settings_mgr.get_user_tags()
        # Populate central Pre/Post organization view (Antes x Depois) with cohesive bundles
        self.organization_view.populate_results(
            self.last_results, self.current_folder, self.allowed_folders, user_tags, self.last_cohesive_bundles
        )

        # Populate side virtual tree
        self.tree_view.populate_results(self.last_results, user_tags)

        self.check_restore_availability()

    def on_bundle_action_changed(self, folder_rel: str, action: str):
        for b in self.last_cohesive_bundles:
            if b.get("folder_rel") == folder_rel:
                b["action"] = action
                break

    def show_shortcuts_guide(self):
        dlg = ShortcutsGuideDialog(self)
        dlg.exec()

    def on_folder_perm_toggled(self, folder_name: str, is_allowed: bool):
        if is_allowed:
            self.allowed_folders.add(folder_name)
        else:
            self.allowed_folders.discard(folder_name)

    def on_tag_renamed(self, tag_id: str, new_name: str):
        user_tags = self.settings_mgr.get_user_tags()
        self.tree_view.populate_results(self.last_results, user_tags)
        if self.current_folder:
            self.organization_view.populate_results(self.last_results, self.current_folder, self.allowed_folders, user_tags, self.last_cohesive_bundles)

    def on_file_reclassified(self, abs_path: str, new_tag_name: str, new_category: str):
        for it in self.last_results:
            if it.get("abs_path") == abs_path:
                if new_tag_name:
                    it["tag_name"] = new_tag_name
                if new_category:
                    it["category"] = new_category
                    it["caminho_fisico"] = new_category.replace(" ", "_")
                it["status"] = "identificado"
                it["confidence"] = 1.0
                primary_date = it.get("primary_date", "")
                ext = Path(abs_path).suffix.lower()
                rename_cfg = self.settings_mgr.data.get("configs", {})
                it["suggested_filename"] = generate_standard_filename(
                    primary_date, it.get("entity"), it.get("tag_name", ""), ext, Path(abs_path).stem, rename_cfg
                )
                break

        user_tags = self.settings_mgr.get_user_tags()
        if self.current_folder:
            self.organization_view.populate_results(self.last_results, self.current_folder, self.allowed_folders, user_tags, self.last_cohesive_bundles)
        self.tree_view.populate_results(self.last_results, user_tags)
        self.pending_view.populate_pending(self.last_results)
        self.stats_view.update_stats(self.last_results, self.last_duplicates_count, self.last_duplicates_bytes)
        self.lbl_status.setText(f"Tag atualizada para o arquivo: {Path(abs_path).name}")

    def on_file_marked_trash(self, abs_path: str):
        db_path = str(get_db_path())
        try:
            db = indexo_core.PyIndexoDatabase.open(db_path)
            for it in self.last_results:
                if it.get("abs_path") == abs_path:
                    it["status"] = "marked_for_deletion"
                    file_id = it.get("file_id")
                    if file_id:
                        db.mark_for_deletion(file_id, True)
                    break
        except Exception as e:
            logger.error("Failed to mark file for deletion: {}", e)

        user_tags = self.settings_mgr.get_user_tags()
        if self.current_folder:
            self.organization_view.populate_results(self.last_results, self.current_folder, self.allowed_folders, user_tags, self.last_cohesive_bundles)
        self.tree_view.populate_results(self.last_results, user_tags)
        self.pending_view.populate_pending(self.last_results)
        self.stats_view.update_stats(self.last_results, self.last_duplicates_count, self.last_duplicates_bytes)
        self.trash_view.load_trash()
        QMessageBox.information(self, "Lixeira", f"Arquivo '{Path(abs_path).name}' marcado para a Lixeira.")

    def on_file_selected_for_preview(self, abs_path: str):
        self.preview_panel.preview_file(abs_path)

    def execute_organization(self):
        """Organizes files physically into Indexo_Files after user confirmation, preserving cohesive bundles."""
        if not self.last_results or not self.current_folder:
            return

        eligible = [it for it in self.last_results if it.get("status") == "identificado"]
        if not eligible and not self.organization_view.cohesive_bundles:
            QMessageBox.information(self, "Indexo", "Nenhum arquivo identificado para organizar.")
            return

        count = len(eligible)
        bundles_to_move = [b for b in self.organization_view.cohesive_bundles if b.get("action") == "move_parent"]
        bundle_msg = f" incluindo {len(bundles_to_move)} pasta(s) integrada(s)" if bundles_to_move else ""
        msg = f"Deseja realmente organizar os {count} arquivos no disco{bundle_msg}?\n\nEsta operação moverá os arquivos para a pasta 'Indexo_Files' de forma segura com suporte a restauração (Desfazer)."
        reply = QMessageBox.question(
            self,
            "Confirmar Organização no Disco",
            msg,
            QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No
        )
        if reply != QMessageBox.StandardButton.Yes:
            return

        rename_enabled = bool(self.settings_mgr.get("rename_enabled", True))
        dest_root = self.current_folder / "Indexo_Files"

        moved_count = 0
        handled_bundle_folders = set()

        # 1. Move cohesive bundles whose action is "move_parent"
        for bundle in self.organization_view.cohesive_bundles:
            folder_rel = bundle.get("folder_rel", "")
            action = bundle.get("action", "move_parent")
            cat_name = bundle.get("category", "Outros")

            if action == "move_parent":
                src_folder = self.current_folder / folder_rel
                dest_folder = dest_root / cat_name / bundle.get("folder_name", folder_rel)
                if src_folder.exists() and "Indexo_Files" not in src_folder.parts:
                    if move_folder_safe(src_folder, dest_folder, self.current_folder):
                        moved_count += bundle.get("file_count", 1)
                        handled_bundle_folders.add(folder_rel)
            elif action == "keep":
                handled_bundle_folders.add(folder_rel)

        # 2. Move loose or disassembled files with confidence >= 0.65
        for it in self.last_results:
            conf = float(it.get("confidence", 0.0))
            status = it.get("status", "")
            if status == "identificado" and conf >= 0.65:
                bundle_folder = it.get("bundle_folder")
                if bundle_folder and bundle_folder in handled_bundle_folders:
                    continue

                src = Path(it["abs_path"])
                if not src.exists() or "Indexo_Files" in src.parts:
                    continue

                default_general = "Geral" if LanguageManager.get_instance().current_language == "ptBR" else "General"
                caminho_fisico = it.get("caminho_fisico") or it.get("category") or default_general
                dest_dir = dest_root / caminho_fisico
                
                target_name = it.get("suggested_filename") if rename_enabled else src.name
                resolved = indexo_core.py_resolve_collision(str(dest_dir), target_name or src.name)
                dest = Path(resolved)

                if move_file_safe(src, dest, self.current_folder):
                    moved_count += 1

        QMessageBox.information(
            self,
            tr("dialog.org_disk_completed_title"),
            tr("dialog.org_disk_completed_msg", count=moved_count, dest=str(dest_root))
        )
        self.check_restore_availability()
        self.start_scan()

    def check_restore_availability(self):
        if self.current_folder:
            restore_file = get_restore_path(self.current_folder)
            self.organization_view.btn_restore.setVisible(restore_file.exists())

    def restore_last_session_action(self):
        if not self.current_folder:
            return

        count, errors = restore_session(self.current_folder)
        if errors:
            QMessageBox.warning(self, "Aviso de Restauração", f"{count} arquivos restaurados.\nAvisos:\n" + "\n".join(errors[:5]))
        else:
            QMessageBox.information(self, tr("dialog.undo_completed_title"), tr("dialog.undo_completed_msg", count=count))

        self.check_restore_availability()
        self.start_scan()

    def on_trash_or_duplicate_changed(self):
        self.trash_view.load_trash()

    def on_promotion_suggested(self, tag_name: str, entity: str):
        msg = tr("dialog.promotion_prompt", tag=tag_name)
        reply = QMessageBox.question(self, tr("dialog.promotion_title"), msg, QMessageBox.StandardButton.Yes | QMessageBox.StandardButton.No)
        if reply == QMessageBox.StandardButton.Yes:
            new_tag = {
                "id": f"user_{tag_name.lower().replace(' ', '_')}",
                "nome": tag_name,
                "categoria": tag_name,
                "subcategoria": tag_name,
                "entidade": entity,
                "caminho_fisico": tag_name.replace(" ", "_"),
                "origem": "user",
                "idioma": LanguageManager.get_instance().current_language,
                "sinonimos": [],
                "palavras_chave": [tag_name.lower(), entity.lower()] if entity else [tag_name.lower()],
                "confianca_base": 1.0,
                "usar_para_automacao": True,
                "version": 1
            }
            self.settings_mgr.add_user_tag(new_tag)
            QMessageBox.information(self, tr("dialog.promotion_title"), tr("dialog.promotion_success", tag=tag_name))

    def on_file_selected_for_preview(self, abs_path: str):
        if not abs_path:
            return
        p = Path(abs_path)
        if not p.exists():
            return

        item_data = None
        for it in self.last_results:
            if it.get("abs_path") == abs_path:
                item_data = it
                break

        self.preview_panel.load_file(abs_path, item_data)

    def on_palette_folder_selected(self, folder_path: str):
        if not folder_path:
            return
        p = Path(folder_path)
        if p.exists() and p.is_dir():
            self.set_active_folder(p)

    def open_search_palette(self):
        dlg = SearchPaletteDialog(self.current_folder, self.last_results, self)
        dlg.folder_selected.connect(self.on_palette_folder_selected)
        dlg.file_selected.connect(self.on_file_selected_for_preview)
        dlg.exec()

    def change_theme(self, theme_name: str):
        self.settings_mgr.set("theme", theme_name)
        self.apply_theme()

    def change_font_size(self, size: int):
        self.settings_mgr.set("font_size", size)
        self.apply_theme()

    def apply_theme(self):
        theme = self.settings_mgr.get("theme", "system")
        font_size = int(self.settings_mgr.get("font_size", 15))
        apply_app_theme(None, theme, font_size)

    def change_language(self, lang: str):
        self.settings_mgr.set("language", lang)
        LanguageManager.get_instance().set_language(lang)
        self.repopulate_nav_list()
        self.btn_select_dir.setText(f"{tr('action.select_folder')} (Ctrl+O)")
        self.btn_search_palette.setText(f"{tr('search.placeholder')}")
        self.btn_settings_icon.setToolTip(f"{tr('nav.settings')} (Ctrl+,)")
        self.setWindowTitle(tr("app.title"))
        self.lbl_welcome_slogan.setText(tr("welcome.slogan"))
        self.lbl_welcome_prompt.setText(tr("welcome.select_prompt"))
        self.btn_welcome_select.setText(tr("welcome.btn_select"))
        # Propagate retranslation to sub-views
        self.settings_widget.retranslate_ui()
        self.organization_view.retranslate_ui()
        self.tag_manager_widget.retranslate_ui()
        
        self.btn_cancel_scan.setText(tr("action.cancel"))
        self.lbl_status.setText(tr("status.ready"))
        
        if self.current_folder:
            self.start_scan()
        QMessageBox.information(self, "Language / Idioma", "Language updated / Idioma atualizado!")

    def on_scan_error(self, err_msg: str):
        self.progress_bar.setVisible(False)
        self.btn_cancel_scan.setVisible(False)
        QMessageBox.critical(self, "Scan Error", f"An error occurred during indexing:\n{err_msg}")

    def closeEvent(self, event):
        """Cleanly shutdown threads, background workers, and flush database connections."""
        try:
            if hasattr(self, 'index_worker') and self.index_worker and self.index_worker.isRunning():
                self.index_worker.cancel()
                self.index_worker.wait(300)
            from PySide6.QtCore import QThreadPool
            QThreadPool.globalInstance().waitForDone(300)
        except Exception as e:
            logger.debug("Error during shutdown cleanup: {}", e)
        event.accept()
