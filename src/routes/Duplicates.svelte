<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    scanFolderDuplicates,
    resolveDuplicatesActions,
    openInExplorer,
    openWithDefaultApp,
    type DuplicateGroup,
    type DuplicateItem,
    type DuplicateResolveAction,
  } from "../lib/api";
  import { selectedFolder, showToast, currentView } from "../lib/stores";

  let targetFolder: string | null = null;
  let isScanning: boolean = false;
  let isResolving: boolean = false;
  let duplicateGroups: DuplicateGroup[] = [];
  let actionType: "trash" | "delete" | "archive_folder" = "trash";
  let archiveDirectoryPath: string = "";
  let searchQuery: string = "";
  let showConfirmModal: boolean = false;

  onMount(async () => {
    if ($selectedFolder) {
      targetFolder = $selectedFolder;
      await runScan();
    }
  });

  async function handlePickFolder() {
    try {
      const picked = await open({
        directory: true,
        multiple: false,
        title: "Selecionar Pasta para Detectar Duplicatas",
      });
      if (picked && typeof picked === "string") {
        targetFolder = picked;
        await runScan();
      }
    } catch (err: any) {
      showToast("Erro ao selecionar pasta: " + err, "error");
    }
  }

  async function runScan() {
    if (!targetFolder) return;
    isScanning = true;
    duplicateGroups = [];
    try {
      duplicateGroups = await scanFolderDuplicates(targetFolder);
      if (duplicateGroups.length === 0) {
        showToast("Nenhum arquivo duplicado encontrado nesta pasta!", "success");
      } else {
        const totalDupes = duplicateGroups.reduce((acc, g) => acc + (g.items.length - 1), 0);
        showToast(`${duplicateGroups.length} grupos de duplicatas identificados (${totalDupes} arquivos redundantes).`, "info");
      }
    } catch (err: any) {
      showToast("Erro ao escanear duplicatas: " + err, "error");
    } finally {
      isScanning = false;
    }
  }

  function selectItemToKeep(group: DuplicateGroup, chosenItem: DuplicateItem) {
    for (const item of group.items) {
      item.is_selected_to_keep = item.path === chosenItem.path;
    }
    duplicateGroups = [...duplicateGroups];
  }

  function formatBytes(bytes: number): string {
    if (!bytes || bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
  }

  function formatDateTime(dateStr?: string | null): string {
    if (!dateStr) return "Desconhecida";
    try {
      const d = new Date(dateStr);
      if (isNaN(d.getTime())) return dateStr;
      return d.toLocaleString("pt-BR", {
        day: "2-digit",
        month: "2-digit",
        year: "numeric",
        hour: "2-digit",
        minute: "2-digit",
      });
    } catch {
      return dateStr;
    }
  }

  function getFileExtension(filename: string): string {
    const parts = filename.split(".");
    return parts.length > 1 ? parts.pop()?.toLowerCase() || "" : "";
  }

  $: totalRedundantFiles = duplicateGroups.reduce((acc, g) => acc + (g.items.length - 1), 0);
  $: totalPotentialSavings = duplicateGroups.reduce((acc, g) => acc + g.potential_savings_bytes, 0);

  $: filteredGroups = duplicateGroups.filter((g) => {
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase();
    return g.items.some((i) => i.filename.toLowerCase().includes(q) || i.path.toLowerCase().includes(q));
  });

  async function handleApplyResolution() {
    showConfirmModal = false;
    isResolving = true;
    try {
      const actions: DuplicateResolveAction[] = [];

      for (const group of duplicateGroups) {
        const keepItem = group.items.find((i) => i.is_selected_to_keep) || group.items[0];
        const deletePaths = group.items.filter((i) => i.path !== keepItem.path).map((i) => i.path);

        if (deletePaths.length > 0) {
          actions.push({
            keep_path: keepItem.path,
            delete_or_move_paths: deletePaths,
            action_type: actionType,
            archive_folder_path: actionType === "archive_folder" ? archiveDirectoryPath : null,
          });
        }
      }

      const count = await resolveDuplicatesActions(actions);
      showToast(`${count} arquivo(s) duplicado(s) resolvidos com sucesso!`, "success");
      await runScan();
    } catch (err: any) {
      showToast("Erro ao resolver duplicatas: " + err, "error");
    } finally {
      isResolving = false;
    }
  }
</script>

<div class="duplicates-view">
  <!-- Header -->
  <div class="view-header">
    <div class="header-titles">
      <div class="badge-title">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
          <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
        </svg>
        <h1>Deduplicador Inteligente</h1>
      </div>
      <p class="subtitle">
        Detecção de arquivos 100% idênticos com hash SHA-256 e comparação visual com controle total de qual versão manter.
      </p>
    </div>

    <div class="header-actions">
      <button class="secondary-btn" on:click={handlePickFolder}>
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
        </svg>
        {targetFolder ? "Trocar Pasta" : "Selecionar Pasta"}
      </button>

      {#if targetFolder}
        <button class="primary-btn" disabled={isScanning} on:click={runScan}>
          {#if isScanning}
            <div class="mini-spin"></div>
            Analisando Hashes...
          {:else}
            <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="23 4 23 10 17 10"></polyline>
              <polyline points="1 20 1 14 7 14"></polyline>
              <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
            </svg>
            Reanalisar Pasta
          {/if}
        </button>
      {/if}
    </div>
  </div>

  <!-- Folder Banner / Stats Bar -->
  {#if targetFolder}
    <div class="folder-stats-banner">
      <div class="folder-path-box">
        <span class="folder-label">Pasta Analisada:</span>
        <span class="path-mono" title={targetFolder}>{targetFolder}</span>
      </div>

      {#if duplicateGroups.length > 0}
        <div class="metrics-row">
          <div class="metric-item">
            <span class="metric-val">{duplicateGroups.length}</span>
            <span class="metric-lbl">Grupos de Duplicatas</span>
          </div>
          <div class="metric-item">
            <span class="metric-val text-warning">{totalRedundantFiles}</span>
            <span class="metric-lbl">Arquivos Redundantes</span>
          </div>
          <div class="metric-item">
            <span class="metric-val text-success">{formatBytes(totalPotentialSavings)}</span>
            <span class="metric-lbl">Espaço a Liberar</span>
          </div>
        </div>

        <!-- Action Control Bar -->
        <div class="action-resolution-bar">
          <div class="action-options">
            <span class="action-label">Ação para os redundantes:</span>
            <label class="radio-label">
              <input type="radio" bind:group={actionType} value="trash" />
              <span>Mover para Lixeira (Recomendado)</span>
            </label>
            <label class="radio-label">
              <input type="radio" bind:group={actionType} value="delete" />
              <span class="text-danger">Excluir Permanente</span>
            </label>
          </div>

          <div class="search-wrap">
            <input
              type="text"
              placeholder="Buscar nos duplicados..."
              bind:value={searchQuery}
              class="search-mini-input"
            />
          </div>

          <button
            class="resolve-apply-btn"
            disabled={isResolving || totalRedundantFiles === 0}
            on:click={() => (showConfirmModal = true)}
          >
            {#if isResolving}
              <div class="mini-spin"></div>
              Liberando Espaço...
            {:else}
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="20 6 9 17 4 12"></polyline>
              </svg>
              Liberar {formatBytes(totalPotentialSavings)} ({totalRedundantFiles} arquivos)
            {/if}
          </button>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Content List -->
  <div class="duplicates-content">
    {#if isScanning}
      <div class="state-box">
        <div class="spinner"></div>
        <h3>Analisando arquivos e calculando hashes SHA-256...</h3>
        <p>Isso garante 0 falsos positivos comparando o conteúdo exato byte a byte.</p>
      </div>
    {:else if !targetFolder}
      <div class="state-box empty">
        <div class="state-icon">
          <svg width="44" height="44" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
          </svg>
        </div>
        <h2>Selecione uma pasta para encontrar duplicatas</h2>
        <p>O Indexo agrupará fotos, vídeos, músicas e documentos repetidos para você liberar espaço com segurança.</p>
        <button class="primary-btn" on:click={handlePickFolder}>Selecionar Pasta</button>
      </div>
    {:else if duplicateGroups.length === 0}
      <div class="state-box empty">
        <div class="state-icon success">
          <svg width="44" height="44" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
            <polyline points="22 4 12 14.01 9 11.01"></polyline>
          </svg>
        </div>
        <h2>Nenhum arquivo duplicado encontrado!</h2>
        <p>Todos os arquivos na pasta <strong>{targetFolder}</strong> são únicos.</p>
        <button class="secondary-btn" on:click={handlePickFolder}>Analisar Outra Pasta</button>
      </div>
    {:else}
      <div class="groups-list">
        {#each filteredGroups as group, gIdx (group.group_id)}
          <div class="group-card">
            <div class="group-header">
              <div class="group-title-col">
                <span class="group-badge">Grupo #{gIdx + 1}</span>
                <span class="group-size-info">{formatBytes(group.size_bytes)} cada</span>
                <span class="group-savings-tag">
                  Economiza {formatBytes(group.potential_savings_bytes)}
                </span>
              </div>
              <span class="group-hash-tag" title="Hash SHA-256">{group.hash.substring(0, 12)}...</span>
            </div>

            <!-- Items Comparison Grid -->
            <div class="items-grid">
              {#each group.items as item}
                {@const ext = getFileExtension(item.filename)}
                {@const isKeep = item.is_selected_to_keep}

                <div
                  class="duplicate-item-box"
                  class:is-chosen-to-keep={isKeep}
                  role="button"
                  tabindex="0"
                  on:click={() => selectItemToKeep(group, item)}
                  on:keydown={(e) => (e.key === "Enter" || e.key === " ") && selectItemToKeep(group, item)}
                >
                  <div class="item-top-row">
                    <div class="item-radio-wrap">
                      <input
                        type="radio"
                        name={`group-radio-${group.group_id}`}
                        checked={isKeep}
                        on:change={() => selectItemToKeep(group, item)}
                      />
                      <span class="keep-text-label">
                        {isKeep ? "Manter este arquivo" : "Descartar / Mover"}
                      </span>
                    </div>

                    {#if item.is_recommended_to_keep}
                      <span class="recommended-badge" title="Melhor nome ou data mais recente">
                        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                          <polyline points="20 6 9 17 4 12"></polyline>
                        </svg>
                        Sugerido pelo Indexo
                      </span>
                    {/if}
                  </div>

                  <div class="item-filename-row">
                    <span class="ext-badge">{ext.toUpperCase() || "ARQ"}</span>
                    <span class="item-filename" title={item.filename}>{item.filename}</span>
                  </div>

                  <div class="item-path-row" title={item.path}>
                    <span class="path-txt">{item.path}</span>
                  </div>

                  <div class="item-meta-footer">
                    {#if item.resolution}
                      <span class="meta-tag resolution">📐 {item.resolution}</span>
                    {/if}
                    <span class="meta-tag date">🕒 {formatDateTime(item.modified_at)}</span>

                    <div class="item-actions">
                      <button
                        class="icon-btn"
                        title="Abrir no Explorador de Arquivos"
                        on:click|stopPropagation={() => openInExplorer(item.path)}
                      >
                        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                          <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                          <polyline points="15 3 21 3 21 9"></polyline>
                          <line x1="10" y1="14" x2="21" y2="3"></line>
                        </svg>
                      </button>
                      <button
                        class="icon-btn"
                        title="Abrir com Aplicativo Padrão"
                        on:click|stopPropagation={() => openWithDefaultApp(item.path)}
                      >
                        <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                          <polygon points="5 3 19 12 5 21 5 3"></polygon>
                        </svg>
                      </button>
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- Modal de Confirmação de Resolução -->
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
      <div class="modal-icon-wrap text-warning">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"></path>
          <line x1="12" y1="9" x2="12" y2="13"></line>
          <line x1="12" y1="17" x2="12.01" y2="17"></line>
        </svg>
      </div>

      <h2>Confirmar Resolução de Duplicatas</h2>
      <p class="modal-p">
        Você está prestes a processar <strong>{totalRedundantFiles} arquivos redundantes</strong>, liberando aproximadamente <strong>{formatBytes(totalPotentialSavings)}</strong>.
      </p>

      <div class="action-alert-box">
        {#if actionType === "trash"}
          <span>Os arquivos redundantes serão movidos com segurança para a <strong>Lixeira do Windows</strong>.</span>
        {:else if actionType === "delete"}
          <span class="text-danger">Os arquivos redundantes serão <strong>excluídos permanentemente</strong> do disco.</span>
        {/if}
      </div>

      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showConfirmModal = false)}>Cancelar</button>
        <button class="primary-btn apply-btn" on:click={handleApplyResolution}>
          Confirmar e Liberar Espaço
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .duplicates-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 1.5rem 2rem;
    gap: 1.25rem;
    overflow: hidden;
    min-height: 0;
    animation: fadeIn 200ms ease-out;
  }

  .view-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1.5rem;
    flex-wrap: wrap;
  }

  .header-titles {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .badge-title {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    color: var(--accent-primary);
  }

  .badge-title h1 {
    font-size: 1.45rem;
    font-weight: 800;
    color: var(--text-primary);
    margin: 0;
    letter-spacing: -0.02em;
  }

  .subtitle {
    font-size: 0.86rem;
    color: var(--text-muted);
    margin: 0;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .folder-stats-banner {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    padding: 1rem 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  .folder-path-box {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
  }

  .folder-label {
    color: var(--text-muted);
    font-weight: 600;
  }

  .path-mono {
    font-family: var(--font-mono);
    color: var(--text-primary);
    font-weight: 700;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 600px;
  }

  .metrics-row {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    padding: 0.5rem 0;
    border-top: 1px solid var(--border-subtle);
    border-bottom: 1px solid var(--border-subtle);
    flex-wrap: wrap;
  }

  .metric-item {
    display: flex;
    flex-direction: column;
  }

  .metric-val {
    font-size: 1.25rem;
    font-weight: 800;
    color: var(--text-primary);
  }

  .metric-lbl {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .text-warning {
    color: #f59e0b;
  }

  .text-success {
    color: #10b981;
  }

  .text-danger {
    color: #ef4444;
  }

  .action-resolution-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .action-options {
    display: flex;
    align-items: center;
    gap: 1rem;
    font-size: 0.82rem;
  }

  .action-label {
    color: var(--text-muted);
    font-weight: 600;
  }

  .radio-label {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    cursor: pointer;
    color: var(--text-primary);
  }

  .search-mini-input {
    background: var(--bg-primary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    padding: 0.35rem 0.75rem;
    font-size: 0.82rem;
    color: var(--text-primary);
    width: 220px;
  }

  .resolve-apply-btn {
    background: var(--accent-primary);
    color: white;
    border: none;
    padding: 0.5rem 1.25rem;
    border-radius: var(--radius-md);
    font-size: 0.84rem;
    font-weight: 700;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    transition: all 150ms ease;
  }

  .resolve-apply-btn:hover:not(:disabled) {
    box-shadow: 0 0 12px rgba(59, 130, 246, 0.4);
  }

  .duplicates-content {
    flex: 1;
    overflow-y: auto;
    padding-right: 0.35rem;
    scrollbar-width: thin;
  }

  .state-box {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 4rem 2rem;
    gap: 1rem;
    color: var(--text-muted);
  }

  .state-box.empty {
    background: var(--bg-secondary);
    border: 1px dashed var(--border-medium);
    border-radius: var(--radius-xl);
  }

  .state-icon {
    background: var(--bg-tertiary);
    padding: 1.25rem;
    border-radius: 50%;
    color: var(--text-muted);
  }

  .state-icon.success {
    color: #10b981;
    background: rgba(16, 185, 129, 0.12);
  }

  .groups-list {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .group-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    overflow: hidden;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  }

  .group-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    background: rgba(0, 0, 0, 0.08);
    border-bottom: 1px solid var(--border-subtle);
  }

  .group-title-col {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .group-badge {
    font-size: 0.8rem;
    font-weight: 800;
    color: var(--text-primary);
  }

  .group-size-info {
    font-size: 0.78rem;
    color: var(--text-muted);
  }

  .group-savings-tag {
    background: rgba(16, 185, 129, 0.12);
    color: #10b981;
    border: 1px solid rgba(16, 185, 129, 0.25);
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-full);
    font-size: 0.72rem;
    font-weight: 700;
  }

  .group-hash-tag {
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--text-muted);
  }

  .items-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 0.75rem;
    padding: 0.85rem;
  }

  .duplicate-item-box {
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    cursor: pointer;
    transition: all 120ms ease;
  }

  .duplicate-item-box:hover {
    border-color: var(--border-strong, #475569);
  }

  .duplicate-item-box.is-chosen-to-keep {
    border-color: #10b981;
    background: rgba(16, 185, 129, 0.04);
    box-shadow: 0 0 0 1px #10b981;
  }

  .item-top-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .item-radio-wrap {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.78rem;
    font-weight: 700;
  }

  .is-chosen-to-keep .keep-text-label {
    color: #10b981;
  }

  .recommended-badge {
    background: rgba(59, 130, 246, 0.12);
    color: #3b82f6;
    font-size: 0.68rem;
    font-weight: 700;
    padding: 0.1rem 0.4rem;
    border-radius: var(--radius-sm);
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
  }

  .item-filename-row {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .ext-badge {
    background: var(--bg-tertiary);
    font-size: 0.65rem;
    font-weight: 800;
    padding: 0.1rem 0.35rem;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
  }

  .item-filename {
    font-size: 0.85rem;
    font-weight: 700;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .item-path-row {
    font-size: 0.74rem;
    font-family: var(--font-mono);
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .item-meta-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    margin-top: 0.25rem;
    padding-top: 0.35rem;
    border-top: 1px solid var(--border-subtle);
    font-size: 0.74rem;
    color: var(--text-muted);
  }

  .meta-tag.resolution {
    color: #8b5cf6;
    font-weight: 600;
  }

  .item-actions {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    margin-left: auto;
  }

  .icon-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 0.2rem;
    border-radius: var(--radius-sm);
    transition: all 120ms ease;
  }

  .icon-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  /* Modals */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.72);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 3000;
    padding: 1rem;
    animation: fadeIn 150ms ease-out;
  }

  .modal-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-xl);
    padding: 1.75rem;
    max-width: 460px;
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 0.85rem;
    box-shadow: var(--shadow-xl);
  }

  .modal-card h2 {
    margin: 0;
    font-size: 1.2rem;
    color: var(--text-primary);
  }

  .modal-p {
    margin: 0;
    font-size: 0.88rem;
    color: var(--text-muted);
    line-height: 1.45;
  }

  .action-alert-box {
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    padding: 0.75rem 1rem;
    border-radius: var(--radius-md);
    font-size: 0.82rem;
    width: 100%;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    width: 100%;
    margin-top: 0.5rem;
  }

  .secondary-btn {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-medium);
    color: var(--text-primary);
    padding: 0.5rem 1rem;
    border-radius: var(--radius-md);
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
  }

  .primary-btn {
    background: var(--accent-primary);
    color: white;
    border: none;
    padding: 0.5rem 1.25rem;
    border-radius: var(--radius-md);
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid rgba(59, 130, 246, 0.2);
    border-top-color: var(--accent-primary);
    border-radius: 50%;
    animation: spin 800ms linear infinite;
  }

  .mini-spin {
    display: inline-block;
    width: 12px;
    height: 12px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 800ms linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
