<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import {
    listCategories,
    createCategory,
    renameCategory,
    mergeCategories,
    deleteCategory,
    cleanUnusedCategories,
    purgeAutoCategories,
    type Category,
  } from "../lib/api";
  import { showToast } from "../lib/stores";
  import CategoryHistoryModal from "../lib/CategoryHistoryModal.svelte";

  let tags: Category[] = [];
  let searchQuery = "";
  let activeTab: "all" | "manual" | "auto" = "all";
  let isLoading = true;

  // Modals state
  let showNewModal = false;
  let showRenameModal = false;
  let showMergeModal = false;
  let showDeleteModal = false;
  let showPurgeModal = false;
  let showHistoryModal = false;

  let activeTag: Category | null = null;
  let inputName = "";
  let inputColor = "#3b82f6";
  let selectedTargetTagId = "";

  const presetColors = [
    "#3b82f6", "#06b6d4", "#10b981", "#84cc16",
    "#f59e0b", "#f97316", "#ef4444", "#ec4899",
    "#8b5cf6", "#6366f1", "#64748b"
  ];

  onMount(async () => {
    await fetchTags();
  });

  async function fetchTags() {
    isLoading = true;
    try {
      tags = await listCategories();
    } catch (err: any) {
      showToast("Erro ao carregar tags: " + err, "error");
    } finally {
      isLoading = false;
    }
  }

  $: filteredTags = tags.filter((t) => {
    if (activeTab === "manual" && t.created_by !== "user") return false;
    if (activeTab === "auto" && t.created_by !== "auto") return false;
    if (!searchQuery.trim()) return true;
    return t.name.toLowerCase().includes(searchQuery.toLowerCase());
  });

  function openNewModal() {
    inputName = "";
    inputColor = presetColors[Math.floor(Math.random() * presetColors.length)];
    showNewModal = true;
  }

  function openRenameModal(tag: Category) {
    activeTag = tag;
    inputName = tag.name;
    inputColor = tag.color ?? "#3b82f6";
    showRenameModal = true;
  }

  function openMergeModal(tag: Category) {
    activeTag = tag;
    const remaining = tags.filter((t) => t.id !== tag.id);
    selectedTargetTagId = remaining[0]?.id || "";
    showMergeModal = true;
  }

  function openDeleteModal(tag: Category) {
    activeTag = tag;
    showDeleteModal = true;
  }

  function openHistoryModal(tag: Category) {
    activeTag = tag;
    showHistoryModal = true;
  }

  async function handleCreate() {
    if (!inputName.trim()) return;
    try {
      await createCategory(inputName.trim(), inputColor);
      showNewModal = false;
      showToast("Tag criada com sucesso!", "success");
      await fetchTags();
    } catch (err: any) {
      showToast("Erro ao criar tag: " + err, "error");
    }
  }

  async function handleRename() {
    if (!activeTag || !inputName.trim()) return;
    try {
      await renameCategory(activeTag.id, inputName.trim());
      showRenameModal = false;
      showToast("Tag renomeada com sucesso!", "success");
      await fetchTags();
    } catch (err: any) {
      showToast("Erro ao renomear tag: " + err, "error");
    }
  }

  async function handleMerge() {
    if (!activeTag || !selectedTargetTagId) return;
    try {
      await mergeCategories(activeTag.id, selectedTargetTagId);
      showMergeModal = false;
      showToast("Tags mescladas com sucesso!", "success");
      await fetchTags();
    } catch (err: any) {
      showToast("Erro ao mesclar tags: " + err, "error");
    }
  }

  async function handleDelete() {
    if (!activeTag) return;
    try {
      await deleteCategory(activeTag.id);
      showDeleteModal = false;
      showToast("Tag excluída com sucesso!", "success");
      await fetchTags();
    } catch (err: any) {
      showToast("Erro ao excluir tag: " + err, "error");
    }
  }

  async function handlePurgeAuto() {
    try {
      const count = await purgeAutoCategories();
      showPurgeModal = false;
      showToast($_("tags.toast.cleaned", { values: { count } }), "success");
      await fetchTags();
    } catch (err: any) {
      showToast("Erro ao limpar tags automáticas: " + err, "error");
    }
  }

  async function handleCleanUnused() {
    try {
      const count = await cleanUnusedCategories();
      showToast($_("tags.toast.cleaned", { values: { count } }), "success");
      await fetchTags();
    } catch (err: any) {
      showToast("Erro ao limpar tags não utilizadas: " + err, "error");
    }
  }
</script>

<div class="tags-view">
  <!-- Header with search & new tag CTA -->
  <div class="tags-header">
    <div class="header-titles">
      <h1>{$_("tags.manage")}</h1>
      <p class="subtitle">{$_("tags.subtitle")}</p>
    </div>

    <div class="header-actions">
      <div class="search-box">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        <input
          type="text"
          placeholder={$_("preview.search_categories")}
          bind:value={searchQuery}
        />
        {#if searchQuery}
          <button class="clear-search" on:click={() => (searchQuery = "")}>✕</button>
        {/if}
      </div>

      {#if tags.some(t => t.created_by === "auto")}
        <button class="secondary-btn danger-hover" on:click={() => (showPurgeModal = true)} title={$_("tags.clean_auto")}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6"></polyline>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
          </svg>
          {$_("tags.clean_auto")}
        </button>
      {/if}

      <button class="primary-btn" on:click={openNewModal}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19"></line>
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
        {$_("tags.new")}
      </button>
    </div>
  </div>

  <!-- Filter Tabs: All, Manual, Auto -->
  <div class="tabs-nav">
    <button
      class="tab-btn"
      class:active={activeTab === "all"}
      on:click={() => (activeTab = "all")}
    >
      {$_("tags.tab.all")}
      <span class="tab-count">{tags.length}</span>
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === "manual"}
      on:click={() => (activeTab = "manual")}
    >
      {$_("tags.tab.manual")}
      <span class="tab-count">{tags.filter(t => t.created_by === "user").length}</span>
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === "auto"}
      on:click={() => (activeTab = "auto")}
    >
      {$_("tags.tab.auto")}
      <span class="tab-count">{tags.filter(t => t.created_by === "auto").length}</span>
    </button>
  </div>

  <!-- Tag Cards Grid -->
  <div class="tags-container">
    {#if isLoading}
      <div class="empty-state">Carregando tags...</div>
    {:else if filteredTags.length === 0}
      <div class="empty-state">Nenhuma tag encontrada.</div>
    {:else}
      <div class="tags-grid">
        {#each filteredTags as tag (tag.id)}
          <div class="tag-card glass-panel" style="border-top: 3px solid {tag.color ?? '#3b82f6'}">
            <div class="tag-main">
              <div class="tag-title-row" title={tag.name}>
                <span class="tag-dot" style="background: {tag.color ?? '#3b82f6'}"></span>
                <span class="tag-name">{tag.name}</span>
              </div>
              <div class="tag-meta-row">
                <span class="tag-origin-badge {tag.created_by}">
                  {tag.created_by === "auto" ? $_("tags.created_by_auto") : $_("tags.created_by_user")}
                </span>
                <span class="tag-file-count">
                  {$_("tags.files_count", { values: { count: tag.file_count } })}
                </span>
              </div>
            </div>

            <div class="tag-actions-row">
              <button class="action-btn" title="Histórico de Mudanças" on:click={() => openHistoryModal(tag)}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"></circle>
                  <polyline points="12 6 12 12 16 14"></polyline>
                </svg>
              </button>

              <button class="action-btn" title={$_("tags.rename")} on:click={() => openRenameModal(tag)}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                  <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                </svg>
              </button>

              <button class="action-btn" title={$_("tags.merge")} on:click={() => openMergeModal(tag)}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="16 3 21 3 21 8"></polyline>
                  <line x1="4" y1="20" x2="21" y2="3"></line>
                  <polyline points="21 16 21 21 16 21"></polyline>
                  <line x1="15" y1="15" x2="21" y2="21"></line>
                </svg>
              </button>

              <button class="action-btn danger" title={$_("tags.delete")} on:click={() => openDeleteModal(tag)}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="3 6 5 6 21 6"></polyline>
                  <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                </svg>
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- Modal: Criar Nova Tag -->
{#if showNewModal}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (showNewModal = false)}
    on:keydown={(e) => e.key === "Escape" && (showNewModal = false)}
  >
    <div class="modal-card">
      <h2>{$_("tags.new")}</h2>
      <input
        type="text"
        placeholder={$_("tags.name")}
        bind:value={inputName}
        class="text-input"
        on:keydown={(e) => e.key === "Enter" && handleCreate()}
      />
      <div class="color-palette-select">
        <label for="tag-color-picker">Cor de Destaque:</label>
        <div class="palette-dots">
          {#each presetColors as col}
            <button
              class="palette-dot"
              class:selected={inputColor === col}
              style="background: {col}"
              aria-label="Cor {col}"
              title="Cor {col}"
              on:click={() => (inputColor = col)}
            ></button>
          {/each}
        </div>
      </div>
      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showNewModal = false)}>Cancelar</button>
        <button class="primary-btn" on:click={handleCreate}>{$_("tags.create")}</button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Renomear Tag -->
{#if showRenameModal && activeTag}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (showRenameModal = false)}
    on:keydown={(e) => e.key === "Escape" && (showRenameModal = false)}
  >
    <div class="modal-card">
      <h2>{$_("tags.modal.rename_title")}</h2>
      <input
        type="text"
        placeholder={$_("tags.name")}
        bind:value={inputName}
        class="text-input"
        on:keydown={(e) => e.key === "Enter" && handleRename()}
      />
      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showRenameModal = false)}>Cancelar</button>
        <button class="primary-btn" on:click={handleRename}>Salvar</button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Mesclar Tags -->
{#if showMergeModal && activeTag}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (showMergeModal = false)}
    on:keydown={(e) => e.key === "Escape" && (showMergeModal = false)}
  >
    <div class="modal-card">
      <h2>{$_("tags.modal.merge_title")}</h2>
      <p class="modal-subtitle">
        {$_("tags.modal.merge_desc", { values: { source: activeTag.name } })}
      </p>
      <select bind:value={selectedTargetTagId} class="select-input">
        {#each tags.filter((t) => t.id !== activeTag?.id) as targetT}
          <option value={targetT.id}>{targetT.name} ({targetT.file_count} arquivos)</option>
        {/each}
      </select>
      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showMergeModal = false)}>Cancelar</button>
        <button class="primary-btn" on:click={handleMerge}>Mesclar</button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Excluir Tag -->
{#if showDeleteModal && activeTag}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (showDeleteModal = false)}
    on:keydown={(e) => e.key === "Escape" && (showDeleteModal = false)}
  >
    <div class="modal-card">
      <h2>{$_("tags.modal.delete_title")}</h2>
      <p class="modal-subtitle">
        {$_("tags.modal.delete_desc", { values: { name: activeTag.name } })}
      </p>
      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showDeleteModal = false)}>Cancelar</button>
        <button class="danger-btn" on:click={handleDelete}>Excluir</button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Limpar Tags Automáticas -->
{#if showPurgeModal}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (showPurgeModal = false)}
    on:keydown={(e) => e.key === "Escape" && (showPurgeModal = false)}
  >
    <div class="modal-card">
      <h2>{$_("tags.modal.purge_title")}</h2>
      <p class="modal-subtitle">
        {$_("tags.modal.purge_desc")}
      </p>
      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showPurgeModal = false)}>Cancelar</button>
        <button class="secondary-btn" on:click={handleCleanUnused}>{$_("tags.clean_unused")}</button>
        <button class="danger-btn" on:click={handlePurgeAuto}>{$_("tags.clean_auto")}</button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Histórico de Mudanças da Tag -->
<CategoryHistoryModal
  show={showHistoryModal}
  categoryId={activeTag?.id ?? null}
  categoryName={activeTag?.name ?? ""}
  categoryColor={activeTag?.color ?? null}
  onClose={() => (showHistoryModal = false)}
/>

<style>
  .tags-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 1.25rem 1.5rem;
    gap: 1.25rem;
    overflow: hidden;
    min-height: 0;
    min-width: 0;
    animation: fadeIn 250ms ease-out;
  }

  .tags-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1.5rem;
    flex-wrap: wrap;
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
    gap: 0.75rem;
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 0.45rem 0.85rem;
    min-width: 240px;
  }

  .search-box input {
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-primary);
    font-size: 0.85rem;
    width: 100%;
  }

  .clear-search {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 0.85rem;
  }

  .tabs-nav {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    border-bottom: 1px solid var(--border-subtle);
    padding-bottom: 0.5rem;
  }

  .tab-btn {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.45rem 0.9rem;
    border-radius: var(--radius-md);
    background: transparent;
    border: 1px solid transparent;
    color: var(--text-muted);
    font-size: 0.85rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 150ms ease;
  }

  .tab-btn:hover {
    color: var(--text-primary);
    background: var(--bg-tertiary);
  }

  .tab-btn.active {
    background: var(--accent-primary);
    color: white;
    font-weight: 600;
  }

  .tab-count {
    background: rgba(0, 0, 0, 0.18);
    font-size: 0.72rem;
    padding: 0.1rem 0.45rem;
    border-radius: var(--radius-full);
  }

  .tags-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    min-height: 0;
  }

  .tags-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 1rem;
    padding-bottom: 1rem;
  }

  .tag-card {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 1rem;
    border-radius: var(--radius-md);
    gap: 0.85rem;
    transition: transform 150ms ease, box-shadow 150ms ease;
  }

  .tag-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.12);
  }

  .tag-title-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 600;
    font-size: 0.95rem;
    color: var(--text-primary);
  }

  .tag-dot {
    width: 10px;
    height: 10px;
    border-radius: var(--radius-full);
    flex-shrink: 0;
  }

  .tag-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tag-meta-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 0.4rem;
    font-size: 0.75rem;
  }

  .tag-origin-badge {
    padding: 0.15rem 0.45rem;
    border-radius: var(--radius-sm);
    font-weight: 500;
  }

  .tag-origin-badge.auto {
    background: rgba(139, 92, 246, 0.15);
    color: #a78bfa;
  }

  .tag-origin-badge.user {
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
  }

  .tag-file-count {
    color: var(--text-muted);
  }

  .tag-actions-row {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.35rem;
    border-top: 1px solid var(--border-subtle);
    padding-top: 0.6rem;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    background: transparent;
    border: 1px solid var(--border-subtle);
    color: var(--text-muted);
    cursor: pointer;
    transition: all 120ms ease;
  }

  .action-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .action-btn.danger:hover {
    background: rgba(239, 68, 68, 0.15);
    color: #ef4444;
    border-color: rgba(239, 68, 68, 0.3);
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 200px;
    color: var(--text-muted);
    font-size: 0.95rem;
  }

  /* Modals */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.65);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .modal-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    padding: 1.5rem;
    width: 100%;
    max-width: 440px;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    box-shadow: 0 16px 32px rgba(0, 0, 0, 0.35);
  }

  .modal-card h2 {
    font-size: 1.15rem;
    font-weight: 700;
    margin: 0;
    color: var(--text-primary);
  }

  .modal-subtitle {
    font-size: 0.85rem;
    color: var(--text-muted);
    margin: 0;
  }

  .text-input,
  .select-input {
    width: 100%;
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 0.6rem 0.8rem;
    color: var(--text-primary);
    font-size: 0.9rem;
    outline: none;
  }

  .color-palette-select {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    font-size: 0.85rem;
    color: var(--text-muted);
  }

  .palette-dots {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  .palette-dot {
    width: 22px;
    height: 22px;
    border-radius: var(--radius-full);
    border: 2px solid transparent;
    cursor: pointer;
    transition: transform 120ms ease;
  }

  .palette-dot.selected {
    border-color: white;
    transform: scale(1.2);
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: 0.5rem;
  }

  .primary-btn {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    background: var(--accent-primary);
    color: white;
    border: none;
    border-radius: var(--radius-md);
    padding: 0.5rem 1rem;
    font-weight: 600;
    cursor: pointer;
    font-size: 0.85rem;
    transition: filter 150ms ease;
  }

  .primary-btn:hover {
    filter: brightness(1.1);
  }

  .secondary-btn {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 0.5rem 1rem;
    font-weight: 500;
    cursor: pointer;
    font-size: 0.85rem;
  }

  .danger-btn {
    background: #ef4444;
    color: white;
    border: none;
    border-radius: var(--radius-md);
    padding: 0.5rem 1rem;
    font-weight: 600;
    cursor: pointer;
    font-size: 0.85rem;
  }
</style>
