<script lang="ts">
  import { _ } from "svelte-i18n";
  import { classifiedFiles, showToast } from "./stores";
  import { recordUserCorrection, createCategory, type ClassifiedFile } from "./api";

  export let show = false;
  export let onClose: () => void;
  export let onSaved: () => void = () => {};

  // Lista de itens não identificados
  $: unidentifiedItems = $classifiedFiles.filter((f) => f.is_unidentified || f.suggested_category.startsWith("Nao-Identificados") || f.suggested_category === "Outros Arquivos");

  let selectedFileIds = new Set<string>();
  let selectedGlobalCategory = "Executaveis";
  let customSubfolderInput = "";
  let isSubmitting = false;

  const globalCategories = [
    { id: "Executaveis", label: "Executáveis", desc: "Jogos, Softwares, IDEs, Instaladores", color: "#d14d41" },
    { id: "Media", label: "Mídia", desc: "Imagens, Vídeos, Áudios, SFX", color: "#24837b" },
    { id: "Documentos", label: "Documentos", desc: "Fiscais, Trabalho, Estudos", color: "#da702c" },
    { id: "Projetos", label: "Projetos", desc: "GitHub, Local, 3D, Scripts", color: "#8b7ec8" },
    { id: "Compactados-Backups", label: "Compactados", desc: "Arquivos .ZIP, .RAR, Backups", color: "#4385be" },
    { id: "Fontes-Tipografia", label: "Fontes", desc: "Fontes .TTF, .OTF, Tipografia", color: "#879a39" },
  ];

  const quickSubfolderPresets: Record<string, string[]> = {
    Executaveis: ["Jogos-Indies-Portateis", "Jogos-Steam", "Jogos-Epicgames", "Aplicativos-IDEs", "Aplicativos-Navegadores", "Aplicativos-Utilitarios", "Instaladores-Setups"],
    Media: ["Imagens-Fotografias", "Videos-Gravacoes", "Audios-Musicas", "Imagens-Fotografias/Wallpapers", "Imagens-Fotografias/Screenshots"],
    Documentos: ["Fiscais-Pessoais", "Trabalho", "Estudos", "Fiscais-Pessoais/Boletos-Faturas", "Trabalho/Planilhas", "Estudos/Livros-Ebooks"],
    Projetos: ["Repositorios-GitHub", "Repositorios-Locais", "Modelos-3D-CAD", "Scripts-Automacoes"],
    "Compactados-Backups": ["Backups", "Arquivos-ZIP"],
    "Fontes-Tipografia": ["Fontes-Principais", "Icones-Fontes"],
  };

  function selectAll() {
    selectedFileIds = new Set(unidentifiedItems.map((f) => f.file_id));
  }

  function clearSelection() {
    selectedFileIds = new Set();
  }

  function toggleFile(id: string) {
    if (selectedFileIds.has(id)) {
      selectedFileIds.delete(id);
    } else {
      selectedFileIds.add(id);
    }
    selectedFileIds = new Set(selectedFileIds);
  }

  function formatBytes(bytes: number): string {
    if (!bytes || bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
  }

  function sanitizeFolderName(name: string): string {
    return name
      .replace(/\s+/g, "-")
      .replace(/[<>:"|?*]/g, "_")
      .trim();
  }

  $: computedDestinationPath = (() => {
    const sub = sanitizeFolderName(customSubfolderInput);
    if (!sub) {
      return selectedGlobalCategory;
    }
    return `${selectedGlobalCategory}/${sub}`;
  })();

  async function handleApplyDestination() {
    if (selectedFileIds.size === 0) {
      showToast("Selecione pelo menos um arquivo ou pasta para mover.", "info");
      return;
    }

    const finalPath = computedDestinationPath;
    isSubmitting = true;

    try {
      // Criar a categoria correspondente no banco
      const category = await createCategory(finalPath, "#d14d41");

      const targetIds = new Set(selectedFileIds);

      for (const id of targetIds) {
        await recordUserCorrection(id, "", category.id);
      }

      classifiedFiles.update((list) =>
        list.map((item) => {
          if (targetIds.has(item.file_id)) {
            return {
              ...item,
              suggested_category: finalPath,
              category_id: category.id,
              category_color: category.color ?? "#d14d41",
              confidence: 1.0,
              tier_used: 1,
              is_unidentified: false,
            };
          }
          return item;
        })
      );

      const count = targetIds.size;
      showToast(
        count === 1
          ? `1 item atribuído para '${finalPath}' com sucesso!`
          : `${count} itens atribuídos para '${finalPath}' com sucesso!`,
        "success"
      );

      selectedFileIds.clear();
      selectedFileIds = new Set();
      customSubfolderInput = "";

      onSaved();

      // Se não restar mais nenhum item não identificado, fecha o modal
      const remaining = $classifiedFiles.filter((f) => f.is_unidentified);
      if (remaining.length === 0) {
        onClose();
      }
    } catch (err: any) {
      showToast("Erro ao definir destino: " + err, "error");
    } finally {
      isSubmitting = false;
    }
  }
</script>

{#if show}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={onClose}
    on:keydown={(e) => e.key === "Escape" && onClose()}
  >
    <div class="modal-card modal-unidentified glass-panel">
      <!-- Header -->
      <div class="modal-header">
        <div class="header-title-box">
          <div class="header-icon-box">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#d14d41" stroke-width="2">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="8" x2="12" y2="12"></line>
              <line x1="12" y1="16" x2="12.01" y2="16"></line>
            </svg>
          </div>
          <div>
            <h2 class="modal-title">Revisão Manual de Não Identificados</h2>
            <p class="modal-subtitle">
              {unidentifiedItems.length} {unidentifiedItems.length === 1 ? 'item requer' : 'itens requerem'} decisão de destino. Escolha ou crie pastas e subpastas personalizadas.
            </p>
          </div>
        </div>
        <button class="close-btn" on:click={onClose} title="Fechar">✕</button>
      </div>

      <!-- Main Dual Panel -->
      <div class="review-grid">
        <!-- Left Panel: List of Unidentified Items -->
        <div class="items-panel">
          <div class="panel-toolbar">
            <span class="selection-status">
              {selectedFileIds.size} de {unidentifiedItems.length} selecionados
            </span>
            <div class="selection-buttons">
              <button class="text-action-btn" on:click={selectAll}>Selecionar Todos</button>
              <button class="text-action-btn" on:click={clearSelection}>Limpar</button>
            </div>
          </div>

          <div class="items-list-container">
            {#if unidentifiedItems.length === 0}
              <div class="empty-list-box">
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="#24837b" stroke-width="2">
                  <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
                  <polyline points="22 4 12 14.01 9 11.01"></polyline>
                </svg>
                <span>Nenhum item pendente de identificação!</span>
              </div>
            {:else}
              {#each unidentifiedItems as file (file.file_id)}
                <div
                  class="item-row"
                  class:selected={selectedFileIds.has(file.file_id)}
                  role="button"
                  tabindex="0"
                  on:click={() => toggleFile(file.file_id)}
                  on:keydown={(e) => (e.key === "Enter" || e.key === " ") && toggleFile(file.file_id)}
                >
                  <input
                    type="checkbox"
                    checked={selectedFileIds.has(file.file_id)}
                    on:click|stopPropagation={() => toggleFile(file.file_id)}
                  />
                  <div class="item-info">
                    <div class="item-name truncate" title={file.filename}>
                      {file.filename}
                    </div>
                    <div class="item-path truncate" title={file.path}>
                      {file.path}
                    </div>
                  </div>
                  <div class="item-meta">
                    <span class="item-size">{formatBytes(file.size_bytes || 0)}</span>
                  </div>
                </div>
              {/each}
            {/if}
          </div>
        </div>

        <!-- Right Panel: Destination Setup (Manual & Custom) -->
        <div class="destination-panel">
          <h3 class="section-title">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
            </svg>
            1. Selecione a Pasta Global
          </h3>

          <div class="global-categories-grid">
            {#each globalCategories as cat}
              <button
                class="global-cat-btn"
                class:active={selectedGlobalCategory === cat.id}
                on:click={() => {
                  selectedGlobalCategory = cat.id;
                  customSubfolderInput = "";
                }}
              >
                <div class="cat-pill-indicator" style="background: {cat.color}"></div>
                <div class="cat-text-group">
                  <span class="cat-label">{cat.label}</span>
                  <span class="cat-desc truncate">{cat.desc}</span>
                </div>
              </button>
            {/each}
          </div>

          <h3 class="section-title" style="margin-top: 1rem;">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="9 18 15 12 9 6"></polyline>
            </svg>
            2. Criar ou Escolher Subpasta (Opcional)
          </h3>

          <div class="subfolder-input-box">
            <input
              type="text"
              class="text-input no-margin"
              placeholder="Ex: Jogos-Indies-Portateis/PEAK ou Minhas-Planilhas"
              bind:value={customSubfolderInput}
              on:input={(e) => {
                customSubfolderInput = e.currentTarget.value.replace(/\s+/g, "-");
              }}
            />
          </div>

          <!-- Quick Presets Chips -->
          {#if quickSubfolderPresets[selectedGlobalCategory]}
            <div class="presets-row">
              <span class="presets-label">Sugestões rápidas:</span>
              <div class="chips-container">
                {#each quickSubfolderPresets[selectedGlobalCategory] as preset}
                  <button
                    class="chip-btn"
                    class:active={customSubfolderInput === preset}
                    on:click={() => (customSubfolderInput = preset)}
                  >
                    {preset}
                  </button>
                {/each}
              </div>
            </div>
          {/if}

          <!-- Preview of Target Path -->
          <div class="path-preview-box">
            <span class="path-preview-label">Destino Final:</span>
            <div class="path-preview-value truncate">
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="#24837b" stroke-width="2">
                <polyline points="20 6 9 17 4 12"></polyline>
              </svg>
              <strong>{computedDestinationPath}/</strong>[Arquivo]
            </div>
          </div>

          <!-- Actions -->
          <div class="destination-actions">
            <button
              class="primary-btn apply-btn"
              disabled={isSubmitting || selectedFileIds.size === 0}
              on:click={handleApplyDestination}
            >
              {#if isSubmitting}
                <span>Gravando...</span>
              {:else}
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M5 12h14"></path>
                  <path d="M12 5l7 7-7 7"></path>
                </svg>
                <span>Mover {selectedFileIds.size} {selectedFileIds.size === 1 ? 'Item' : 'Itens'} para este Destino</span>
              {/if}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(16, 15, 15, 0.75);
    backdrop-filter: blur(4px);
    z-index: 3000;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
    animation: fadeIn 150ms ease-out;
  }

  .modal-card.modal-unidentified {
    background: var(--bg-primary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-xl);
    width: 950px;
    max-width: 95vw;
    height: 640px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1.15rem 1.5rem;
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-secondary);
  }

  .header-title-box {
    display: flex;
    align-items: center;
    gap: 0.85rem;
  }

  .header-icon-box {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: var(--radius-md);
    background: rgba(209, 77, 65, 0.12);
    border: 1px solid rgba(209, 77, 65, 0.3);
  }

  .modal-title {
    font-size: 1.05rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .modal-subtitle {
    font-size: 0.8rem;
    color: var(--text-muted);
    margin: 0.15rem 0 0 0;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 1.1rem;
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    border-radius: var(--radius-sm);
  }

  .close-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .review-grid {
    flex: 1;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    padding: 1.25rem;
    min-height: 0;
    overflow: hidden;
  }

  .items-panel, .destination-panel {
    display: flex;
    flex-direction: column;
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    padding: 1rem;
    min-height: 0;
    overflow: hidden;
  }

  .panel-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-bottom: 0.75rem;
    border-bottom: 1px solid var(--border-subtle);
    margin-bottom: 0.75rem;
  }

  .selection-status {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .selection-buttons {
    display: flex;
    gap: 0.5rem;
  }

  .text-action-btn {
    background: transparent;
    border: none;
    color: var(--accent-primary);
    font-size: 0.78rem;
    cursor: pointer;
    font-weight: 500;
    padding: 0.2rem 0.4rem;
    border-radius: var(--radius-sm);
  }

  .text-action-btn:hover {
    background: var(--bg-tertiary);
  }

  .items-list-container {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .empty-list-box {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    height: 100%;
    color: var(--text-muted);
    font-size: 0.85rem;
  }

  .item-row {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.55rem 0.75rem;
    border-radius: var(--radius-md);
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    cursor: pointer;
    transition: all 120ms ease;
  }

  .item-row:hover {
    border-color: var(--border-medium);
    background: var(--bg-hover);
  }

  .item-row.selected {
    background: rgba(209, 77, 65, 0.08);
    border-color: rgba(209, 77, 65, 0.4);
  }

  .item-info {
    flex: 1;
    min-width: 0;
  }

  .item-name {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .item-path {
    font-size: 0.72rem;
    color: var(--text-muted);
  }

  .item-meta {
    font-size: 0.75rem;
    color: var(--text-muted);
    flex-shrink: 0;
  }

  .section-title {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.85rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0 0 0.6rem 0;
  }

  .global-categories-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.5rem;
  }

  .global-cat-btn {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.55rem 0.75rem;
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    transition: all 120ms ease;
  }

  .global-cat-btn:hover {
    background: var(--bg-hover);
    border-color: var(--border-medium);
  }

  .global-cat-btn.active {
    border-color: var(--accent-primary);
    background: rgba(209, 77, 65, 0.08);
  }

  .cat-pill-indicator {
    width: 6px;
    height: 24px;
    border-radius: 3px;
    flex-shrink: 0;
  }

  .cat-text-group {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .cat-label {
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .cat-desc {
    font-size: 0.7rem;
    color: var(--text-muted);
  }

  .subfolder-input-box {
    margin-bottom: 0.5rem;
  }

  .text-input {
    width: 100%;
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 0.5rem 0.75rem;
    color: var(--text-primary);
    font-size: 0.82rem;
    outline: none;
    box-sizing: border-box;
  }

  .text-input:focus {
    border-color: var(--accent-primary);
  }

  .presets-row {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    margin-bottom: 0.75rem;
  }

  .presets-label {
    font-size: 0.72rem;
    color: var(--text-muted);
  }

  .chips-container {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }

  .chip-btn {
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 0.25rem 0.55rem;
    font-size: 0.72rem;
    color: var(--text-primary);
    cursor: pointer;
    transition: all 120ms ease;
  }

  .chip-btn:hover {
    background: var(--bg-hover);
    border-color: var(--border-medium);
  }

  .chip-btn.active {
    background: var(--accent-primary);
    color: #fff;
    border-color: var(--accent-primary);
  }

  .path-preview-box {
    background: var(--bg-tertiary);
    border: 1px dashed var(--border-medium);
    border-radius: var(--radius-md);
    padding: 0.65rem 0.85rem;
    margin-top: auto;
    margin-bottom: 0.85rem;
  }

  .path-preview-label {
    font-size: 0.7rem;
    color: var(--text-muted);
    display: block;
    margin-bottom: 0.2rem;
  }

  .path-preview-value {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.82rem;
    color: var(--text-primary);
    font-family: monospace;
  }

  .destination-actions {
    display: flex;
  }

  .apply-btn {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.65rem 1rem;
    background: var(--accent-primary);
    color: #fff;
    border: none;
    border-radius: var(--radius-md);
    font-weight: 600;
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 150ms ease;
  }

  .apply-btn:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .apply-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .truncate {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>
