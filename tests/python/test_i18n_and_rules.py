import pytest
from app.i18n.language_manager import LanguageManager, tr
from app.classification.rule_loader import RuleLoader

def test_i18n_keys_and_fallbacks():
    lm = LanguageManager.get_instance()
    
    lm.set_language("enUS")
    assert tr("app.title") == "Indexo — Semantic File Organizer"
    assert tr("action.organize") == "Organize"

    lm.set_language("ptBR")
    assert tr("app.title") == "Indexo — Sistema de Organização Semântica"
    assert tr("action.organize") == "Organizar"

def test_rule_loader_validation():
    loader = RuleLoader()
    assert len(loader.system_rules) >= 15
    json_str = loader.build_kernel_json()
    assert len(json_str) > 100
