# Distribuição Portátil — Indexo Portable

Este diretório contém os artefatos de distribuição 100% autossuficientes e portáteis do **Indexo** para o sistema operacional Microsoft Windows.

---

## Opções de Distribuição

### 1. Executável Único Standalone (Recomendado para uso rápido)
- **Arquivo:** `Portable-EXE/Indexo.exe`
- **Como usar:** Copie **apenas** o arquivo `Indexo.exe` para qualquer pasta, área de trabalho ou pendrive. Ao ser executado pela primeira vez, ele criará automaticamente suas pastas `data/` e `configs/` ao lado de si mesmo. Não requer nenhum outro arquivo externo.

### 2. Pacote Completo Estruturado (Recomendado para ZIP)
- **Pasta:** `Portable-EXE/Indexo-Portable/`
- **Estrutura:**
```text
Portable-EXE/
├── Indexo.exe                  # Executável Único Standalone
├── DISTRIBUICAO_PORTATIL.md    # Este documento
├── INDEXO_PORTABLE_GUIDE.md    # Manual detalhado de arquitetura portátil
└── Indexo-Portable/            # Pacote pronto para ZIP
    ├── Indexo.exe              # Executável principal
    ├── LEIA-ME_GUIA_PORTATIL.md# Instruções para o usuário final
    ├── resources/              # Regras nativas, ícones e traduções
    ├── configs/                # Regras personalizadas (criado no 1º uso)
    └── data/                   # Banco de dados SQLite (criado no 1º uso)
```

---

## Como Recompilar

Para recompilar a versão portátil após alterações no código:

```powershell
python scripts/build.py
```

O script compila o núcleo em Rust em modo Release, empacota com PyInstaller (`--onefile`) e atualiza tanto o executável único quanto a pasta de distribuição.
