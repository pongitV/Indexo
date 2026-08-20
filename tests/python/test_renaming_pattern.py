import pytest
from app.classification.entity_regex import (
    generate_standard_filename,
    format_custom_date,
    apply_casing
)

def test_format_custom_date():
    iso = "2024-05-10"
    assert format_custom_date(iso, "DD-MM-YYYY") == "10-05-2024"
    assert format_custom_date(iso, "YYYY-MM-DD") == "2024-05-10"
    assert format_custom_date(iso, "YYYY_MM_DD") == "2024_05_10"
    assert format_custom_date(iso, "YYYYMMDD") == "20240510"
    assert format_custom_date(iso, "DD_MM_YYYY") == "10_05_2024"
    assert format_custom_date(iso, "DDMMYYYY") == "10052024"
    assert format_custom_date(iso, "MM-DD-YYYY") == "05-10-2024"

def test_apply_casing():
    assert apply_casing("conta_de_luz", "title") == "Conta_De_Luz"
    assert apply_casing("conta de luz", "title") == "Conta De Luz"
    assert apply_casing("Conta_De_Luz", "lower") == "conta_de_luz"
    assert apply_casing("conta_de_luz", "upper") == "CONTA_DE_LUZ"
    assert apply_casing("Conta_De_Luz", "original") == "Conta_De_Luz"

def test_generate_standard_filename_patterns():
    # 1. New default: dash with spaces, suffix, DD-MM-YYYY, Title case
    cfg_default = {
        "rename_separator": " - ",
        "rename_date_position": "suffix",
        "rename_date_format": "DD-MM-YYYY",
        "rename_casing": "title"
    }
    assert generate_standard_filename("2024-05-10", "Enel", "Conta Luz", ".pdf", "documento", cfg_default) == "Enel - Conta Luz - 10-05-2024.pdf"
    # Without entity
    assert generate_standard_filename("2024-05-10", None, "Boleto", ".pdf", "doc", cfg_default) == "Boleto - 10-05-2024.pdf"

    # 2. Prefix with underscore and ISO date
    cfg_custom = {
        "rename_separator": "_",
        "rename_date_position": "prefix",
        "rename_date_format": "YYYY-MM-DD",
        "rename_casing": "title"
    }
    assert generate_standard_filename("2024-05-10", "Enel", "Conta Luz", ".pdf", "documento", cfg_custom) == "2024-05-10_Enel_Conta_Luz.pdf"

    # 3. No date with lowercase
    cfg_no_date = {
        "rename_separator": " - ",
        "rename_date_position": "none",
        "rename_date_format": "YYYY-MM-DD",
        "rename_casing": "lower"
    }
    assert generate_standard_filename("2024-05-10", "Enel", "Conta Luz", ".pdf", "documento", cfg_no_date) == "enel - conta luz.pdf"
