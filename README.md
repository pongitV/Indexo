# Indexo — Sistema Inteligente de Organização Semântica de Arquivos

<p align="center">
  <img src="src-tauri/icons/icon.png" width="128" height="128" alt="Indexo Logo">
</p>

<p align="center">
  <b>Organizador e indexador de arquivos semântico, inteligente, portátil e 100% offline para Windows.</b><br>
  Construído com arquitetura de alta performance: <b>Rust nativo (Tauri 2)</b> no backend e <b>Svelte 5 (TypeScript)</b> no frontend.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011%20x64-blue.svg" alt="Platform">
  <img src="https://img.shields.io/badge/Backend-Rust%202021%20%7C%20Tauri%202-orange.svg" alt="Backend">
  <img src="https://img.shields.io/badge/Frontend-Svelte%205%20%7C%20TypeScript-red.svg" alt="Frontend">
  <img src="https://img.shields.io/badge/Engine-OCR%20%2B%20384D%20Dense%20Neural%20Classifier-purple.svg" alt="Engine">
  <img src="https://img.shields.io/badge/Privacy-100%25%20Offline%20%26%20Local-success.svg" alt="Privacy">
  <img src="https://img.shields.io/badge/License-GNU%20GPLv3-yellow.svg" alt="License">
</p>

<p align="center">
  <b>Português</b> | <a href="README_EN.md">English</a>
</p>

> [!NOTE]
> **Versão em Python disponível**: Procurando pela versão anterior construída em Python (PySide6 / Qt)? Ela está totalmente preservada, funcional e disponível no repositório dedicado [`Indexo-py`](https://github.com/pongitV/Indexo-py).

---

## Sumário

- [Sobre o Projeto](#sobre-o-projeto)
- [Principais Destaques](#principais-destaques)
- [Motor de Classificação Semântica em 3 Camadas](#motor-de-classificação-semântica-em-3-camadas)
- [Arquitetura do Sistema](#arquitetura-do-sistema)
- [Estrutura Completa do Repositório](#estrutura-completa-do-repositório)
- [Fluxo de Experiência do Usuário (UI/UX)](#fluxo-de-experiência-do-usuário-uiux)
- [Como Executar e Desenvolver](#como-executar-e-desenvolver)
- [Gerando o Executável Portátil Standalone](#gerando-o-executável-portátil-standalone)
- [Roteiro de Desenvolvimento](#roteiro-de-desenvolvimento)
- [Licença](#licença)

---

## Sobre o Projeto

O **Indexo** é um organizador e classificador semântico de arquivos projetado para resolver a desordem crônica de diretórios complexos no Windows (como pastas *Downloads*, *Documentos*, *Área de Trabalho* ou coleções de projetos).

Diferente de organizadores tradicionais baseados em extensões fixas ou regras manuais rígidas, o Indexo opera sob o princípio de **inteligência adaptativa sem taxonomia pré-definida (zero-hardcode)**: ele analisa o conteúdo real dos arquivos (inspecionando *magic numbers*, executando OCR nativo de imagens, extraindo textos de PDFs/Office e avaliando similaridade vetorial densa) e agrupa tudo dinamicamente em categorias legíveis e naturais.

### Pilares Fundamentais:
1. **100% Offline & Privado**: Nenhum arquivo, metadado ou dado de telemetria sai da máquina do usuário.
2. **Alta Performance & Baixo Consumo**: Backend compilado em **Rust 2021** com paralelismo real via `rayon`, processando milhares de arquivos sem travamentos e consumindo mínima memória RAM.
3. **Interface Moderna & Reativa**: Frontend construído com **Svelte 5** e **TypeScript**, proporcionando uma experiência desktop fluida, suporte a temas (claro/escuro) e internacionalização (pt-BR / en-US).
4. **Segurança Absoluta (Zero Risco de Perda de Dados)**: Arquivos nunca são deletados automaticamente — apenas organizados após confirmação visual lado a lado, sempre acompanhados de histórico transacional para **desfazer (undo) em 1 clique**.

---

## Principais Destaques

* **OCR Nativo de Imagens e Scans (Windows.Media.Ocr)**: Lê texto diretamente de capturas de tela, fotos de recibos, comprovantes PIX e documentos escaneados (`.png`, `.jpg`, `.jpeg`, `.webp`, `.bmp`, `.tiff`) usando as APIs nativas do Windows sem adicionar dependências externas ou downloads pesados.
* **Embeddings Neurais Densos de 384 Dimensões**: Vetorização semântica contínua em espaço de 384 dimensões combinando projeções hiperdimensionais de subpalavras com 64 âncoras temáticas latentes e clusterização por centróides adaptativos.
* **Zero-Hardcode & Categorização Dinâmica**: O aplicativo não impõe regras engessadas de fábrica. As categorias e tags são sintetizadas em tempo real a partir dos arquivos reais do usuário.
* **Subcategorias Inteligentes e Hierárquicas**: Quando 2 ou mais arquivos da mesma categoria compartilham uma entidade (jogos como *Zelda*, *Minecraft*; empresas como *Enel*, *Nubank*; ou assuntos como *Praia*, *Projetos*), o Indexo cria automaticamente subpastas profundas (ex: `Fotos e Imagens/Jogos/Zelda` ou `Boletos e Faturas/Enel`).
* **Detecção e Preservação de Pastas Já Organizadas**: Subpastas que já possuem estrutura coerente (como `Fotos de Férias/Praia/`) são identificadas automaticamente, preservadas por padrão com indicador visual SVG no preview e opções rápidas de manter ou reorganizar.
* **Renomeador Inteligente em Lote**: Padroniza e higieniza nomes de arquivos em lote, removendo ruídos de câmeras/apps (`IMG_`, `WA_`, `Scan_`), extraindo datas formatadas, herdando assuntos de subcategorias e preservando sequenciais numéricos (`_01`, `_02`).
* **Inspeção Real de Conteúdo**: Não confia cegamente em extensões declaradas. Detecta o formato real pelos bytes de cabeçalho (*magic numbers*) e extrai texto de PDFs, DOCX, XLSX, TXT, MD e CSV.
* **Pré-visualização Lado a Lado (Antes x Depois)**: Comparação visual clara entre a estrutura original e a proposta de destino antes de qualquer operação no disco.
* **Aprendizado Incremental por Correção Manual**: Permite corrigir classificações com o botão direito (alterar categoria, criar nova tag, criar regra permanente). Cada correção alimenta o banco SQLite local (`data/profile.db`), aprimorando classificações futuras.
* **Gerenciador de Tags e Categorias**: Interface dedicada para criar, renomear, mesclar, auditar e expurgar tags/categorias automáticas ou não utilizadas com otimização `VACUUM`.
* **Reversão Completa de Sessão (Undo)**: Trilha de auditoria transacional para reverter qualquer organização ou renomeação com restauração precisa em 1 clique.
* **100% Portátil (.exe Standalone)**: Roda diretamente sem instalador, sem tocar no `%APPDATA%` ou no Registro do Windows. Copiar a pasta do executável copia todo o perfil de aprendizado.

---

## Motor de Classificação Semântica em 3 Camadas

O pipeline de classificação do Indexo avalia cada arquivo em três camadas hierárquicas em cascata:

```mermaid
flowchart TD
    A["Arquivo Selecionado"] --> P{"Já está em pasta organizada?"}
    P -->|"Sim"| PRESERV["Preserva Estrutura Original (Badge de Revisão)"]
    P -->|"Não"| B["Camada 1: Heurísticas & Assinatura de Bytes"]
    B --> Z{"Confiança >= 80%?"}
    Z -->|"Sim"| R1["Classificação Rápida (0ms)"]
    Z -->|"Não"| C["Camada 2: OCR Nativo, Extração Textual & Embeddings 384D"]
    C --> Y{"Confiança >= 70%?"}
    Y -->|"Sim"| R2["Classificação Semântica (~5ms)"]
    Y -->|"Não"| D["Camada 3: Raciocínio Profundo SLM Local"]
    D --> R3["Classificação Profunda Local"]
    R1 --> SUB["Motor de Subcategorias (Jogos / Empresas / Assuntos)"]
    R2 --> SUB
    R3 --> SUB
```

1. **Camada 1 — Heurísticas e Assinaturas Reais de Bytes (`0ms`)**:
   - Detecção do MIME-type real através de *magic numbers* via `infer`.
   - Consulta ao banco local de regras personalizadas e histórico de correções do usuário (`profile.db`).
   - Resolução direta de mídias conhecidas e instaladores.
2. **Camada 2 — OCR Nativo, Extração Textual & Embeddings 384D (`~5ms`)**:
   - Reconhecimento óptico de caracteres (OCR) nativo via `Windows.Media.Ocr` para fotos de comprovantes, prints e imagens com texto.
   - Extração de texto estruturado de documentos (`pdf-extract`, `docx-rs`, `calamine`).
   - Vetorização semântica em espaço denso de 384 dimensões com 64 âncoras conceituais bilíngues e clusterização adaptativa por centróides.
3. **Camada 3 — Raciocínio Profundo com SLM Local**:
   - Síntese semântica de nomes de categorias com filtro de ruídos e fallback refinado para *"Documentos Diversos"*.
4. **Motor de Subcategorias Hierárquicas**:
   - Agrupa arquivos afins dentro da mesma categoria principal em subpastas multiníveis (`Fotos e Imagens/Jogos/Zelda`, `Boletos e Faturas/Enel`).

---

## Arquitetura do Sistema

```mermaid
flowchart TB
    subgraph FE ["Frontend (Svelte 5 / TypeScript)"]
        UI["App.svelte (Navegação & Telas)"]
        ST["Stores Reativos (Estado Global)"]
        TREE["FileTreeNode (Árvore Interativa)"]
    end

    subgraph IPC ["Camada IPC (Tauri 2 Bridge)"]
        CMD_SCAN["scan_folder / scan_specific_files"]
        CMD_CLASS["classify_scanned_files"]
        CMD_APPLY["apply_organization / undo_last_apply"]
        CMD_RENAME["suggest_semantic_names / apply_renames"]
        CMD_PROF["profile / rules / maintenance"]
    end

    subgraph BE ["Backend Nativo (Rust 2021)"]
        SCANNER["Scanner Paralelo (walkdir + rayon)"]
        EXTRACT["Extração de Texto (PDF / DOCX / XLSX)"]
        ENGINE["Motor Semântico & Subcategorias"]
        RENAMER["Motor do Renomeador Inteligente"]
        MOVER["Operações Seguras & Log de Auditoria"]
        DB[("SQLite Local (data/profile.db)")]
    end

    UI --> ST
    ST --> CMD_SCAN
    ST --> CMD_CLASS
    ST --> CMD_RENAME
    TREE --> CMD_APPLY
    UI --> CMD_PROF

    CMD_SCAN --> SCANNER
    CMD_CLASS --> EXTRACT
    EXTRACT --> ENGINE
    CMD_CLASS --> ENGINE
    CMD_RENAME --> RENAMER
    CMD_APPLY --> MOVER
    CMD_PROF --> DB

    ENGINE --> DB
    MOVER --> DB
    SCANNER --> DB
```

---

## Estrutura Completa do Repositório

```text
Indexo/
├── index.html                      # Ponto de entrada HTML do frontend Vite
├── package.json                    # Dependências e scripts do frontend Svelte 5 / Tauri
├── package-lock.json               # Trava de dependências Node.js
├── tsconfig.json                   # Configuração do compilador TypeScript
├── vite.config.ts                  # Configuração do Vite e plugin Svelte
├── README.md                       # Apresentação do projeto e guia do usuário (Português)
├── README_EN.md                    # Apresentação do projeto e guia do usuário (Inglês)
├── LICENSE                         # Licença GNU General Public License v3.0
├── .gitignore                      # Regras de exclusão do Git
├── Indexo.exe                      # Executável portátil compilado standalone
│
├── src/                            # Frontend em Svelte 5 + TypeScript
│   ├── main.ts                     # Inicialização da aplicação Svelte
│   ├── App.svelte                  # Componente raiz e orquestrador de navegação
│   │
│   ├── lib/                        # Módulos compartilhados e componentes reativos
│   │   ├── api.ts                  # Clientes e chamadas tipadas para comandos Tauri
│   │   ├── stores.ts               # Gerenciamento de estado reativo global (Svelte stores)
│   │   ├── FileTreeNode.svelte     # Árvore visual de arquivos com suporte a subpastas e badges SVG
│   │   └── i18n/                   # Dicionários de internacionalização
│   │       ├── pt-BR.json          # Português do Brasil
│   │       └── en-US.json          # Inglês Americano
│   │
│   ├── routes/                     # Telas e fluxos da aplicação
│   │   ├── FolderSelect.svelte     # Seleção e drag-and-drop de pastas ou arquivos
│   │   ├── Scanning.svelte         # Progresso de varredura e extração de conteúdo
│   │   ├── Preview.svelte          # Pré-visualização lado a lado (Antes x Depois) e menu de contexto
│   │   ├── Renamer.svelte          # Renomeador inteligente em lote com presets semânticos
│   │   ├── TagManager.svelte       # Gerenciador de tags e regras aprendidas
│   │   ├── CategoryManager.svelte  # Gerenciador de categorias e ferramentas de expurgo
│   │   └── Settings.svelte         # Painel de configurações (tema, idioma, limites)
│   │
│   ├── i18n/                       # Configuração de internacionalização (svelte-i18n)
│   │   └── setup.ts                # Inicialização e detecção do idioma do sistema
│   │
│   └── styles/                     # Estilização global
│       └── theme.css               # Design system, tokens de cores e temas escuro/claro
│
├── src-tauri/                      # Backend nativo em Rust (Tauri 2)
│   ├── Cargo.toml                  # Manifesto de dependências Rust (tauri, tokio, rusqlite, rayon, infer)
│   ├── Cargo.lock                  # Trava de dependências Rust
│   ├── build.rs                    # Script de compilação Tauri
│   ├── tauri.conf.json             # Configurações do runtime Tauri 2 e janelas
│   │
│   ├── src/                        # Código-fonte Rust
│   │   ├── main.rs                 # Ponto de entrada do executável e registro de comandos
│   │   │
│   │   ├── commands/               # Handlers de comandos invocados pelo frontend
│   │   │   ├── mod.rs              # Exportação dos módulos de comandos
│   │   │   ├── scan.rs             # Varredura recursiva de diretórios e arquivos específicos
│   │   │   ├── classify.rs         # Classificação semântica em lote e roteamento de camadas
│   │   │   ├── apply.rs            # Movimentação atômica e reversão transacional (Undo)
│   │   │   ├── rename.rs           # Geração de sugestões e aplicação de renomeação em lote
│   │   │   ├── profile.rs          # Gerenciamento de categorias, regras e limpeza do banco
│   │   │   └── system.rs           # Abertura de caminhos no Explorador de Arquivos do Windows
│   │   │
│   │   ├── engine/                 # Núcleo do motor de inteligência e classificação
│   │   │   ├── mod.rs              # Orquestrador do pipeline de classificação e testes unitários
│   │   │   ├── heuristics.rs       # Camada 1: Heurísticas, magic numbers e detecção de pastas organizadas
│   │   │   ├── ocr.rs              # OCR nativo de alta velocidade via Windows.Media.Ocr
│   │   │   ├── content_extract.rs  # Extração segura de texto de PDF, DOCX, XLSX e despacho para OCR
│   │   │   ├── embeddings.rs       # Camada 2: Embeddings neurais densos 384D com 64 âncoras temáticas e centróides
│   │   │   ├── subcategories.rs    # Motor de subcategorias hierárquicas por jogos, empresas e assuntos
│   │   │   ├── llm_local.rs        # Camada 3: Nomenclatura semântica e filtro de ruídos binários
│   │   │   ├── renamer.rs          # Motor do renomeador inteligente e resolução de colisões
│   │   │   └── rules.rs            # Avaliador e sintetizador dinâmico de regras aprendidas
│   │   │
│   │   ├── fs_ops/                 # Operações seguras no sistema de arquivos
│   │   │   ├── mod.rs              # Tratamento de caminhos e prevenção anti-traversal
│   │   │   └── mover.rs            # Movimentação segura, resolução de colisão e rollback
│   │   │
│   │   └── db/                     # Camada de banco de dados SQLite local
│   │       ├── mod.rs              # Conexão, queries, manutenção e transações em data/profile.db
│   │       ├── models.rs           # Estruturas de dados (Category, Rule, ActionLog, Session)
│   │       └── schema.sql          # Esquema relacional inicial com tabelas e índices
│   │
│   ├── capabilities/               # Políticas de segurança e permissões do Tauri 2
│   │   └── default.json            # Permissões de diálogo, sistema de arquivos e IPC
│   │
│   └── icons/                      # Ícones da aplicação em múltiplos formatos
│       ├── icon.ico                # Ícone do executável Windows
│       ├── icon.png                # Ícone padrão em alta resolução
│       └── ...                     # Ícones para resoluções variadas
│
└── data/                           # Diretório de persistência local (criado em runtime)
    └── profile.db                  # Banco SQLite do perfil de aprendizado do usuário
```

---

## Fluxo de Experiência do Usuário (UI/UX)

1. **Seleção de Pasta ou Arquivos**: O usuário abre o aplicativo e seleciona pastas ou arquivos específicos via diálogo ou arrastar e soltar.
2. **Varredura e Extração**: O motor em Rust varre diretórios em paralelo, detecta *magic numbers* e extrai textos em milissegundos, com feedback de progresso em tempo real.
3. **Pré-visualização Interativa Multinível**: A tela exibe a árvore **Antes x Depois** com suporte a subcategorias hierárquicas profundas e identificação de pastas preservadas.
4. **Renomeação Semântica Integrada**: Com um simples toggle, o usuário pode visualizar e padronizar nomes de arquivos com datas, categorias e assuntos.
5. **Aplicação com Segurança Absoluta**: Ao clicar em **Aplicar**, os arquivos são movidos atomicamente para a nova estrutura organizada, gerando a trilha de auditoria para reversão.
6. **Desfazer em 1 Clique**: A qualquer momento, o botão **Desfazer (Undo)** reverte a última sessão de organização de forma 100% fiel.

---

## Como Executar e Desenvolver

### Pré-requisitos
* **Windows 10 ou 11 (64-bit)**
* **Rust Toolchain** (`rustc` e `cargo` instalados via [rustup.rs](https://rustup.rs))
* **Node.js 18+** e **npm** ([nodejs.org](https://nodejs.org))

### 1. Clonar o Repositório

```powershell
git clone https://github.com/pongitV/Indexo.git
cd Indexo
```

### 2. Instalar Dependências do Frontend

```powershell
npm install
```

### 3. Executar em Modo de Desenvolvimento

```powershell
npm run tauri dev
```

Este comando iniciará o servidor de desenvolvimento Vite e a janela nativa do Tauri conectada ao backend Rust com *Hot Module Replacement (HMR)*.

---

## Gerando o Executável Portátil Standalone

Para compilar a aplicação em modo release otimizado e gerar o `.exe` autônomo:

```powershell
npm run tauri build
```

O binário portátil standalone será gerado em:
`src-tauri/target/release/indexo.exe`

> Este arquivo é 100% autossuficiente: basta copiar o `indexo.exe` para qualquer pasta ou pendrive e executá-lo diretamente.

---

## Roteiro de Desenvolvimento

* **Fase 0**: Fundação da stack (Rust + Tauri 2 + Svelte 5 + SQLite `profile.db`).
* **Fase 1**: Extração de conteúdo e motor heurístico de classificação semântica.
* **Fase 2**: Interface reativa, árvore de pré-visualização lado a lado e fluxo de aprovação.
* **Fase 3**: Movimentação física segura, reversão transacional (Undo) e empacotamento portátil.

---

## Licença

Este projeto é software livre e de código aberto, distribuído sob a licença **GNU General Public License v3.0 (GPLv3)**. Consulte o arquivo [LICENSE](LICENSE) para mais detalhes.
