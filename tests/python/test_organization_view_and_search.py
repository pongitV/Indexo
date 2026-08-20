import pytest
from pathlib import Path
import indexo_core
from app.i18n.language_manager import LanguageManager, tr
from app.config.settings_manager import SettingsManager
from app.classification.rule_loader import RuleLoader

def test_language_defaults_to_enus():
    lm = LanguageManager.get_instance()
    lm.set_language("enUS")
    assert lm.current_language == "enUS"
    assert tr("app.title") == "Indexo — Semantic File Organizer"
    assert tr("view.before_title") == "1. Current Structure (Before)"
    assert tr("view.after_title") == "2. Suggested Semantic Organization (After — Visual Only)"
    assert tr("type.document") == "Documents"
    assert tr("type.image") == "Photos and Images"

def test_language_ptbr_support():
    lm = LanguageManager.get_instance()
    lm.set_language("ptBR")
    assert lm.current_language == "ptBR"
    assert tr("app.title") == "Indexo — Sistema de Organização Semântica"
    assert tr("view.before_title") == "1. Estrutura Atual (Antes)"
    assert tr("view.after_title") == "2. Organização Semântica Sugerida (Depois — Visual Only)"
    assert tr("type.document") == "Documentos"
    assert tr("type.image") == "Fotos e Imagens"
    # Restore default
    lm.set_language("enUS")

def test_fts_multiword_portuguese_search(tmp_path):
    db_path = str(tmp_path / "test_fts_pt.db")
    db = indexo_core.PyIndexoDatabase.open(db_path)

    root_id = db.get_or_create_root(str(tmp_path), "test_root")
    db.upsert_file(root_id, "Faturas_e_Boletos/Energia_Eletrica/2024-05-10_Enel_Conta_Luz.pdf", "C:/test/2024-05-10_Enel_Conta_Luz.pdf", 1024, 1700000000, Some:=None, status="identificado")

    db.update_fts_content(
        "Faturas_e_Boletos/Energia_Eletrica/2024-05-10_Enel_Conta_Luz.pdf",
        "Faturas e Boletos Energia Elétrica",
        "Enel Distribuição",
        "kwh consumo ativo bandeira tarifaria 2024-05-10_Enel_Conta_Luz.pdf Faturas e Boletos",
        "Conta de luz Enel consumo 285 kWh total R$ 198,40",
        "",
        "",
        ""
    )

    # 1. Search by single word in Portuguese category
    res1 = db.search_fts("faturas")
    assert len(res1) >= 1
    assert "2024-05-10_Enel_Conta_Luz.pdf" in res1[0]["rel_path"]

    # 2. Search by multiple words (category + keyword)
    res2 = db.search_fts("conta luz")
    assert len(res2) >= 1
    assert "2024-05-10_Enel_Conta_Luz.pdf" in res2[0]["rel_path"]

    # 3. Search by subcategory
    res3 = db.search_fts("energia eletrica")
    assert len(res3) >= 1

    # 4. Search by filename prefix
    res4 = db.search_fts("enel")
    assert len(res4) >= 1
