<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { _ } from "svelte-i18n";
  import {
    classifiedFiles,
    currentSessionId,
    selectedFolder,
    scanSummary,
    showToast,
  } from "../lib/stores";
  import {
    scanFolder,
    scanSpecificFiles,
    classifyScannedFiles,
    suggestSemanticNames,
    applyRenames,
    undoLastApply,
    openInExplorer,
    openWithDefaultApp,
    getFilePreview,
    onScanProgress,
    onClassifyProgress,
    type RenameConfig,
    type RenameSuggestion,
    type FileRenameCandidate,
    type RenameOperation,
    type FilePreviewData,
    type ScanProgressPayload,
    type ClassifyProgressPayload,
  } from "../lib/api";
  import FilePreviewModal from "../lib/FilePreviewModal.svelte";

  // Reorderable structure elements (sem emojis)
  interface StructureBlock {
    id: "date" | "subject" | "clean_name";
    label: string;
    description: string;
    enabled: boolean;
  }

  let structureBlocks: StructureBlock[] = [
    { id: "date", label: "Data", description: "Ano-Mês ou Dia", enabled: true },
    { id: "subject", label: "Assunto / Conteúdo", description: "Detecção de fatura, jogo, local", enabled: true },
    { id: "clean_name", label: "Nome Limpo", description: "Nome original sem ruídos", enabled: true },
  ];

  // Renamer configuration
  let config: RenameConfig = {
    preset: "custom",
    separator: "_",
    case_style: "title",
    date_format: "YYYY-MM",
    include_category: false,
    remove_noise: true,
    custom_template: null,
    structure_order: ["date", "subject", "clean_name"],
  };

  let suggestions: RenameSuggestion[] = [];
  let userEditedNames = new Map<string, string>(); // file_id -> custom_name
  let ignoredFileIds = new Set<string>();

  let searchQuery = "";
  let isGenerating = false;
  let isScanning = false;
  let isApplying = false;
  let isUndoing = false;
  let isDragging = false;

  // Real-time Scan & Analysis Progress State
  let progressState = {
    phase: "",
    processed: 0,
    total: 0,
    currentFile: "",
    percent: 0,
  };

  // Modals state
  let showConfirmModal = false;
  let showEditModal = false;
  let showPreviewModal = false;

  let activeSuggestion: RenameSuggestion | null = null;
  let editInputName = "";
  let previewLoading = false;
  let filePreviewData: FilePreviewData | null = null;

  // Context Menu State
  let contextMenu = {
    visible: false,
    x: 0,
    y: 0,
    item: null as RenameSuggestion | null,
  };

  let unlistenScan: (() => void) | null = null;
  let unlistenClassify: (() => void) | null = null;

  onMount(async () => {
    try {
      unlistenScan = await onScanProgress((payload: ScanProgressPayload) => {
        progressState.phase = "Varrendo arquivos no disco...";
        progressState.processed = payload.files_scanned;
        progressState.currentFile = payload.current_file;
      });

      unlistenClassify = await onClassifyProgress((payload: ClassifyProgressPayload) => {
        progressState.phase =
          payload.current_phase === "heuristics"
            ? "Analisando metadados e datas..."
            : payload.current_phase === "extracting"
            ? "Extraindo conteúdo e texto..."
            : payload.current_phase === "clustering"
            ? "Detectando assuntos semânticos..."
            : "Finalizando análise...";
        progressState.processed = payload.processed;
        progressState.total = payload.total;
        progressState.percent = payload.total > 0 ? Math.round((payload.processed / payload.total) * 100) : 0;
        if (payload.item) {
          progressState.currentFile = payload.item.filename;
        }
      });
    } catch (_) {}

    syncConfigStructure();
    await updateSuggestions();
  });

  onDestroy(() => {
    if (unlistenScan) unlistenScan();
    if (unlistenClassify) unlistenClassify();
  });

  function syncConfigStructure() {
    config.structure_order = structureBlocks
      .filter((b) => b.enabled)
      .map((b) => b.id);
  }

  function moveBlockLeft(index: number) {
    if (index <= 0) return;
    const temp = structureBlocks[index];
    structureBlocks[index] = structureBlocks[index - 1];
    structureBlocks[index - 1] = temp;
    structureBlocks = [...structureBlocks];
    syncConfigStructure();
    updateSuggestions();
  }

  function moveBlockRight(index: number) {
    if (index >= structureBlocks.length - 1) return;
    const temp = structureBlocks[index];
    structureBlocks[index] = structureBlocks[index + 1];
    structureBlocks[index + 1] = temp;
    structureBlocks = [...structureBlocks];
    syncConfigStructure();
    updateSuggestions();
  }

  function toggleBlock(index: number) {
    structureBlocks[index].enabled = !structureBlocks[index].enabled;
    structureBlocks = [...structureBlocks];
    syncConfigStructure();
    updateSuggestions();
  }

  $: if (config && $classifiedFiles.length > 0) {
    updateSuggestions();
  }

  $: previewExampleName = (() => {
    const activeIds = structureBlocks.filter((b) => b.enabled).map((b) => b.id);
    if (activeIds.length === 0) return "Arquivo.ext";

    const parts: string[] = [];
    const sep = config.separator;

    for (const id of activeIds) {
      if (id === "date") {
        parts.push(config.date_format === "YYYY-MM-DD" ? "2026-05-15" : "2026-05");
      } else if (id === "subject") {
        parts.push("Fatura");
      } else if (id === "clean_name") {
        parts.push("Cartao");
      }
    }

    let joined = parts.join(sep);
    if (config.case_style === "lower") joined = joined.toLowerCase();
    else if (config.case_style === "upper") joined = joined.toUpperCase();
    return joined + ".pdf";
  })();

  async function handlePickFolder() {
    try {
      const path = await open({
        directory: true,
        multiple: false,
        title: "Selecionar Pasta para Renomear",
      });

      if (typeof path === "string" && path.trim().length > 0) {
        await processFolderPath(path);
      }
    } catch (err: any) {
      showToast("Erro ao abrir seletor de pasta: " + err, "error");
    }
  }

  async function handlePickFiles() {
    try {
      const paths = await open({
        directory: false,
        multiple: true,
        title: "Selecionar Arquivos para Renomear",
      });

      if (Array.isArray(paths) && paths.length > 0) {
        await processFilePaths(paths);
      } else if (typeof paths === "string" && paths.trim().length > 0) {
        await processFilePaths([paths]);
      }
    } catch (err: any) {
      showToast("Erro ao abrir seletor de arquivos: " + err, "error");
    }
  }

  async function processFolderPath(path: string) {
    isScanning = true;
    progressState = {
      phase: "Iniciando varredura...",
      processed: 0,
      total: 0,
      currentFile: "",
      percent: 0,
    };
    selectedFolder.set(path);
    userEditedNames.clear();
    ignoredFileIds.clear();

    try {
      const summary = await scanFolder(path);
      scanSummary.set(summary);
      currentSessionId.set(summary.session_id);
      progressState.total = summary.total_files;

      const results = await classifyScannedFiles(summary.session_id);
      classifiedFiles.set(results);

      await updateSuggestions();
      showToast(`Pasta carregada: ${results.length} arquivos analisados.`, "success");
    } catch (err: any) {
      showToast("Erro ao processar pasta: " + err, "error");
    } finally {
      isScanning = false;
    }
  }

  async function processFilePaths(paths: string[]) {
    isScanning = true;
    progressState = {
      phase: "Iniciando leitura dos arquivos...",
      processed: 0,
      total: paths.length,
      currentFile: "",
      percent: 0,
    };
    userEditedNames.clear();
    ignoredFileIds.clear();

    try {
      const summary = await scanSpecificFiles(paths);
      scanSummary.set(summary);
      currentSessionId.set(summary.session_id);
      progressState.total = summary.total_files;

      const results = await classifyScannedFiles(summary.session_id);
      classifiedFiles.set(results);

      await updateSuggestions();
      showToast(`${results.length} arquivos carregados para renomeação.`, "success");
    } catch (err: any) {
      showToast("Erro ao processar arquivos: " + err, "error");
    } finally {
      isScanning = false;
    }
  }

  async function updateSuggestions() {
    if ($classifiedFiles.length === 0) {
      suggestions = [];
      return;
    }

    isGenerating = true;
    try {
      const candidates: FileRenameCandidate[] = $classifiedFiles.map((f) => ({
        file_id: f.file_id || "",
        path: f.path || "",
        filename: f.filename || "",
        category: "",
        category_color: null,
        size_bytes: f.size_bytes || 0,
        modified_at: (f as any).modified_at || null,
        text_sample: (f as any).text_sample || null,
      }));

      const res = await suggestSemanticNames(candidates, config);

      suggestions = res.map((s) => {
        const isIgnored = ignoredFileIds.has(s.file_id);
        const customName = userEditedNames.get(s.file_id);
        if (customName) {
          return {
            ...s,
            proposed_filename: customName,
            is_modified_by_user: true,
            is_ignored: isIgnored,
          };
        }
        return {
          ...s,
          is_ignored: isIgnored,
        };
      });
    } catch (err: any) {
      showToast("Erro ao calcular sugestões de nomes: " + err, "error");
    } finally {
      isGenerating = false;
    }
  }

  $: filteredSuggestions = suggestions.filter((s) => {
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase();
    return (
      s.current_filename.toLowerCase().includes(q) ||
      s.proposed_filename.toLowerCase().includes(q)
    );
  });

  $: changedCount = suggestions.filter((s) => !s.is_ignored && s.current_filename !== s.proposed_filename).length;

  // Handlers do menu de clique direito e cálculo de posicionamento (Viewport Clamping)
  function clampContextMenu(node: HTMLElement, coords: { x: number; y: number }) {
    function position(pos: { x: number; y: number }) {
      requestAnimationFrame(() => {
        const winWidth = window.innerWidth;
        const winHeight = window.innerHeight;
        const rect = node.getBoundingClientRect();

        let newTop = pos.y;
        let newLeft = pos.x;

        // Se o menu ultrapassar a borda inferior da janela, ajusta para cima
        if (newTop + rect.height > winHeight - 12) {
          newTop = Math.max(12, winHeight - rect.height - 12);
        }

        // Se o menu ultrapassar a borda direita da janela, ajusta para a esquerda
        if (newLeft + rect.width > winWidth - 12) {
          newLeft = Math.max(12, winWidth - rect.width - 12);
        }

        // Limites mínimos de segurança
        if (newTop < 12) newTop = 12;
        if (newLeft < 12) newLeft = 12;

        node.style.top = `${newTop}px`;
        node.style.left = `${newLeft}px`;
        node.style.maxHeight = `${winHeight - 24}px`;
      });
    }

    position(coords);

    return {
      update(newCoords: { x: number; y: number }) {
        position(newCoords);
      },
    };
  }

  function openContextMenu(e: MouseEvent, item: RenameSuggestion) {
    e.preventDefault();
    contextMenu = {
      visible: true,
      x: e.clientX,
      y: e.clientY,
      item,
    };
  }

  function closeContextMenu() {
    contextMenu.visible = false;
  }

  function handleOpenEditModal(item: RenameSuggestion) {
    activeSuggestion = item;
    editInputName = item.proposed_filename;
    closeContextMenu();
    showEditModal = true;
  }

  function handleSaveCustomName() {
    if (!activeSuggestion || !editInputName.trim()) return;
    const clean = editInputName.trim();
    userEditedNames.set(activeSuggestion.file_id, clean);
    userEditedNames = new Map(userEditedNames);

    suggestions = suggestions.map((s) =>
      s.file_id === activeSuggestion!.file_id
        ? { ...s, proposed_filename: clean, is_modified_by_user: true }
        : s
    );

    showEditModal = false;
    showToast("Nome personalizado salvo!", "success");
  }

  function handleRestoreOriginal(item: RenameSuggestion) {
    userEditedNames.delete(item.file_id);
    userEditedNames = new Map(userEditedNames);
    closeContextMenu();
    updateSuggestions();
    showToast(`Nome restaurado para '${item.current_filename}'`, "info");
  }

  function handleToggleIgnore(item: RenameSuggestion) {
    if (ignoredFileIds.has(item.file_id)) {
      ignoredFileIds.delete(item.file_id);
    } else {
      ignoredFileIds.add(item.file_id);
    }
    ignoredFileIds = new Set(ignoredFileIds);
    suggestions = suggestions.map((s) =>
      s.file_id === item.file_id ? { ...s, is_ignored: ignoredFileIds.has(s.file_id) } : s
    );
    closeContextMenu();
  }

  async function handleOpenPreview(path: string) {
    closeContextMenu();
    previewLoading = true;
    showPreviewModal = true;
    filePreviewData = null;
    try {
      filePreviewData = await getFilePreview(path);
    } catch (err: any) {
      showToast("Erro ao abrir pré-visualização: " + err, "error");
    } finally {
      previewLoading = false;
    }
  }

  async function handleOpenWithDefaultApp(path: string) {
    closeContextMenu();
    try {
      await openWithDefaultApp(path);
    } catch (err: any) {
      showToast("Erro ao abrir com aplicativo padrão: " + err, "error");
    }
  }

  async function handleOpenExplorer(path: string) {
    closeContextMenu();
    try {
      await openInExplorer(path);
    } catch (err: any) {
      showToast("Erro ao abrir no Explorador: " + err, "error");
    }
  }

  async function handleApplyRenames() {
    if (!$currentSessionId) return;
    showConfirmModal = false;
    isApplying = true;

    try {
      const operations: RenameOperation[] = suggestions
        .filter((s) => !s.is_ignored && s.current_filename !== s.proposed_filename)
        .map((s) => {
          const sep = s.current_path.includes("\\") ? "\\" : "/";
          const lastSep = s.current_path.lastIndexOf(sep);
          const parentDir = lastSep !== -1 ? s.current_path.substring(0, lastSep) : "";
          const newPath = parentDir ? `${parentDir}${sep}${s.proposed_filename}` : s.proposed_filename;
          return {
            file_id: s.file_id,
            from_path: s.current_path,
            to_path: newPath,
          };
        });

      if (operations.length === 0) {
        showToast("Nenhum arquivo precisa ser renomeado.", "info");
        return;
      }

      const summary = await applyRenames($currentSessionId, operations);

      if (summary.failed.length === 0) {
        showToast($_("renamer.toast.applied", { values: { count: summary.moved } }), "success");
        classifiedFiles.update((list) =>
          list.map((item) => {
            const op = operations.find((o) => o.file_id === item.file_id);
            if (op) {
              const newFilename = op.to_path.split(/[\\/]/).pop() || item.filename;
              return {
                ...item,
                filename: newFilename,
                path: op.to_path,
              };
            }
            return item;
          })
        );
        userEditedNames.clear();
        await updateSuggestions();
      } else {
        showToast(
          `${summary.moved} renomeados, ${summary.failed.length} falharam: ${summary.failed[0]}`,
          "error"
        );
      }
    } catch (err: any) {
      showToast("Erro ao aplicar renomeações: " + err, "error");
    } finally {
      isApplying = false;
    }
  }

  async function handleUndo() {
    isUndoing = true;
    try {
      const count = await undoLastApply($currentSessionId);
      if (count > 0) {
        showToast($_("renamer.toast.undone", { values: { count } }), "success");
        userEditedNames.clear();
        await updateSuggestions();
      } else {
        showToast("Nenhuma renomeação anterior para desfazer.", "info");
      }
    } catch (err: any) {
      showToast("Erro ao desfazer: " + err, "error");
    } finally {
      isUndoing = false;
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDragging = true;
  }

  function handleDragLeave() {
    isDragging = false;
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragging = false;
    if (e.dataTransfer && e.dataTransfer.files.length > 0) {
      const paths = Array.from(e.dataTransfer.files).map((f: any) => f.path || f.name).filter(Boolean);
      if (paths.length > 0) {
        processFilePaths(paths);
      }
    }
  }

  function formatBytes(bytes: number): string {
    if (!bytes || bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
  }
</script>

<svelte:window on:click={closeContextMenu} />

<div
  class="renamer-view-page"
  role="region"
  aria-label="Tela do Renomeador"
  on:dragover={handleDragOver}
  on:dragleave={handleDragLeave}
  on:drop={handleDrop}
>
  <!-- Header with Title & Action Buttons -->
  <div class="renamer-header">
    <div class="header-titles">
      <div class="visual-badge">
        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
          <polyline points="14 2 14 8 20 8"></polyline>
          <line x1="16" y1="13" x2="8" y2="13"></line>
        </svg>
        <span>Renomeador por Conteúdo e Metadados</span>
      </div>
      <h1>{$_("renamer.title")}</h1>
      <p class="subtitle">Padroniza nomes no mesmo local onde estão, sem mover arquivos para outras pastas.</p>
    </div>

    <!-- Source Selection & Action Buttons -->
    <div class="header-actions">
      <button class="source-btn secondary-btn" on:click={handlePickFolder} title="Escolher uma pasta para renomear">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
        </svg>
        Selecionar Pasta
      </button>

      <button class="source-btn secondary-btn" on:click={handlePickFiles} title="Escolher arquivos específicos">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
          <polyline points="14 2 14 8 20 8"></polyline>
        </svg>
        Selecionar Arquivos
      </button>

      <button
        class="secondary-btn"
        disabled={isUndoing || isApplying}
        on:click={handleUndo}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M3 7v6h6"></path>
          <path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"></path>
        </svg>
        {isUndoing ? $_("preview.undoing") : $_("renamer.undo")}
      </button>

      <button
        class="primary-btn"
        disabled={isApplying || changedCount === 0}
        on:click={() => (showConfirmModal = true)}
      >
        {#if isApplying}
          <div class="mini-spinner"></div>
          {$_("renamer.applying")}
        {:else}
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20 6 9 17 4 12"></polyline>
          </svg>
          {$_("renamer.apply")} ({changedCount})
        {/if}
      </button>
    </div>
  </div>

  <!-- Interactive Structure Reordering & Format Card -->
  <div class="pattern-builder-card glass-panel">
    <div class="structure-control-section">
      <div class="section-title-row">
        <span class="pattern-label">Ordem da Estrutura do Nome:</span>
        <span class="order-hint">Use as setas para mover a posição de cada bloco na composição do nome.</span>
      </div>

      <!-- Pipeline de Blocos Reordenáveis -->
      <div class="structure-pipeline">
        {#each structureBlocks as block, i (block.id)}
          <div class="pipeline-block" class:disabled={!block.enabled}>
            <div class="block-order-index">{i + 1}</div>

            <div class="block-info">
              <button
                class="block-toggle-btn"
                class:active={block.enabled}
                on:click={() => toggleBlock(i)}
                title="Clique para ativar/desativar este bloco"
              >
                <span class="block-name">{block.label}</span>
                <span class="block-status">{block.enabled ? "Ativo" : "Desativado"}</span>
              </button>
            </div>

            <!-- Botões de Reordenação Esquerda / Direita -->
            <div class="block-nav-btns">
              <button
                class="nav-arrow-btn"
                disabled={i === 0}
                on:click={() => moveBlockLeft(i)}
                title="Mover para esquerda"
              >
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                  <polyline points="15 18 9 12 15 6"></polyline>
                </svg>
              </button>

              <button
                class="nav-arrow-btn"
                disabled={i === structureBlocks.length - 1}
                on:click={() => moveBlockRight(i)}
                title="Mover para direita"
              >
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                  <polyline points="9 18 15 12 9 6"></polyline>
                </svg>
              </button>
            </div>
          </div>

          {#if i < structureBlocks.length - 1}
            <div class="pipeline-connector">
              <span class="sep-symbol">{config.separator}</span>
            </div>
          {/if}
        {/each}

        <div class="pipeline-connector">
          <span class="sep-symbol">.</span>
        </div>

        <div class="pipeline-block extension-block">
          <span class="ext-name">EXTENSÃO</span>
        </div>
      </div>
    </div>

    <!-- Live Preview Sample & Formatting Controls -->
    <div class="format-controls-row">
      <!-- Exemplo Dinâmico em Tempo Real -->
      <div class="live-preview-sample">
        <span class="sample-label">Resultado:</span>
        <code class="sample-code">{previewExampleName}</code>
      </div>

      <!-- Formato de Data -->
      <div class="pill-group">
        <span class="pill-group-title">Formato de Data:</span>
        <button class="style-pill" class:active={config.date_format === "YYYY-MM"} on:click={() => (config.date_format = "YYYY-MM")}>AAAA-MM</button>
        <button class="style-pill" class:active={config.date_format === "YYYY-MM-DD"} on:click={() => (config.date_format = "YYYY-MM-DD")}>AAAA-MM-DD</button>
        <button class="style-pill" class:active={config.date_format === "DD-MM-YYYY"} on:click={() => (config.date_format = "DD-MM-YYYY")}>DD-MM-AAAA</button>
      </div>

      <!-- Gramática -->
      <div class="pill-group">
        <span class="pill-group-title">Gramática:</span>
        <button class="style-pill" class:active={config.case_style === "title"} on:click={() => (config.case_style = "title")}>Title Case</button>
        <button class="style-pill" class:active={config.case_style === "lower"} on:click={() => (config.case_style = "lower")}>minusculo</button>
        <button class="style-pill" class:active={config.case_style === "upper"} on:click={() => (config.case_style = "upper")}>MAIUSCULO</button>
        <button class="style-pill" class:active={config.case_style === "camel"} on:click={() => (config.case_style = "camel")}>camelCase</button>
        <button class="style-pill" class:active={config.case_style === "snake"} on:click={() => (config.case_style = "snake")}>snake_case</button>
        <button class="style-pill" class:active={config.case_style === "kebab"} on:click={() => (config.case_style = "kebab")}>kebab-case</button>
      </div>

      <!-- Separadores -->
      <div class="pill-group">
        <span class="pill-group-title">Separador:</span>
        <button class="style-pill" class:active={config.separator === "_"} on:click={() => (config.separator = "_")}>Under (_)</button>
        <button class="style-pill" class:active={config.separator === "-"} on:click={() => (config.separator = "-")}>Hífen (-)</button>
        <button class="style-pill" class:active={config.separator === " "} on:click={() => (config.separator = " ")}>Espaço</button>
        <button class="style-pill" class:active={config.separator === "."} on:click={() => (config.separator = ".")}>Ponto (.)</button>
      </div>
    </div>
  </div>

  <!-- Search & Stats Toolbar -->
  <div class="toolbar-row">
    <div class="search-box">
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"></circle>
        <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
      </svg>
      <input
        type="text"
        placeholder="Filtrar por nome original ou novo nome..."
        bind:value={searchQuery}
      />
      {#if searchQuery}
        <button class="clear-search" on:click={() => (searchQuery = "")}>✕</button>
      {/if}
    </div>

    <div class="stats-badge">
      {#if isScanning}
        <div class="mini-spinner small"></div>
        <span>Analisando arquivos...</span>
      {:else if isGenerating}
        <div class="mini-spinner small"></div>
        <span>Calculando nomes...</span>
      {:else}
        <span><strong>{changedCount}</strong> de {suggestions.length} arquivos serão renomeados</span>
      {/if}
    </div>
  </div>

  <!-- Main Comparison List: Original x Preview (Sem thumbnails) -->
  <div class="comparison-list-container glass-panel" class:dragging={isDragging}>
    {#if suggestions.length === 0 && !isScanning}
      <div class="empty-state">
        <div class="empty-icon-svg">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
            <polyline points="14 2 14 8 20 8"></polyline>
            <line x1="16" y1="13" x2="8" y2="13"></line>
          </svg>
        </div>
        <h3>Lista de Renomeação</h3>
        <p>Selecione uma pasta ou arquivos específicos para visualizar a comparação direta entre o nome original e o preview do novo nome.</p>
        <div class="empty-actions">
          <button class="primary-btn" on:click={handlePickFolder}>
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
            </svg>
            Selecionar Pasta
          </button>
          <button class="secondary-btn" on:click={handlePickFiles}>
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
              <polyline points="14 2 14 8 20 8"></polyline>
            </svg>
            Selecionar Arquivos
          </button>
        </div>
        <span class="empty-sub">Ou arraste arquivos diretamente para esta tela.</span>
      </div>
    {:else if isScanning}
      <!-- Real-time Progress Card -->
      <div class="renamer-progress-card">
        <div class="progress-spinner-box">
          <div class="mini-spinner extra-large"></div>
        </div>
        <h3>{progressState.phase || "Processando e analisando arquivos..."}</h3>
        <div class="progress-bar-track">
          <div class="progress-bar-fill" style="width: {progressState.percent}%"></div>
        </div>
        <div class="progress-details-row">
          <span class="progress-file truncate">
            {progressState.currentFile ? `Analisando: ${progressState.currentFile}` : "Lendo metadados..."}
          </span>
          <span class="progress-count">
            {#if progressState.total > 0}
              {progressState.processed} / {progressState.total} ({progressState.percent}%)
            {:else}
              {progressState.processed} arquivos
            {/if}
          </span>
        </div>
      </div>
    {:else}
      <!-- Comparison Table: Original vs Preview -->
      <table class="comparison-table">
        <thead>
          <tr>
            <th class="col-original-header">Nome Original</th>
            <th class="col-divider-header"></th>
            <th class="col-preview-header">Preview do Novo Nome</th>
            <th class="col-size-header">Tamanho</th>
            <th class="col-status-header">Status</th>
            <th class="col-actions-header">Ações</th>
          </tr>
        </thead>
        <tbody>
          {#each filteredSuggestions as item (item.file_id)}
            {@const hasDiff = item.current_filename !== item.proposed_filename && !item.is_ignored}
            <tr
              class="comparison-row"
              class:ignored={item.is_ignored}
              class:modified={item.is_modified_by_user}
              class:has-diff={hasDiff}
              on:contextmenu={(e) => openContextMenu(e, item)}
            >
              <!-- Nome Original -->
              <td class="col-original" title={item.current_path}>
                <div class="original-cell-content">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="row-icon">
                    <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                    <polyline points="14 2 14 8 20 8"></polyline>
                  </svg>
                  <span class="filename-text truncate">{item.current_filename}</span>
                </div>
              </td>

              <!-- Seta indicativa -->
              <td class="col-divider">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="arrow-icon" class:active={hasDiff}>
                  <line x1="5" y1="12" x2="19" y2="12"></line>
                  <polyline points="12 5 19 12 12 19"></polyline>
                </svg>
              </td>

              <!-- Preview do Novo Nome (Editável inline) -->
              <td class="col-preview" title="Clique para editar este nome">
                <div
                  class="preview-cell-content"
                  role="button"
                  tabindex="0"
                  on:click={() => handleOpenEditModal(item)}
                  on:keydown={(e) => e.key === "Enter" && handleOpenEditModal(item)}
                >
                  <span class="preview-filename-text truncate" class:highlight={hasDiff}>
                    {item.proposed_filename}
                  </span>
                  <button class="edit-hint-btn" title="Editar nome">
                    <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                      <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                    </svg>
                  </button>
                </div>
              </td>

              <!-- Tamanho -->
              <td class="col-size text-muted">
                {formatBytes(item.size_bytes)}
              </td>

              <!-- Status -->
              <td class="col-status">
                {#if item.is_ignored}
                  <span class="status-badge ignored">Ignorado</span>
                {:else if item.is_modified_by_user}
                  <span class="status-badge edited">Manual</span>
                {:else if item.has_collision}
                  <span class="status-badge collision">Colisão resolvida</span>
                {:else if hasDiff}
                  <span class="status-badge ready">Pronto</span>
                {:else}
                  <span class="status-badge no-change">Sem alteração</span>
                {/if}
              </td>

              <!-- Ações Rápidas por Linha -->
              <td class="col-actions">
                <div class="row-actions-group">
                  <button
                    class="action-icon-btn"
                    title="Editar nome"
                    on:click={() => handleOpenEditModal(item)}
                  >
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                      <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                    </svg>
                  </button>

                  {#if item.is_modified_by_user || hasDiff}
                    <button
                      class="action-icon-btn"
                      title="Restaurar nome original"
                      on:click={() => handleRestoreOriginal(item)}
                    >
                      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"></path>
                        <path d="M21 3v5h-5"></path>
                      </svg>
                    </button>
                  {/if}

                  <button
                    class="action-icon-btn"
                    title="Visualizar arquivo"
                    on:click={() => handleOpenPreview(item.current_path)}
                  >
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
                      <circle cx="12" cy="12" r="3"></circle>
                    </svg>
                  </button>

                  <button
                    class="action-icon-btn"
                    title="Abrir no Explorador"
                    on:click={() => handleOpenExplorer(item.current_path)}
                  >
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
                    </svg>
                  </button>

                  <button
                    class="action-icon-btn"
                    class:active-danger={item.is_ignored}
                    title={item.is_ignored ? "Incluir na renomeação" : "Ignorar arquivo"}
                    on:click={() => handleToggleIgnore(item)}
                  >
                    {#if item.is_ignored}
                      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="20 6 9 17 4 12"></polyline>
                      </svg>
                    {:else}
                      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <circle cx="12" cy="12" r="10"></circle>
                        <line x1="4.93" y1="4.93" x2="19.07" y2="19.07"></line>
                      </svg>
                    {/if}
                  </button>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
</div>

<!-- Context Menu no Clique Direito -->
{#if contextMenu.visible && contextMenu.item}
  <div
    class="custom-context-menu"
    use:clampContextMenu={{ x: contextMenu.x, y: contextMenu.y }}
    role="menu"
    tabindex="-1"
    on:click|stopPropagation
    on:keydown|stopPropagation
  >
    <div class="context-card-header">
      <div class="context-file-title truncate" title={contextMenu.item.current_filename}>
        {contextMenu.item.current_filename}
      </div>
      <div class="context-file-path truncate" title={contextMenu.item.proposed_filename}>
        ↳ {contextMenu.item.proposed_filename}
      </div>
    </div>

    <div class="context-divider"></div>

    <!-- 1. Editar Nome -->
    <button
      class="context-item"
      role="menuitem"
      on:click={() => handleOpenEditModal(contextMenu.item!)}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
        <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
      </svg>
      {$_("renamer.context.edit_name")}
    </button>

    <!-- 2. Restaurar Nome Original -->
    {#if contextMenu.item.is_modified_by_user || contextMenu.item.current_filename !== contextMenu.item.proposed_filename}
      <button
        class="context-item"
        role="menuitem"
        on:click={() => handleRestoreOriginal(contextMenu.item!)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"></path>
          <path d="M21 3v5h-5"></path>
        </svg>
        {$_("renamer.context.restore_original")}
      </button>
    {/if}

    <div class="context-divider"></div>

    <!-- 3. Visualizar -->
    <button
      class="context-item highlight-action"
      role="menuitem"
      on:click={() => handleOpenPreview(contextMenu.item!.current_path)}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
        <circle cx="12" cy="12" r="3"></circle>
      </svg>
      {$_("renamer.context.preview")}
    </button>

    <!-- 4. Abrir no Aplicativo Padrão -->
    <button
      class="context-item"
      role="menuitem"
      on:click={() => handleOpenWithDefaultApp(contextMenu.item!.current_path)}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
        <polyline points="15 3 21 3 21 9"></polyline>
        <line x1="10" y1="14" x2="21" y2="3"></line>
      </svg>
      Abrir no App Padrão
    </button>

    <!-- 5. Abrir no Explorador -->
    <button
      class="context-item"
      role="menuitem"
      on:click={() => handleOpenExplorer(contextMenu.item!.current_path)}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
      </svg>
      {$_("renamer.context.open_explorer")}
    </button>

    <div class="context-divider"></div>

    <!-- 5. Ignorar / Incluir -->
    <button
      class="context-item {contextMenu.item.is_ignored ? '' : 'text-danger'}"
      role="menuitem"
      on:click={() => handleToggleIgnore(contextMenu.item!)}
    >
      {#if contextMenu.item.is_ignored}
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="20 6 9 17 4 12"></polyline>
        </svg>
        {$_("renamer.context.unignore")}
      {:else}
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"></circle>
          <line x1="4.93" y1="4.93" x2="19.07" y2="19.07"></line>
        </svg>
        {$_("renamer.context.ignore")}
      {/if}
    </button>
  </div>
{/if}

<!-- Modal: Editar Nome Proposto -->
{#if showEditModal && activeSuggestion}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (showEditModal = false)}
    on:keydown={(e) => e.key === "Escape" && (showEditModal = false)}
  >
    <div class="modal-card">
      <h2>{$_("renamer.modal.edit_title")}</h2>
      <p class="modal-subtitle">Nome original: <strong>{activeSuggestion.current_filename}</strong></p>

      <input
        type="text"
        bind:value={editInputName}
        class="text-input"
        placeholder="Digite o novo nome do arquivo com a extensão..."
        on:keydown={(e) => e.key === "Enter" && handleSaveCustomName()}
      />

      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showEditModal = false)}>
          Cancelar
        </button>
        <button class="primary-btn" on:click={handleSaveCustomName}>
          Salvar Nome
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Confirmar Aplicação da Renomeação -->
{#if showConfirmModal}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (showConfirmModal = false)}
    on:keydown={(e) => e.key === "Escape" && (showConfirmModal = false)}
  >
    <div class="modal-card">
      <h2>{$_("renamer.modal.apply_title")}</h2>
      <p class="modal-subtitle">{$_("renamer.modal.apply_msg", { values: { count: changedCount } })}</p>

      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showConfirmModal = false)}>
          Cancelar
        </button>
        <button class="primary-btn" on:click={handleApplyRenames}>
          {$_("renamer.modal.confirm")}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal Amplo: Visualização de Conteúdo do Arquivo (Inspector / Quick Look) -->
<FilePreviewModal
  show={showPreviewModal}
  loading={previewLoading}
  data={filePreviewData}
  onClose={() => (showPreviewModal = false)}
  onOpenWithDefaultApp={handleOpenWithDefaultApp}
  onOpenInExplorer={handleOpenExplorer}
/>

<style>
  .renamer-view-page {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 1.25rem 1.5rem;
    gap: 1rem;
    overflow-y: auto;
    overflow-x: hidden;
    height: 100%;
    width: 100%;
    min-height: 0;
    min-width: 0;
    animation: fadeIn 250ms ease-out;
  }

  .renamer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1.5rem;
    flex-wrap: wrap;
    flex-shrink: 0;
  }

  .visual-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.25rem 0.65rem;
    border-radius: var(--radius-full);
    background: var(--accent-light);
    color: var(--accent-primary);
    font-size: 0.74rem;
    font-weight: 700;
    margin-bottom: 0.35rem;
    border: 1px solid rgba(20, 184, 166, 0.25);
  }

  .header-titles h1 {
    font-size: 1.4rem;
    font-weight: 700;
    margin: 0;
    color: var(--text-primary);
  }

  .subtitle {
    font-size: 0.85rem;
    color: var(--text-muted);
    margin-top: 0.2rem;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    flex-wrap: wrap;
  }

  .source-btn {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-medium);
    font-weight: 600;
  }

  .source-btn:hover {
    border-color: var(--accent-primary);
    color: var(--accent-primary);
  }

  /* Structure Reordering Section */
  .pattern-builder-card {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1.1rem 1.25rem;
    border-radius: var(--radius-lg);
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .structure-control-section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .section-title-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .pattern-label {
    font-size: 0.8rem;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .order-hint {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .structure-pipeline {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    background: var(--bg-primary);
    padding: 0.75rem;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
  }

  .pipeline-block {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    padding: 0.35rem 0.55rem;
    transition: all 120ms ease;
  }

  .pipeline-block.disabled {
    opacity: 0.45;
    background: var(--bg-tertiary);
    border-style: dashed;
  }

  .block-order-index {
    font-size: 0.72rem;
    font-weight: 700;
    background: var(--bg-tertiary);
    color: var(--text-muted);
    width: 20px;
    height: 20px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .block-toggle-btn {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    background: transparent;
    border: none;
    cursor: pointer;
    text-align: left;
    padding: 0.15rem 0.35rem;
  }

  .block-name {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .block-status {
    font-size: 0.68rem;
    color: var(--text-muted);
  }

  .block-toggle-btn.active .block-name {
    color: var(--accent-primary);
  }

  .block-nav-btns {
    display: flex;
    align-items: center;
    gap: 0.2rem;
    border-left: 1px solid var(--border-subtle);
    padding-left: 0.35rem;
    margin-left: 0.2rem;
  }

  .nav-arrow-btn {
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 100ms ease;
  }

  .nav-arrow-btn:hover:not(:disabled) {
    background: var(--accent-primary);
    color: white;
    border-color: var(--accent-primary);
  }

  .nav-arrow-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .pipeline-connector {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0 0.15rem;
  }

  .sep-symbol {
    font-family: var(--font-mono);
    font-weight: 700;
    color: var(--text-muted);
    font-size: 0.95rem;
  }

  .extension-block {
    background: rgba(139, 92, 246, 0.1);
    border-color: rgba(139, 92, 246, 0.3);
    padding: 0.45rem 0.75rem;
  }

  .ext-name {
    font-size: 0.76rem;
    font-weight: 700;
    color: #a78bfa;
    letter-spacing: 0.05em;
  }

  .format-controls-row {
    display: flex;
    align-items: center;
    gap: 1.25rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--border-subtle);
    flex-wrap: wrap;
  }

  .live-preview-sample {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: var(--bg-primary);
    padding: 0.35rem 0.75rem;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
  }

  .sample-label {
    font-size: 0.74rem;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
  }

  .sample-code {
    font-family: var(--font-mono);
    font-size: 0.84rem;
    color: var(--accent-primary);
    font-weight: 700;
  }

  .pill-group {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    flex-wrap: wrap;
  }

  .pill-group-title {
    font-size: 0.74rem;
    font-weight: 600;
    color: var(--text-muted);
    margin-right: 0.2rem;
  }

  .style-pill {
    padding: 0.22rem 0.55rem;
    border-radius: var(--radius-sm);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    font-size: 0.74rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 120ms ease;
  }

  .style-pill:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .style-pill.active {
    background: var(--accent-primary);
    border-color: var(--accent-primary);
    color: white;
    font-weight: 600;
  }

  /* Toolbar */
  .toolbar-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-shrink: 0;
    flex-wrap: wrap;
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 0.45rem 0.85rem;
    min-width: 280px;
    flex: 1;
  }

  .search-box input {
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-primary);
    font-size: 0.84rem;
    width: 100%;
  }

  .clear-search {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 0.85rem;
  }

  .stats-badge {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.82rem;
    color: var(--text-muted);
    background: var(--bg-secondary);
    padding: 0.45rem 0.85rem;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
  }

  /* Main Comparison List Container */
  .comparison-list-container {
    flex: 1;
    overflow-y: auto;
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    min-height: 0;
  }

  .comparison-list-container.dragging {
    border-color: var(--accent-primary);
    background: var(--accent-light);
  }

  /* Progress Display Card */
  .renamer-progress-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1.25rem;
    padding: 3.5rem 2rem;
    background: var(--bg-primary);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border-subtle);
    max-width: 540px;
    margin: 2rem auto;
    text-align: center;
  }

  .progress-spinner-box {
    margin-bottom: 0.5rem;
  }

  .renamer-progress-card h3 {
    margin: 0;
    font-size: 1.1rem;
    color: var(--text-primary);
    font-weight: 700;
  }

  .progress-bar-track {
    width: 100%;
    height: 8px;
    border-radius: var(--radius-full);
    background: var(--bg-tertiary);
    overflow: hidden;
  }

  .progress-bar-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--accent-primary), #0284c7);
    border-radius: var(--radius-full);
    transition: width 150ms ease-out;
  }

  .progress-details-row {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.78rem;
    color: var(--text-muted);
    gap: 1rem;
  }

  .progress-file {
    font-family: var(--font-mono);
    max-width: 320px;
  }

  .progress-count {
    font-weight: 700;
    color: var(--text-primary);
    flex-shrink: 0;
  }

  /* Comparison Table */
  .comparison-table {
    width: 100%;
    border-collapse: collapse;
    text-align: left;
    font-size: 0.84rem;
  }

  .comparison-table thead {
    background: var(--bg-tertiary);
    position: sticky;
    top: 0;
    z-index: 10;
    border-bottom: 1px solid var(--border-subtle);
  }

  .comparison-table th {
    padding: 0.75rem 1rem;
    font-weight: 600;
    font-size: 0.78rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .col-original-header {
    width: 34%;
  }

  .col-divider-header {
    width: 30px;
    padding: 0 !important;
  }

  .col-preview-header {
    width: 38%;
  }

  .col-size-header {
    width: 90px;
  }

  .col-status-header {
    width: 120px;
  }

  .col-actions-header {
    width: 140px;
    text-align: right;
  }

  .comparison-row {
    border-bottom: 1px solid var(--border-subtle);
    transition: background 120ms ease;
  }

  .comparison-row:hover {
    background: var(--bg-tertiary);
  }

  .comparison-row.ignored {
    opacity: 0.4;
    text-decoration: line-through;
  }

  .comparison-row.modified {
    background: rgba(16, 185, 129, 0.04);
  }

  .comparison-table td {
    padding: 0.65rem 1rem;
    vertical-align: middle;
  }

  .original-cell-content {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-family: var(--font-mono);
    font-size: 0.82rem;
    color: var(--text-secondary);
  }

  .row-icon {
    flex-shrink: 0;
    color: var(--text-muted);
  }

  .col-divider {
    text-align: center;
    padding: 0 !important;
  }

  .arrow-icon {
    color: var(--text-muted);
    opacity: 0.5;
  }

  .arrow-icon.active {
    color: var(--accent-primary);
    opacity: 1;
  }

  .preview-cell-content {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-family: var(--font-mono);
    font-size: 0.82rem;
    cursor: pointer;
    border-radius: var(--radius-sm);
    padding: 0.2rem 0.4rem;
    margin: -0.2rem -0.4rem;
    transition: background 120ms ease;
  }

  .preview-cell-content:hover {
    background: var(--bg-hover);
  }

  .preview-filename-text {
    color: var(--text-primary);
  }

  .preview-filename-text.highlight {
    color: var(--accent-primary);
    font-weight: 600;
  }

  .edit-hint-btn {
    opacity: 0;
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 0.1rem 0.25rem;
    display: flex;
    align-items: center;
    transition: opacity 120ms ease;
  }

  .preview-cell-content:hover .edit-hint-btn {
    opacity: 1;
  }

  .col-size {
    font-size: 0.78rem;
  }

  .status-badge {
    display: inline-block;
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-sm);
    font-size: 0.72rem;
    font-weight: 600;
  }

  .status-badge.ready {
    background: rgba(59, 130, 246, 0.15);
    color: #60a5fa;
  }

  .status-badge.edited {
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
  }

  .status-badge.collision {
    background: rgba(245, 158, 11, 0.15);
    color: #fbbf24;
  }

  .status-badge.ignored {
    background: rgba(100, 116, 139, 0.15);
    color: #94a3b8;
  }

  .status-badge.no-change {
    background: transparent;
    color: var(--text-muted);
  }

  .col-actions {
    text-align: right;
  }

  .row-actions-group {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.25rem;
  }

  .action-icon-btn {
    width: 26px;
    height: 26px;
    border-radius: var(--radius-sm);
    background: transparent;
    border: 1px solid var(--border-subtle);
    color: var(--text-muted);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 120ms ease;
  }

  .action-icon-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
    border-color: var(--border-medium);
  }

  .action-icon-btn.active-danger {
    color: var(--accent-rose);
    border-color: rgba(244, 63, 94, 0.3);
  }

  .action-icon-btn.active-danger:hover {
    background: rgba(244, 63, 94, 0.15);
  }

  /* General Buttons */
  .primary-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: var(--accent-primary);
    color: white;
    border: none;
    border-radius: var(--radius-md);
    padding: 0.55rem 1.1rem;
    font-weight: 600;
    cursor: pointer;
    font-size: 0.88rem;
    transition: filter 150ms ease;
  }

  .primary-btn:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .primary-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .secondary-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 0.55rem 1rem;
    font-weight: 500;
    cursor: pointer;
    font-size: 0.88rem;
  }

  .empty-state {
    padding: 3.5rem 1rem;
    text-align: center;
    color: var(--text-muted);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.6rem;
  }

  .empty-icon-svg {
    color: var(--text-muted);
    margin-bottom: 0.5rem;
  }

  .empty-state h3 {
    margin: 0;
    font-size: 1.15rem;
    color: var(--text-primary);
  }

  .empty-state p {
    margin: 0;
    font-size: 0.88rem;
    max-width: 480px;
    line-height: 1.4;
  }

  .empty-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-top: 0.5rem;
  }

  .empty-sub {
    font-size: 0.78rem;
    color: var(--text-muted);
    margin-top: 0.5rem;
  }

  /* Context Menu */
  .custom-context-menu {
    position: fixed;
    z-index: 2000;
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-xl);
    padding: 0.5rem;
    width: 280px;
    max-height: calc(100vh - 24px);
    overflow-y: auto;
    overflow-x: hidden;
    overscroll-behavior: contain;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    animation: fadeIn 120ms ease-out;
    /* Oculta a barra de rolagem visualmente mantendo o scroll 100% funcional */
    scrollbar-width: none; /* Firefox */
    -ms-overflow-style: none; /* IE e Edge */
  }

  .custom-context-menu::-webkit-scrollbar {
    display: none; /* Chrome, Safari e Webview2 */
    width: 0px;
    height: 0px;
  }

  .context-card-header {
    padding: 0.45rem 0.6rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .context-file-title {
    font-size: 0.85rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .context-file-path {
    font-size: 0.72rem;
    font-family: var(--font-mono);
    color: var(--accent-primary);
  }

  .context-item {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    padding: 0.5rem 0.65rem;
    font-size: 0.82rem;
    font-weight: 500;
    color: var(--text-primary);
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    text-align: left;
    width: 100%;
    cursor: pointer;
    transition: all 120ms ease;
  }

  .context-item:hover {
    background: var(--bg-hover);
    color: var(--accent-primary);
  }

  .context-item.text-danger:hover {
    background: rgba(244, 63, 94, 0.12);
    color: var(--accent-rose);
  }

  .context-divider {
    height: 1px;
    background: var(--border-subtle);
    margin: 0.2rem 0;
  }

  /* Modals */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(6px);
    z-index: 3000;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
  }

  .modal-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-xl);
    padding: 1.75rem;
    max-width: 480px;
    width: 100%;
    animation: fadeIn 200ms ease-out;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .modal-subtitle {
    font-size: 0.85rem;
    color: var(--text-muted);
    margin: 0;
    line-height: 1.4;
  }

  .text-input {
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 0.45rem 0.75rem;
    color: var(--text-primary);
    font-size: 0.84rem;
    outline: none;
    width: 100%;
  }

  .text-input:focus {
    border-color: var(--accent-primary);
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.6rem;
    margin-top: 0.5rem;
  }

  .mini-spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 800ms linear infinite;
  }

  .mini-spinner.small {
    border-color: rgba(59, 130, 246, 0.2);
    border-top-color: var(--accent-primary);
  }

  .mini-spinner.extra-large {
    width: 44px;
    height: 44px;
    border-width: 4px;
    border-color: rgba(59, 130, 246, 0.2);
    border-top-color: var(--accent-primary);
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(6px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
