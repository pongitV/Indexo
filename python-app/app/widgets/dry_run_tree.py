import csv
from pathlib import Path
from typing import Dict, List, Any, Optional
from PySide6.QtCore import Qt, Signal
from PySide6.QtGui import QFont
from PySide6.QtWidgets import (
    QWidget, QVBoxLayout, QHBoxLayout, QLabel, QTreeWidget,
    QTreeWidgetItem, QPushButton, QFileDialog, QMessageBox, QSplitter
)
from app.i18n.language_manager import tr

class DryRunTreeView(QWidget):
    apply_clicked = Signal()
    undo_clicked = Signal()

    def __init__(self, parent=None):
        super().__init__(parent)
        self.plan_items: List[Dict[str, Any]] = []
        self.init_ui()

    def init_ui(self):
        layout = QVBoxLayout(self)
        layout.setContentsMargins(12, 12, 12, 12)
        layout.setSpacing(8)

        # Header with summary and action buttons
        header = QHBoxLayout()
        self.lbl_summary = QLabel("Prévia de Destinos: Antes → Depois")
        self.lbl_summary.setFont(QFont("Inter", 10, QFont.Weight.Bold))
        header.addWidget(self.lbl_summary)

        header.addStretch()
        self.btn_export = QPushButton("Exportar Relatório (CSV)")
        self.btn_export.clicked.connect(self.export_csv)
        header.addWidget(self.btn_export)

        layout.addLayout(header)

        # Splitter: Left = ANTES, Right = DEPOIS
        splitter = QSplitter(Qt.Orientation.Horizontal)

        # Left panel: ANTES
        left_widget = QWidget()
        left_layout = QVBoxLayout(left_widget)
        left_layout.setContentsMargins(0, 0, 0, 0)
        lbl_left = QLabel("ANTES (Origem)")
        lbl_left.setFont(QFont("Inter", 9, QFont.Weight.Bold))
        left_layout.addWidget(lbl_left)
        self.tree_before = QTreeWidget()
        self.tree_before.setHeaderHidden(True)
        left_layout.addWidget(self.tree_before)
        splitter.addWidget(left_widget)

        # Right panel: DEPOIS
        right_widget = QWidget()
        right_layout = QVBoxLayout(right_widget)
        right_layout.setContentsMargins(0, 0, 0, 0)
        lbl_right = QLabel("DEPOIS (Indexo_Files)")
        lbl_right.setFont(QFont("Inter", 9, QFont.Weight.Bold))
        right_layout.addWidget(lbl_right)
        self.tree_after = QTreeWidget()
        self.tree_after.setHeaderHidden(True)
        right_layout.addWidget(self.tree_after)
        splitter.addWidget(right_widget)

        layout.addWidget(splitter)

    def populate_dry_run(self, items: List[Dict[str, Any]], root_dir: Path):
        self.plan_items = items
        self.tree_before.clear()
        self.tree_after.clear()

        total = len(items)
        categories = set(it.get("category") for it in items if it.get("category"))
        self.lbl_summary.setText(f"{total} arquivos → {len(categories)} categorias de destino")

        # Populate ANTES tree (grouped by relative folder)
        root_before = QTreeWidgetItem(self.tree_before)
        root_before.setText(0, f"{root_dir.name} ({total})")
        root_before.setFont(0, QFont("Inter", 9, QFont.Weight.Bold))

        for it in items[:100]:  # Limit top items for fast rendering
            f_item = QTreeWidgetItem(root_before)
            f_item.setText(0, f"{it['rel_path']}")
        root_before.setExpanded(True)

        # Populate DEPOIS tree (grouped by destination folder)
        root_after = QTreeWidgetItem(self.tree_after)
        root_after.setText(0, f"Indexo_Files ({total})")
        root_after.setFont(0, QFont("Inter", 9, QFont.Weight.Bold))

        dest_groups: Dict[str, List[Dict[str, Any]]] = {}
        for it in items:
            dest_folder = it.get("caminho_fisico") or it.get("category") or "Geral"
            if dest_folder not in dest_groups:
                dest_groups[dest_folder] = []
            dest_groups[dest_folder].append(it)

        for folder_name, f_list in sorted(dest_groups.items()):
            folder_item = QTreeWidgetItem(root_after)
            folder_item.setText(0, f"{folder_name} ({len(f_list)})")
            folder_item.setFont(0, QFont("Inter", 9, QFont.Weight.Bold))

            for it in f_list[:20]:
                f_item = QTreeWidgetItem(folder_item)
                f_item.setText(0, f"{it['suggested_filename']}")
            folder_item.setExpanded(True)

        root_after.setExpanded(True)

    def export_csv(self):
        if not self.plan_items:
            QMessageBox.information(self, "Indexo", "Nenhum dado para exportar.")
            return

        file_path, _ = QFileDialog.getSaveFileName(self, "Salvar Relatório", "indexo_relatorio.csv", "CSV Files (*.csv)")
        if not file_path:
            return

        try:
            with open(file_path, "w", newline="", encoding="utf-8") as f:
                writer = csv.writer(f)
                writer.writerow(["Arquivo Origem", "Destino Planejado", "Nome Sugerido", "Categoria", "Confianca"])
                for it in self.plan_items:
                    writer.writerow([
                        it.get("rel_path"),
                        it.get("caminho_fisico"),
                        it.get("suggested_filename"),
                        it.get("category"),
                        f"{it.get('confidence', 0.0) * 100:.1f}%"
                    ])
            QMessageBox.information(self, "Indexo", f"Relatório exportado com sucesso em:\n{file_path}")
        except Exception as e:
            QMessageBox.critical(self, "Erro", f"Falha ao exportar CSV: {e}")
