import pytest
import json
from pathlib import Path
import indexo_core

from app.classification.rule_loader import RuleLoader
from app.classification.entity_regex import (
    extract_primary_date, extract_amount, extract_due_date,
    extract_keywords_from_text, generate_standard_filename
)

from app.config.settings_manager import SettingsManager

@pytest.fixture
def kernel():
    mgr = SettingsManager()
    mgr.data["tags"] = []
    mgr.save_data()
    loader = RuleLoader()
    rules_json = loader.build_kernel_json()
    return indexo_core.PyClassificationKernel.from_rules_json(rules_json)

def test_golden_boleto_44_digits(kernel):
    text = "Comprovante de pagamento Banco Santander 03399654321098765432109876543210987654321098 Vencimento 20/08/2026 Valor R$ 350,00"
    res = kernel.classify(text, ".pdf", 0.0)
    assert res is not None
    assert "boleto" in res["tag_id"].lower() or "fatura" in res["categoria"].lower()
    assert res["confianca"] >= 0.85

def test_golden_conta_luz_aes_sul(kernel):
    text = "AES SUL DISTRIBUIDORA DE ENERGIA S.A. CONTA DE LUZ - ENERGIA ELETRICA Consumo ativo: 280 kWh Vencimento: 15/08/2026"
    res = kernel.classify(text, ".pdf", 0.0)
    assert res is not None
    assert "luz" in res["nome"].lower() or "energia" in res["nome"].lower()
    assert res["confianca"] >= 0.85

def test_golden_conta_agua_corsan(kernel):
    text = "CORSAN - Companhia Riograndense de Saneamento CONTA DE AGUA E ESGOTO Hidrometro A1234 Leitura: 145 m3 Vencimento: 10/08/2026"
    res = kernel.classify(text, ".pdf", 0.0)
    assert res is not None
    assert "agua" in res["nome"].lower() or "água" in res["nome"].lower()
    assert res["confianca"] >= 0.85

def test_golden_darf_receita_federal(kernel):
    text = "MINISTERIO DA FAZENDA - SECRETARIA DA RECEITA FEDERAL DO BRASIL DARF - DOCUMENTO DE ARRECADACAO DE RECEITAS FEDERAIS Codigo da Receita: 0561"
    res = kernel.classify(text, ".pdf", 0.0)
    assert res is not None
    assert "darf" in res["nome"].lower() or "darf" in res["tag_id"].lower()
    assert res["confianca"] >= 0.85

def test_golden_holerite_salario(kernel):
    text = "DEMONSTRATIVO DE PAGAMENTO DE SALARIO - HOLERITE Salario Base: R$ 4.500,00 Descontos INSS: R$ 450,00 FGTS do mes: R$ 360,00 Liquido a receber: R$ 3.800,00"
    res = kernel.classify(text, ".pdf", 0.0)
    assert res is not None
    assert "holerite" in res["nome"].lower() or "contracheque" in res["nome"].lower()
    assert res["confianca"] >= 0.85

def test_fuzzy_ocr_resilience(kernel):
    # Simulates OCR typo where 'demonstrativo' is read as 'demonstrativ0'
    text = "DEMONSTRATIV0 DE PAGAMENTO - H0LERITE Salario Liquido: R$ 3.200,00"
    res = kernel.classify(text, ".pdf", 0.0)
    assert res is not None
    assert "holerite" in res["nome"].lower() or "contracheque" in res["nome"].lower()

def test_smart_metadata_extraction():
    text = "ENEL DISTRIBUICAO SP - CONTA DE ENERGIA ELETRICA\nVencimento: 15/06/2024\nVALOR A PAGAR: R$ 185,50\nCNPJ: 61.695.227/0001-93"
    
    amount = extract_amount(text)
    assert amount == "R$185,50"

    due = extract_due_date(text)
    assert due == "15/06/2024"

    date_iso = extract_primary_date(text)
    assert date_iso == "2024-06-15"

    fn = generate_standard_filename(date_iso, "Enel", "Conta de Luz", ".pdf", amount=amount)
    assert "185,50" in fn
    assert "Enel" in fn

def test_keyword_auto_learning():
    sample_text = "Contrato de prestacao de servicos juridicos de assessoria tributaria e consultoria societaria com honorarios advocaticios"
    kws = extract_keywords_from_text(sample_text, "contrato_assessoria.pdf")
    assert len(kws) > 0
    assert any(k in ["juridicos", "assessoria", "tributaria", "prestacao", "servicos"] for k in kws)
