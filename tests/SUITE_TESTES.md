# Suite de Testes Automatizados — Indexo

Este diretorio contem a bateria de testes automatizados do Indexo, cobrindo testes unitarios, testes de classificacao semantica com documentos reais, testes de seguranca de arquivos e integracao ponta a ponta (E2E).

---

## Estrutura da Suite

```text
tests/
├── SUITE_TESTES.md         # Este documento
├── conftest.py             # Configuracoes globais do pytest e fixtures
│
├── python/                 # Testes unitarios e funcionais em Python
│   ├── test_classification_golden.py      # Validacao da classificacao semantica de documentos
│   ├── test_i18n_and_rules.py             # Validacao de traducoes e integridade das regras
│   ├── test_index_worker.py               # Testes de processamento assincrono e streaming
│   ├── test_organization_view_and_search.py # Arvore virtual e busca indexada FTS5 (Ctrl+K)
│   ├── test_power_user_mode.py            # Modos de operacao, limites e automacao
│   ├── test_renaming_pattern.py           # Padroes de renomeacao e substituicao de tags
│   ├── test_restore_and_wal.py            # Persistencia de log WAL e reversao de sessao (Undo)
│   ├── test_safety_and_deletion.py        # Validacao do DELETE-ON-CONFIRM e lixeira segura
│   └── test_tag_manager.py                # Gerenciamento de tags e regras do usuario
│
├── integration/            # Testes de integracao End-to-End
│   └── test_end_to_end.py  # Ciclo completo: indexacao -> classificacao -> organizacao
│
└── fixtures/               # Conjuntos de dados estaticos e gerados
    └── sample_dataset/     # Base de arquivos sintetizada via scripts/generate_test_dataset.py
```

---

## Execucao dos Testes

### Executar a suite completa em Python:
```powershell
$env:PYTHONPATH="python-app"; python -m pytest tests/
```

### Executar testes de integracao:
```powershell
$env:PYTHONPATH="python-app"; python -m pytest tests/integration/
```

### Executar testes do motor em Rust:
```powershell
$env:PYO3_USE_ABI3_FORWARD_COMPATIBILITY="1"; cargo test --manifest-path rust-core/Cargo.toml
```

### Gerar ou restaurar a base de fixtures de teste:
```powershell
python scripts/generate_test_dataset.py
```
