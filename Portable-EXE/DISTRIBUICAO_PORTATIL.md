# Distribuicao Portatil — Indexo Portable

Este diretorio contem os artefatos de distribuicao autossuficiente e portatil do Indexo para o sistema operacional Microsoft Windows.

---

## Estrutura do Pacote Distribuivel

```text
Portable-EXE/
├── DISTRIBUICAO_PORTATIL.md    # Este documento
├── INDEXO_PORTABLE_GUIDE.md    # Manual detalhado de uso e especificacoes portateis
└── Indexo-Portable/            # Diretorio empacotado para distribuicao (ZIP)
    ├── Indexo.exe              # Executavel principal standalone
    ├── LEIA-ME_GUIA_PORTATIL.md# Instrucoes de uso para o usuario final
    ├── resources/              # Regras do sistema, traducoes i18n e icones
    ├── configs/                # Regras customizadas do usuario (user_rules.json)
    └── data/                   # Banco de dados local SQLite (indexo.db)
```

---

## Instrucoes de Geracao do Pacote

Para gerar ou atualizar o pacote portatil:

```powershell
python scripts/build.py
```

O executavel final e todas as suas dependencias serao consolidados em `Portable-EXE/Indexo-Portable/`.
