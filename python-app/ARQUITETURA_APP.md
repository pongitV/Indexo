# Arquitetura da Aplicacao Desktop PyQt6 — Indexo

Este documento detalha a organizacao do pacote `app`, responsavel pela interface visual, gerenciamento de estado, controladores de fluxo e integracao assincrona com o motor nativo.

---

## Estrutura do Pacote `app`

```text
python-app/
├── main.py                     # Ponto de entrada, Single-Instance Lock e validacao de ambiente
├── ARQUITETURA_APP.md          # Este documento
└── app/
    ├── main_window.py          # Janela principal com layout em tres paineis
    │
    ├── classification/         # Carregamento e combinacao de regras do sistema e usuario
    │   └── rule_loader.py
    │
    ├── config/                 # Gerenciamento de configuracoes e persistencia local
    │   └── settings_manager.py
    │
    ├── extraction/             # Extratores em Python para formatos complexos
    │   ├── pdf_extractor.py    # Extracao vetorial e textual via PyMuPDF
    │   ├── office_extractor.py # Documentos DOCX, ODT e RTF
    │   └── ocr_extractor.py    # Processamento OCR em background
    │
    ├── i18n/                   # Modulo de internacionalizacao dinamica
    │   └── translator.py
    │
    ├── models/                 # Modelos de dados e estruturas de itens de arquivo
    │   └── file_item.py
    │
    ├── onboarding/             # Assistente de primeiro uso em tres etapas
    │   └── onboarding_wizard.py
    │
    ├── utils/                  # Utilitarios de sistema, seguranca, logs e reversao WAL
    │   ├── logger_setup.py
    │   ├── path_resolver.py
    │   ├── safety_guard.py
    │   └── undo_manager.py
    │
    ├── widgets/                # Componentes visuais PyQt6 reutilizaveis
    │   ├── organization_tree.py# Arvore Semantica Virtual
    │   ├── preview_panel.py    # Painel de pre-visualizacao (PDF, Imagens, Texto)
    │   ├── duplicates_dialog.py# Gerenciador de arquivos duplicados via SHA-256
    │   ├── trash_view.py       # Lixeira Virtual (DELETE-ON-CONFIRM)
    │   ├── search_palette.py   # Paleta de busca global rapida (Ctrl+K)
    │   ├── lite_mode.py        # Alternancia entre Modo Lite e Modo Avancado
    │   ├── folder_review.py    # Painel de revisao de pastas e regras de excecao
    │   └── toast.py            # Notificacoes toast / snackbar
    │
    ├── workers/                # QThreads para processamento em segundo plano
    │   ├── index_worker.py     # Worker de indexacao, extracao e streaming progressivo
    │   └── apply_worker.py     # Worker de execucao de operacoes fisicas no disco
    │
    └── resources/styles/       # Folhas de estilo QSS (Temas Flexoki Claro e Escuro)
        ├── theme_dark.qss
        └── theme_light.qss
```

---

## Padroes de Implementacao

- **Assincronismo**: Toda operacao de I/O pesado (leitura de disco, calculo de hashes, parsing de PDF e gravacao em banco) e executada em `QThread` dedicada (`index_worker.py`, `apply_worker.py`), mantendo a interface grafica permanentemente responsiva a 60 FPS.
- **Isolamento de Estado**: As configuracoes e o banco de dados residem no diretorio de execucao, preservando a portabilidade total da aplicacao.
