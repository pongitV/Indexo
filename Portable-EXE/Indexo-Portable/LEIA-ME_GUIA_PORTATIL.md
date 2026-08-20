# Guia de Utilizacao da Versao Portatil — Indexo

Este documento fornece as instrucoes operacionais da versao portatil do Indexo, detalhando o armazenamento local de dados, a arquitetura de seguranca e as diretrizes de execucao.

---

## 1. Instalacao e Execucao

O Indexo e 100% portatil, dispensando instaladores, dependencias externas e alteracoes no Registro do Windows.

1. Extraia ou copie o conteudo da pasta do aplicativo para qualquer diretorio de sua preferencia (exemplo: `C:\Indexo` ou em uma unidade externa/pendrive).
2. Execute o arquivo `Indexo.exe`.

---

## 2. Localizacao e Persistencia de Dados

O Indexo opera de forma estritamente autocontida, nunca gravando arquivos em pastas do sistema como `%APPDATA%` ou `%LOCALAPPDATA%`.

Todos os dados gerados (banco de dados, regras, configuracoes e logs) residem no proprio diretorio do executavel:

```text
Indexo-Portable/
├── Indexo.exe               # Executavel da aplicacao
├── LEIA-ME_GUIA_PORTATIL.md # Instrucoes do usuario final
│
├── data/                    # Persistencia local
│   ├── indexo.db            # Banco SQLite FTS5 (historico e indice)
│   └── indexo.db.bak        # Backup de seguranca
│
├── configs/                 # Configuracoes e regras
│   ├── system_rules.json    # Regras mestras nativas
│   ├── user_rules.json      # Regras personalizadas do usuario
│   └── user_rules.bak.json  # Backup automatico de regras
│
└── indexo.log               # Registro de eventos e diagnostico
```

---

## 3. Principios de Operacao e Seguranca

- **Virtual-First (Visao sem Aplicacao)**: Por padrao, o Indexo nao move arquivos fisicamente. Os itens sao catalogados no banco de dados e organizados na Arvore Semantica Virtual e na Busca Global (`Ctrl+K`).
- **Movimentacao Autorizada**: A movimentacao fisica no disco e opcional e habilitada de forma granular pelo usuario por pasta de origem.
- **DELETE-ON-CONFIRM (Sem Exclusao Automatica)**: O aplicativo nao realiza delecoes automaticas. O processo de remocao exige confirmacao manual em duas etapas e utiliza a Lixeira nativa do Windows (`FOF_ALLOWUNDO`).
- **Undo Completo**: Toda operacao de movimentacao e registrada em log WAL (`.indexo_restore.json`), permitindo restauracao imediata por meio da funcionalidade "Restaurar Ultima Sessao".

---

## 4. Atalhos de Teclado

- `Ctrl + O`: Selecionar Pasta de Origem
- `Ctrl + K`: Paleta de Busca Global Instantanea (FTS5)
- `Ctrl + Enter`: Executar / Aplicar Organizacao
- `Ctrl + T`: Alternar Tema de Cores (Claro / Escuro / Sistema)
- `Ctrl + Shift + O`: Alternar Modo Somente Leitura
- `Ctrl + Z`: Desfazer Ultima Operacao (Undo)
