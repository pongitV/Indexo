<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import {
    listCategories,
    createCategory,
    renameCategory,
    mergeCategories,
    deleteCategory,
    type Category,
  } from "../lib/api";
  import { showToast } from "../lib/stores";

  let categories: Category[] = [];
  let searchQuery = "";
  let isLoading = true;

  // Modals state
  let showNewModal = false;
  let showRenameModal = false;
  let showMergeModal = false;
  let showDeleteModal = false;

  let activeCategory: Category | null = null;
  let inputName = "";
  let selectedTargetCategoryId = "";

  onMount(async () => {
    await fetchCategories();
  });

  async function fetchCategories() {
    isLoading = true;
    try {
      categories = await listCategories();
    } catch (err: any) {
      showToast("Erro ao carregar categorias: " + err, "error");
    } finally {
      isLoading = false;
    }
  }

  $: filteredCategories = categories.filter((c) => {
    if (!searchQuery.trim()) return true;
    return c.name.toLowerCase().includes(searchQuery.toLowerCase());
  });

  function openNewModal() {
    inputName = "";
    showNewModal = true;
  }

  function openRenameModal(cat: Category) {
    activeCategory = cat;
    inputName = cat.name;
    showRenameModal = true;
  }

  function openMergeModal(cat: Category) {
    activeCategory = cat;
    const remaining = categories.filter((c) => c.id !== cat.id);
    selectedTargetCategoryId = remaining[0]?.id || "";
    showMergeModal = true;
  }

  function openDeleteModal(cat: Category) {
    activeCategory = cat;
    showDeleteModal = true;
  }

  async function handleCreate() {
    if (!inputName.trim()) return;
    try {
      await createCategory(inputName.trim());
      showNewModal = false;
      showToast("Categoria criada com sucesso!", "success");
      await fetchCategories();
    } catch (err: any) {
      showToast("Erro ao criar categoria: " + err, "error");
    }
  }

  async function handleRename() {
    if (!activeCategory || !inputName.trim()) return;
    try {
      await renameCategory(activeCategory.id, inputName.trim());
      showRenameModal = false;
      showToast("Categoria renomeada com sucesso!", "success");
      await fetchCategories();
    } catch (err: any) {
      showToast("Erro ao renomear: " + err, "error");
    }
  }

  async function handleMerge() {
    if (!activeCategory || !selectedTargetCategoryId) return;
    try {
      await mergeCategories(activeCategory.id, selectedTargetCategoryId);
      showMergeModal = false;
      showToast("Categorias mescladas com sucesso!", "success");
      await fetchCategories();
    } catch (err: any) {
      showToast("Erro ao mesclar categorias: " + err, "error");
    }
  }

  async function handleDelete() {
    if (!activeCategory) return;
    try {
      await deleteCategory(activeCategory.id);
      showDeleteModal = false;
      showToast("Categoria excluída com sucesso!", "success");
      await fetchCategories();
    } catch (err: any) {
      showToast("Erro ao excluir categoria: " + err, "error");
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
          placeholder="Buscar categoria..."
          bind:value={searchQuery}
        />
        {#if searchQuery}
          <button class="clear-search" on:click={() => (searchQuery = "")}>✕</button>
        {/if}
      </div>

      <button class="primary-btn" on:click={openNewModal}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19"></line>
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
        {$_("tags.new")}
      </button>
    </div>
  </div>

  <!-- Tag Cards Grid -->
  <div class="tags-container">
    {#if isLoading}
      <div class="empty-state">Carregando categorias...</div>
    {:else if filteredCategories.length === 0}
      <div class="empty-state">Nenhuma categoria encontrada.</div>
    {:else}
      <div class="tags-grid">
        {#each filteredCategories as cat (cat.id)}
          <div class="tag-card glass-panel" style="border-top: 3px solid {cat.color ?? '#3b82f6'}">
            <div class="tag-main">
              <div class="tag-title-row" title={cat.name}>
                <span class="tag-dot" style="background: {cat.color ?? '#3b82f6'}"></span>
                <span class="tag-name">{cat.name}</span>
              </div>
              <div class="tag-meta-row">
                <span class="tag-origin-badge {cat.created_by}">
                  {cat.created_by === "auto" ? $_("tags.created_by_auto") : $_("tags.created_by_user")}
                </span>
                <span class="tag-file-count">
                  {$_("tags.files_count", { values: { count: cat.file_count } })}
                </span>
              </div>
            </div>

            <div class="tag-actions-row">
              <button class="action-btn" title={$_("tags.rename")} on:click={() => openRenameModal(cat)}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                  <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                </svg>
                {$_("tags.rename")}
              </button>

              <button class="action-btn" title={$_("tags.merge")} on:click={() => openMergeModal(cat)}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="16 3 21 3 21 8"></polyline>
                  <line x1="4" y1="20" x2="21" y2="3"></line>
                  <polyline points="21 16 21 21 16 21"></polyline>
                  <line x1="15" y1="15" x2="21" y2="21"></line>
                  <line x1="4" y1="4" x2="9" y2="9"></line>
                </svg>
                {$_("tags.merge")}
              </button>

              <button class="action-btn danger" title={$_("tags.delete")} on:click={() => openDeleteModal(cat)}>
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

<!-- Modal: Criar Nova Categoria -->
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
      <p class="modal-subtitle">Defina o nome da nova categoria:</p>

      <input
        type="text"
        placeholder="Ex: Documentos Financeiros 2026"
        bind:value={inputName}
        class="text-input"
        on:keydown={(e) => e.key === "Enter" && handleCreate()}
      />

      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showNewModal = false)}>
          {$_("preview.modal.cancel")}
        </button>
        <button class="primary-btn" on:click={handleCreate}>
          {$_("tags.create")}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Renomear Categoria -->
{#if showRenameModal}
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
      <p class="modal-subtitle">Altere o nome da categoria:</p>

      <input
        type="text"
        bind:value={inputName}
        class="text-input"
        on:keydown={(e) => e.key === "Enter" && handleRename()}
      />

      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showRenameModal = false)}>
          {$_("preview.modal.cancel")}
        </button>
        <button class="primary-btn" on:click={handleRename}>
          {$_("tags.rename")}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Mesclar Categoria -->
{#if showMergeModal && activeCategory}
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
        {$_("tags.modal.merge_desc", { values: { source: activeCategory.name } })}
      </p>

      <select class="select-input" bind:value={selectedTargetCategoryId}>
        {#each categories.filter((c) => c.id !== activeCategory?.id) as targetCat (targetCat.id)}
          <option value={targetCat.id}>{targetCat.name} ({targetCat.file_count} arquivos)</option>
        {/each}
      </select>

      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showMergeModal = false)}>
          {$_("preview.modal.cancel")}
        </button>
        <button class="primary-btn" on:click={handleMerge}>
          {$_("tags.merge")}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Excluir Categoria -->
{#if showDeleteModal && activeCategory}
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
        {$_("tags.modal.delete_desc", { values: { name: activeCategory.name } })}
      </p>

      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showDeleteModal = false)}>
          {$_("preview.modal.cancel")}
        </button>
        <button class="danger-btn" on:click={handleDelete}>
          {$_("tags.delete")}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .tags-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 1.25rem 1.5rem;
    gap: 1rem;
    overflow: hidden;
    min-height: 0;
    min-width: 0;
    animation: fadeIn 250ms ease-out;
  }

  .tags-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 1rem;
    flex-shrink: 0;
  }

  .header-titles h1 {
    font-size: 1.4rem;
    font-weight: 800;
    letter-spacing: -0.02em;
  }

  .subtitle {
    font-size: 0.84rem;
    color: var(--text-muted);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-shrink: 0;
  }

  .search-box {
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-box svg {
    position: absolute;
    left: 0.85rem;
    color: var(--text-muted);
    pointer-events: none;
  }

  .search-box input {
    padding-left: 2.3rem;
    padding-right: 2rem;
    width: 220px;
    font-size: 0.88rem;
  }

  .clear-search {
    position: absolute;
    right: 0.6rem;
    background: transparent;
    color: var(--text-muted);
    font-size: 0.8rem;
    padding: 0.2rem;
  }

  .primary-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.65rem 1.25rem;
    font-size: 0.88rem;
    font-weight: 600;
    border-radius: var(--radius-md);
    background: var(--accent-primary);
    color: white;
    box-shadow: 0 4px 12px var(--accent-glow);
    flex-shrink: 0;
  }

  .primary-btn:hover {
    background: var(--accent-primary-hover);
  }

  .tags-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding-right: 0.25rem;
    min-height: 0;
  }

  .tags-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 1rem;
    padding-bottom: 1rem;
  }

  .tag-card {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 1.15rem;
    border-radius: var(--radius-md);
    background: var(--bg-card);
    border: 1px solid var(--border-subtle);
    gap: 0.9rem;
    transition: all var(--transition-fast);
    flex-shrink: 0;
    min-height: fit-content;
  }

  .tag-card:hover {
    transform: translateY(-2px);
    box-shadow: var(--shadow-lg);
    border-color: var(--border-medium);
  }

  .tag-main {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-width: 0;
  }

  .tag-title-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .tag-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .tag-name {
    font-size: 0.98rem;
    font-weight: 700;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: block;
    flex: 1;
    min-width: 0;
  }

  .tag-meta-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.78rem;
    gap: 0.5rem;
  }

  .tag-origin-badge {
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-full);
    font-weight: 600;
    font-size: 0.72rem;
    white-space: nowrap;
  }

  .tag-origin-badge.auto {
    background: var(--accent-light);
    color: var(--accent-primary);
  }

  .tag-origin-badge.user {
    background: rgba(59, 130, 246, 0.15);
    color: var(--accent-blue);
  }

  .tag-file-count {
    color: var(--text-muted);
    font-weight: 600;
    white-space: nowrap;
  }

  .tag-actions-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--border-subtle);
    flex-wrap: wrap;
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.4rem 0.65rem;
    font-size: 0.78rem;
    font-weight: 600;
    border-radius: var(--radius-sm);
    background: var(--bg-tertiary);
    color: var(--text-secondary);
    border: 1px solid var(--border-subtle);
    transition: all var(--transition-fast);
    flex-shrink: 0;
  }

  .action-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .action-btn.danger:hover {
    background: rgba(244, 63, 94, 0.15);
    color: var(--accent-rose);
    border-color: var(--accent-rose);
  }

  .empty-state {
    padding: 4rem 1rem;
    text-align: center;
    color: var(--text-muted);
    font-size: 0.95rem;
  }

  /* Modals */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.65);
    backdrop-filter: blur(6px);
    z-index: 3000;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
    overflow-y: auto;
  }

  .modal-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-xl);
    padding: 2rem;
    max-width: 460px;
    width: 100%;
    animation: fadeIn 200ms ease-out;
  }

  .modal-card h2 {
    font-size: 1.25rem;
    font-weight: 700;
    margin-bottom: 0.4rem;
  }

  .modal-subtitle {
    font-size: 0.88rem;
    color: var(--text-muted);
    margin-bottom: 1.25rem;
    line-height: 1.4;
  }

  .text-input, .select-input {
    width: 100%;
    margin-bottom: 1.5rem;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.6rem;
  }

  .secondary-btn {
    padding: 0.65rem 1.1rem;
    font-size: 0.88rem;
    font-weight: 600;
    border-radius: var(--radius-md);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-medium);
    color: var(--text-primary);
  }

  .danger-btn {
    padding: 0.65rem 1.2rem;
    font-size: 0.88rem;
    font-weight: 600;
    border-radius: var(--radius-md);
    background: var(--accent-rose);
    color: white;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(6px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
