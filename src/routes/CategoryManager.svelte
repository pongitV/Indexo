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

  let categories: Category[] = [];
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

  let activeCategory: Category | null = null;
  let inputName = "";
  let inputColor = "#3b82f6";
  let selectedTargetCategoryId = "";

  const presetColors = [
    "#3b82f6", "#06b6d4", "#10b981", "#84cc16",
    "#f59e0b", "#f97316", "#ef4444", "#ec4899",
    "#8b5cf6", "#6366f1", "#64748b"
  ];

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
    if (activeTab === "manual" && c.created_by !== "user") return false;
    if (activeTab === "auto" && c.created_by !== "auto") return false;
    if (!searchQuery.trim()) return true;
    return c.name.toLowerCase().includes(searchQuery.toLowerCase());
  });

  function openNewModal() {
    inputName = "";
    inputColor = presetColors[Math.floor(Math.random() * presetColors.length)];
    showNewModal = true;
  }

  function openRenameModal(cat: Category) {
    activeCategory = cat;
    inputName = cat.name;
    inputColor = cat.color ?? "#3b82f6";
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

  function openHistoryModal(cat: Category) {
    activeCategory = cat;
    showHistoryModal = true;
  }

  async function handleCreate() {
    if (!inputName.trim()) return;
    try {
      await createCategory(inputName.trim(), inputColor);
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
      showToast("Erro ao renomear categoria: " + err, "error");
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

  async function handlePurgeAuto() {
    try {
      const count = await purgeAutoCategories();
      showPurgeModal = false;
      showToast($_("categories.toast.cleaned", { values: { count } }), "success");
      await fetchCategories();
    } catch (err: any) {
      showToast("Erro ao limpar categorias automáticas: " + err, "error");
    }
  }

  async function handleCleanUnused() {
    try {
      const count = await cleanUnusedCategories();
      showToast($_("categories.toast.cleaned", { values: { count } }), "success");
      await fetchCategories();
    } catch (err: any) {
      showToast("Erro ao limpar categorias não utilizadas: " + err, "error");
    }
  }
</script>

<div class="category-view">
  <!-- Header with search & new category CTA -->
  <div class="category-header">
    <div class="header-titles">
      <h1>{$_("categories.manage")}</h1>
      <p class="subtitle">{$_("categories.subtitle")}</p>
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

      {#if categories.some(c => c.created_by === "auto")}
        <button class="secondary-btn danger-hover" on:click={() => (showPurgeModal = true)} title={$_("categories.clean_auto")}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6"></polyline>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
          </svg>
          {$_("categories.clean_auto")}
        </button>
      {/if}

      <button class="secondary-btn" on:click={() => currentView.set("rules")} title="Gerenciar heurísticas, extensões e subpastas">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path>
          <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path>
        </svg>
        Regras & Heurísticas
      </button>

      <button class="primary-btn" on:click={openNewModal}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19"></line>
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
        {$_("categories.new")}
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
      {$_("categories.tab.all")}
      <span class="tab-count">{categories.length}</span>
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === "manual"}
      on:click={() => (activeTab = "manual")}
    >
      {$_("categories.tab.manual")}
      <span class="tab-count">{categories.filter(c => c.created_by === "user").length}</span>
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === "auto"}
      on:click={() => (activeTab = "auto")}
    >
      {$_("categories.tab.auto")}
      <span class="tab-count">{categories.filter(c => c.created_by === "auto").length}</span>
    </button>
  </div>

  <!-- Category Cards Grid -->
  <div class="categories-container">
    {#if isLoading}
      <div class="empty-state">Carregando categorias...</div>
    {:else if filteredCategories.length === 0}
      <div class="empty-state">Nenhuma categoria encontrada.</div>
    {:else}
      <div class="categories-grid">
        {#each filteredCategories as cat (cat.id)}
          <div class="cat-card glass-panel" style="border-top: 3px solid {cat.color ?? '#3b82f6'}">
            <div class="cat-main">
              <div class="cat-title-row" title={cat.name}>
                <span class="cat-dot" style="background: {cat.color ?? '#3b82f6'}"></span>
                <span class="cat-name">{cat.name}</span>
              </div>
              <div class="cat-meta-row">
                <span class="cat-origin-badge {cat.created_by}">
                  {cat.created_by === "auto" ? $_("categories.created_by_auto") : $_("categories.created_by_user")}
                </span>
                <span class="cat-file-count">
                  {$_("categories.files_count", { values: { count: cat.file_count } })}
                </span>
              </div>
            </div>

            <div class="cat-actions-row">
              <button class="action-btn" title="Histórico de Mudanças" on:click={() => openHistoryModal(cat)}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"></circle>
                  <polyline points="12 6 12 12 16 14"></polyline>
                </svg>
              </button>

              <button class="action-btn" title={$_("categories.rename")} on:click={() => openRenameModal(cat)}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                  <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                </svg>
              </button>

              <button class="action-btn" title={$_("categories.merge")} on:click={() => openMergeModal(cat)}>
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="16 3 21 3 21 8"></polyline>
                  <line x1="4" y1="20" x2="21" y2="3"></line>
                  <polyline points="21 16 21 21 16 21"></polyline>
                  <line x1="15" y1="15" x2="21" y2="21"></line>
                </svg>
              </button>

              <button class="action-btn danger" title={$_("categories.delete")} on:click={() => openDeleteModal(cat)}>
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
      <h2>{$_("categories.new")}</h2>
      <input
        type="text"
        placeholder={$_("categories.name")}
        bind:value={inputName}
        class="text-input"
        on:keydown={(e) => e.key === "Enter" && handleCreate()}
      />
      <div class="color-palette-select">
        <label for="cat-color-picker">Cor de Destaque:</label>
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
        <button class="primary-btn" on:click={handleCreate}>{$_("categories.create")}</button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Renomear Categoria -->
{#if showRenameModal && activeCategory}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (showRenameModal = false)}
    on:keydown={(e) => e.key === "Escape" && (showRenameModal = false)}
  >
    <div class="modal-card">
      <h2>{$_("categories.modal.rename_title")}</h2>
      <input
        type="text"
        placeholder={$_("categories.name")}
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

<!-- Modal: Mesclar Categorias -->
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
      <h2>{$_("categories.modal.merge_title")}</h2>
      <p class="modal-subtitle">
        {$_("categories.modal.merge_desc", { values: { source: activeCategory.name } })}
      </p>
      <select bind:value={selectedTargetCategoryId} class="select-input">
        {#each categories.filter((c) => c.id !== activeCategory?.id) as targetCat}
          <option value={targetCat.id}>{targetCat.name} ({targetCat.file_count} arquivos)</option>
        {/each}
      </select>
      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showMergeModal = false)}>Cancelar</button>
        <button class="primary-btn" on:click={handleMerge}>Mesclar</button>
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
      <h2>{$_("categories.modal.delete_title")}</h2>
      <p class="modal-subtitle">
        {$_("categories.modal.delete_desc", { values: { name: activeCategory.name } })}
      </p>
      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showDeleteModal = false)}>Cancelar</button>
        <button class="danger-btn" on:click={handleDelete}>Excluir</button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Limpar Categorias Automáticas -->
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
      <h2>{$_("categories.modal.purge_title")}</h2>
      <p class="modal-subtitle">
        {$_("categories.modal.purge_desc")}
      </p>
      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showPurgeModal = false)}>Cancelar</button>
        <button class="secondary-btn" on:click={handleCleanUnused}>{$_("categories.clean_unused")}</button>
        <button class="danger-btn" on:click={handlePurgeAuto}>{$_("categories.clean_auto")}</button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Histórico de Mudanças da Categoria -->
<CategoryHistoryModal
  show={showHistoryModal}
  categoryId={activeCategory?.id ?? null}
  categoryName={activeCategory?.name ?? ""}
  categoryColor={activeCategory?.color ?? null}
  onClose={() => (showHistoryModal = false)}
/>

<style>
  .category-view {
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

  .category-header {
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

  .categories-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    min-height: 0;
  }

  .categories-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 1rem;
    padding-bottom: 1rem;
  }

  .cat-card {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 1rem;
    border-radius: var(--radius-md);
    gap: 0.85rem;
    transition: transform 150ms ease, box-shadow 150ms ease;
  }

  .cat-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 6px 16px rgba(0, 0, 0, 0.12);
  }

  .cat-title-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-weight: 600;
    font-size: 0.95rem;
    color: var(--text-primary);
  }

  .cat-dot {
    width: 10px;
    height: 10px;
    border-radius: var(--radius-full);
    flex-shrink: 0;
  }

  .cat-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .cat-meta-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 0.4rem;
    font-size: 0.75rem;
  }

  .cat-origin-badge {
    padding: 0.15rem 0.45rem;
    border-radius: var(--radius-sm);
    font-weight: 500;
  }

  .cat-origin-badge.auto {
    background: rgba(139, 92, 246, 0.15);
    color: #a78bfa;
  }

  .cat-origin-badge.user {
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
  }

  .cat-file-count {
    color: var(--text-muted);
  }

  .cat-actions-row {
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
