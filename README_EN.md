# Indexo — Intelligent Semantic File Organization System

<p align="center">
  <img src="src-tauri/icons/icon.png" width="128" height="128" alt="Indexo Logo">
</p>

<p align="center">
  <b>Semantic, intelligent, portable, and 100% offline file organizer and indexer for Windows.</b><br>
  Built with a high-performance architecture: <b>Native Rust (Tauri 2)</b> backend and <b>Svelte 5 (TypeScript)</b> frontend.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011%20x64-blue.svg" alt="Platform">
  <img src="https://img.shields.io/badge/Backend-Rust%202021%20%7C%20Tauri%202-orange.svg" alt="Backend">
  <img src="https://img.shields.io/badge/Frontend-Svelte%205%20%7C%20TypeScript-red.svg" alt="Frontend">
  <img src="https://img.shields.io/badge/Engine-3--Tier%20Adaptive%20Semantic%20Classifier-purple.svg" alt="Engine">
  <img src="https://img.shields.io/badge/Privacy-100%25%20Offline%20%26%20Local-success.svg" alt="Privacy">
  <img src="https://img.shields.io/badge/License-GNU%20GPLv3-yellow.svg" alt="License">
</p>

<p align="center">
  <a href="README.md">Português</a> | <b>English</b>
</p>

> [!NOTE]
> **Python version available**: Looking for the previous implementation built in Python (PySide6 / Qt)? It is fully preserved, functional, and available in the dedicated repository [`Indexo-py`](https://github.com/pongitV/Indexo-py).

---

## Table of Contents

- [About the Project](#about-the-project)
- [Key Highlights](#key-highlights)
- [3-Tier Semantic Classification Engine](#3-tier-semantic-classification-engine)
- [System Architecture](#system-architecture)
- [Complete Repository Structure](#complete-repository-structure)
- [User Experience Flow (UI/UX)](#user-experience-flow-uiux)
- [Getting Started & Development](#getting-started--development)
- [Building the Standalone Portable Executable](#building-the-standalone-portable-executable)
- [Development Roadmap](#development-roadmap)
- [License](#license)

---

## About the Project

**Indexo** is a semantic file organizer and classifier engineered to resolve chronic clutter across complex Windows directory structures (such as *Downloads*, *Documents*, *Desktop*, or scattered project folders).

Unlike conventional file organizers that rely on fixed extensions or rigid manual rules, Indexo operates under the principle of **adaptive intelligence without predefined taxonomies (zero-hardcode)**: it analyzes the genuine content of files (inspecting *magic numbers*, extracting text from PDFs/Office files, and evaluating semantic similarity) and dynamically clusters files into natural, human-readable categories.

### Core Principles:
1. **100% Offline & Private**: Zero user data, metadata, or telemetry ever leaves your machine.
2. **High Performance & Low Memory Footprint**: Backend compiled in **Rust 2021** with true multithreading via `rayon`, scanning thousands of files seamlessly with minimal RAM consumption.
3. **Modern & Reactive UI**: Frontend built with **Svelte 5** and **TypeScript**, providing a fluid desktop experience, theme switching (dark/light), and internationalization (pt-BR / en-US).
4. **Absolute Safety (Zero Data Loss Risk)**: Files are never deleted automatically — only moved after interactive side-by-side visual inspection, accompanied by a transactional log for **1-click undo**.

---

## Key Highlights

* **Zero-Hardcode & Dynamic Categorization**: No static factory taxonomies. Categories and tags are synthesized in real-time based on the user's actual files.
* **Intelligent Hierarchical Subcategories**: When 2 or more files in the same category share an entity (games like *Zelda*, *Minecraft*; companies like *Enel*, *Nubank*; or subjects like *Beach*, *Projects*), Indexo automatically creates deep subfolders (e.g., `Images/Games/Zelda` or `Bills & Invoices/Enel`).
* **Detection & Preservation of Organized Folders**: Subfolders with existing coherent structure (such as `Vacation Photos/Beach/`) are automatically detected, preserved by default with an SVG review indicator in preview, and offer quick toggle options to keep or reorganize.
* **Batch Smart Renamer**: Standardizes and cleans file names in batch, removing camera/app noise prefixes (`IMG_`, `WA_`, `Scan_`), extracting formatted dates, inheriting subcategory entities, and preserving numerical series sequences (`_01`, `_02`).
* **Genuine Content Inspection**: Never trusts declared file extensions blindly. Identifies authentic formats via header bytes (*magic numbers*) and extracts text from PDF, DOCX, XLSX, TXT, MD, and CSV.
* **Side-by-Side Visual Preview (Before vs. After)**: Clear visual comparison between current directory trees and proposed multi-level target paths before any filesystem operation occurs.
* **Incremental Learning via Manual Corrections**: Correct any classification with right-click context actions (change category, create new tag, create permanent rule). Every correction updates the local SQLite profile (`data/profile.db`), enhancing future classification accuracy.
* **Tag & Category Manager**: Dedicated interface to create, rename, merge, audit, and purge automated or unused tags/categories with SQLite `VACUUM` optimization.
* **Full Session Reversion (Undo)**: Transactional audit log allows users to revert any completed organization or renaming session with 1-click precision.
* **100% Portable (.exe Standalone)**: Operates standalone without installers, registry keys, or `%APPDATA%` pollution. Moving the application folder preserves the entire learned profile.

---

## 3-Tier Semantic Classification Engine

Indexo evaluates every file through a 3-tier hierarchical cascaded pipeline:

```mermaid
graph TD
    A[Selected File] --> P{Already in Organized Folder?}
    P -->|Yes| PRESERV[Preserve Original Structure with Review Badge]
    P -->|No| B[Tier 1: Fast Heuristics & Real Bytes]
    B -->|Magic Numbers + User Profile Rules| Z{Confidence >= 80%?}
    Z -->|Yes| R1[Instant Match 0ms]
    Z -->|No| C[Tier 2: Text Extraction & 256D Embeddings]
    C -->|Vector Similarity 32 Topical Anchors| Y{Confidence >= 70%?}
    Y -->|Yes| R2[Semantic Match ~5ms]
    Y -->|No| D[Tier 3: Local SLM/LLM Reasoning]
    D -->|Local GBNF/JSON Inference| R3[Deep Reasoning Classification]
    R1 --> SUB[Subcategories Engine: Games / Companies / Subjects]
    R2 --> SUB
    R3 --> SUB
```

1. **Tier 1 — Fast Heuristics and Magic Numbers (`0ms`)**:
   - Real MIME-type detection using file signatures via `infer`.
   - Matching against the local database of user-defined rules and historical corrections (`profile.db`).
   - Direct resolution for media formats (audio, video, images) and installers.
2. **Tier 2 — Text Extraction & 256D Vector Similarity (`~5ms`)**:
   - Extraction of representative text from documents (`pdf-extract`, `docx-rs`, `calamine`).
   - Strict binary header noise rejection against PE/PNG byte remnants.
   - Semantic similarity calculation using 256-dimensional embeddings with 32 bilingual topical anchors and stable centroid clustering.
3. **Tier 3 — Deep Reasoning with Local SLM**:
   - Semantic category naming synthesis with noise filters and clean fallback to *"Miscellaneous Documents"*.
4. **Hierarchical Subcategories Engine**:
   - Automatically groups related files inside the same primary category into multi-level subdirectories (`Images/Games/Zelda`, `Bills & Invoices/Enel`).

---

## System Architecture

```mermaid
graph TD
    subgraph Frontend [Frontend Svelte 5 / TypeScript]
        UI[App.svelte / Routes]
        ST[Reactive Stores]
        I18N[svelte-i18n]
        TREE[FileTreeNode.svelte]
    end

    subgraph IPC [Tauri 2 IPC Bridge]
        CMD_SCAN[scan_folder / scan_specific_files]
        CMD_CLASS[classify_scanned_files]
        CMD_APPLY[apply_organization / undo_last_apply]
        CMD_RENAME[suggest_semantic_names / apply_renames]
        CMD_PROF[get_profile / update_rule / clean_categories]
    end

    subgraph Backend [Native Rust Backend]
        SCANNER[walkdir + rayon Scanner]
        EXTRACT[Content Extractors PDF/DOCX/XLSX]
        ENGINE[Semantic Classifier & Subcategories Engine]
        RENAMER[Smart Renamer Engine]
        MOVER[Safe File Operations & Undo Log]
        DB[(Local SQLite profile.db)]
    end

    UI --> ST
    ST --> IPC
    IPC --> SCANNER
    SCANNER --> EXTRACT
    EXTRACT --> ENGINE
    ENGINE --> RENAMER
    ENGINE --> DB
    ENGINE --> IPC
    IPC --> TREE
    TREE --> CMD_APPLY
    CMD_APPLY --> MOVER
    MOVER --> DB
```

---

## Complete Repository Structure

```text
Indexo/
├── index.html                      # Frontend HTML entrypoint for Vite
├── package.json                    # Svelte 5 / Tauri frontend dependencies and scripts
├── package-lock.json               # Node.js lockfile
├── tsconfig.json                   # TypeScript compiler configuration
├── vite.config.ts                  # Vite bundler and Svelte plugin configuration
├── README.md                       # Project overview and user guide (Portuguese)
├── README_EN.md                    # Project overview and user guide (English)
├── LICENSE                         # GNU General Public License v3.0
├── .gitignore                      # Git exclusion rules
├── Indexo.exe                      # Standalone compiled portable executable
│
├── src/                            # Svelte 5 + TypeScript Frontend
│   ├── main.ts                     # Svelte application initialization
│   ├── App.svelte                  # Root component and navigation orchestrator
│   │
│   ├── lib/                        # Shared libraries and visual modules
│   │   ├── api.ts                  # Typed client wrappers for Tauri backend commands
│   │   ├── stores.ts               # Global reactive state management (Svelte stores)
│   │   ├── FileTreeNode.svelte     # Visual directory tree with nested subfolders and SVG badges
│   │   └── i18n/                   # Translation dictionaries
│   │       ├── pt-BR.json          # Brazilian Portuguese
│   │       └── en-US.json          # American English
│   │
│   ├── routes/                     # Screen views and application flows
│   │   ├── FolderSelect.svelte     # Landing view with folder/files selection & drag-and-drop
│   │   ├── Scanning.svelte         # Directory scan and content extraction progress view
│   │   ├── Preview.svelte          # Side-by-side (Before vs. After) multi-level preview tree
│   │   ├── Renamer.svelte          # Batch smart renamer with semantic presets
│   │   ├── TagManager.svelte       # Tag and learned rule management
│   │   ├── CategoryManager.svelte  # Category management and purge maintenance view
│   │   └── Settings.svelte         # Settings panel (theme, language, thresholds)
│   │
│   ├── i18n/                       # Internationalization setup (svelte-i18n)
│   │   └── setup.ts                # Locale detection and dictionary loader
│   │
│   └── styles/                     # Global styling
│       └── theme.css               # Design system, color tokens, and dark/light themes
│
├── src-tauri/                      # Native Rust Backend (Tauri 2)
│   ├── Cargo.toml                  # Rust dependencies (tauri, tokio, rusqlite, rayon, infer)
│   ├── Cargo.lock                  # Rust dependency lockfile
│   ├── build.rs                    # Tauri build hook script
│   ├── tauri.conf.json             # Tauri 2 runtime and window configuration
│   │
│   ├── src/                        # Rust source code
│   │   ├── main.rs                 # Executable entrypoint and command registry
│   │   │
│   │   ├── commands/               # Handlers for frontend IPC invocations
│   │   │   ├── mod.rs              # Command module exports
│   │   │   ├── scan.rs             # Recursive directory and specific files scanner
│   │   │   ├── classify.rs         # Batch semantic classifier and tier routing
│   │   │   ├── apply.rs            # Atomic file mover and undo rollback log
│   │   │   ├── rename.rs           # Batch semantic rename suggestions and execution
│   │   │   ├── profile.rs          # Category/rule management and DB cleanup commands
│   │   │   └── system.rs           # Windows File Explorer path integration
│   │   │
│   │   ├── engine/                 # Intelligence & classification engine core
│   │   │   ├── mod.rs              # Pipeline orchestrator and unit tests
│   │   │   ├── heuristics.rs       # Tier 1: Fast heuristics and organized folder detection
│   │   │   ├── content_extract.rs  # Secure text extraction from PDF, DOCX, XLSX, and text
│   │   │   ├── embeddings.rs       # Tier 2: 256D embeddings with 32 anchors and centroid clustering
│   │   │   ├── subcategories.rs    # Hierarchical subcategories engine (games, companies, subjects)
│   │   │   ├── llm_local.rs        # Tier 3: Semantic naming and binary noise filtering
│   │   │   ├── renamer.rs          # Smart renamer engine and collision resolver
│   │   │   └── rules.rs            # Dynamic rule synthesizer and evaluator
│   │   │
│   │   ├── fs_ops/                 # Safe filesystem operations
│   │   │   ├── mod.rs              # Path validation and traversal prevention
│   │   │   └── mover.rs            # Atomic safe move, collision handling, and rollback
│   │   │
│   │   └── db/                     # Local SQLite database layer
│   │       ├── mod.rs              # Connections, maintenance queries, and transactions in profile.db
│   │       ├── models.rs           # Data structures (Category, Rule, ActionLog, Session)
│   │       └── schema.sql          # Relational database schema with indices
│   │
│   ├── capabilities/               # Security policies and permissions
│   │   └── default.json            # Dialog, filesystem, and IPC capability grants
│   │
│   └── icons/                      # Application icons in multiple resolutions
│       ├── icon.ico                # Windows executable icon
│       ├── icon.png                # High-resolution standard icon
│       └── ...                     # Multi-resolution icon assets
│
└── data/                           # Local persistence directory (runtime created)
    └── profile.db                  # Local SQLite user learning profile
```

---

## User Experience Flow (UI/UX)

1. **Folder or File Selection**: Open the app and select or drag-and-drop folders or specific files.
2. **Scanning & Extraction**: The native Rust engine scans directories in parallel, identifying *magic numbers* and extracting text in milliseconds with real-time feedback.
3. **Interactive Multi-Level Preview**: Inspect the proposed **Before vs. After** reorganization tree with deep hierarchical subcategories and preserved folders.
4. **Integrated Semantic Renaming**: With a single toggle, inspect and standardize file names with dates, categories, and subjects.
5. **Safe Execution**: Click **Apply** to atomically organize files into clean categories with full transactional rollback logging.
6. **1-Click Undo**: Revert the entire reorganization session at any time with the **Undo** button.

---

## Getting Started & Development

### Prerequisites
* **Windows 10 or 11 (64-bit)**
* **Rust Toolchain** (`rustc` and `cargo` installed via [rustup.rs](https://rustup.rs))
* **Node.js 18+** and **npm** ([nodejs.org](https://nodejs.org))

### 1. Clone the Repository

```powershell
git clone https://github.com/pongitV/Indexo.git
cd Indexo
```

### 2. Install Frontend Dependencies

```powershell
npm install
```

### 3. Run in Development Mode

```powershell
npm run tauri dev
```

This launches the Vite development server with Hot Module Replacement (HMR) and connects it to the native Tauri Rust backend.

---

## Building the Standalone Portable Executable

To compile the optimized release binary and package the standalone `.exe`:

```powershell
npm run tauri build
```

The self-contained standalone executable will be located at:
`src-tauri/target/release/indexo.exe`

> Simply copy `indexo.exe` to any folder or flash drive and run it directly.

---

## Development Roadmap

* **Phase 0**: Stack Foundation (Rust + Tauri 2 + Svelte 5 + SQLite `profile.db`).
* **Phase 1**: Content extraction and 3-tier semantic classification engine.
* **Phase 2**: Reactive UI, side-by-side preview tree, and user approval flows.
* **Phase 3**: Safe physical mover, transactional rollback (Undo), and standalone packaging.

---

## License

This project is free and open-source software, distributed under the **GNU General Public License v3.0 (GPLv3)**. See the [LICENSE](LICENSE) file for more details.
