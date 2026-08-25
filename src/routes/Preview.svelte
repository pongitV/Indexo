<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import {
    classifiedFiles,
    currentSessionId,
    selectedFolder,
    showToast,
  } from "../lib/stores";
  import {
    applyOrganization,
    undoLastApply,
    recordUserCorrection,
    listCategories,
    createCategory,
    type Category,
    type ClassifiedFile,
    type FileMove,
  } from "../lib/api";
  import FileTreeNode, { type TreeNodeData } from "../lib/FileTreeNode.svelte";

  let searchQuery = "";
  let isApplying = false;
  let isUndoing = false;
  let showConfirmModal = false;
  let showNewTagModal = false;
  let showCategoryPickerModal = false;

  let allCategories: Category[] = [];
  let ignoredFileIds = new Set<string>();
  let categoryPickerSearch = "";

  // Tree expansion state (Set of expanded folder IDs)
  let expandedBeforeIds = new Set<string>();
  let expandedAfterIds = new Set<string>();

  // Context Menu State
  let contextMenu = {
    visible: false,
    x: 0,
    y: 0,
    file: null as ClassifiedFile | null,
    folder: null as TreeNodeData | null,
  };

  // Selected file for reassignment
  let targetFileForReassign: ClassifiedFile | null = null;
  let newTagNameInput = "";

  onMount(async () => {
    await reloadCategories();
  });

  async function reloadCategories() {
    try {
      allCategories = await listCategories();
    } catch (_) {}
  }

  // Filtragem de arquivos por busca
  $: filteredFiles = $classifiedFiles.filter((f) => {
    if (ignoredFileIds.has(f.file_id)) return false;
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase();
    return (
      f.filename.toLowerCase().includes(q) ||
      f.suggested_category.toLowerCase().includes(q) ||
      f.path.toLowerCase().includes(q)
    );
  });

  // Construção da Árvore "Antes" (Estrutura Atual de Pastas do Disco)
  $: beforeTree = (() => {
    const rootPath = $selectedFolder || "";
    return buildBeforeTree(filteredFiles, rootPath);
  })();

  // Construção da Árvore "Depois" (Estrutura Proposta Organizada por Tags/Categorias)
  $: afterTree = (() => {
    const rootPath = $selectedFolder || "";
    return buildAfterTree(filteredFiles, rootPath, allCategories);
  })();

  let initializedSessionId: string | null = null;

  // Expandir todas as pastas inicialmente na primeira carga
  $: if ($currentSessionId !== initializedSessionId && beforeTree.length > 0 && afterTree.length > 0) {
    initializedSessionId = $currentSessionId;
    const allBefore = new Set<string>();
    collectFolderIds(beforeTree, allBefore);
    expandedBeforeIds = allBefore;

    const allAfter = new Set<string>();
    collectFolderIds(afterTree, allAfter);
    expandedAfterIds = allAfter;
  }

  function collectFolderIds(nodes: TreeNodeData[], set: Set<string>) {
    for (const node of nodes) {
      if (node.isFolder) {
        set.add(node.id);
        if (node.children) {
          collectFolderIds(node.children, set);
        }
      }
    }
  }

  function buildBeforeTree(files: ClassifiedFile[], rootPath: string): TreeNodeData[] {
    const normalizedRoot = rootPath.replace(/\\/g, "/").replace(/\/+$/, "");
    const rootName = normalizedRoot ? normalizedRoot.split("/").pop() || "Pasta Raiz" : "Pasta Raiz";

    const rootNode: TreeNodeData = {
      id: "before-root",
      name: `📁 ${rootName} (Original)`,
      isFolder: true,
      fullPath: rootPath,
      children: [],
      fileCount: 0,
    };

    const folderMap = new Map<string, TreeNodeData>();
    folderMap.set("", rootNode);

    for (const file of files) {
      const normPath = file.path.replace(/\\/g, "/");
      let relPath = "";
      if (normalizedRoot && normPath.toLowerCase().startsWith(normalizedRoot.toLowerCase())) {
        relPath = normPath.slice(normalizedRoot.length).replace(/^\/+/, "");
      } else {
        relPath = file.filename;
      }

      const parts = relPath.split("/").filter(Boolean);
      if (parts.length === 0) continue;

      let currentRel = "";
      let parentNode = rootNode;

      for (let i = 0; i < parts.length - 1; i++) {
        const seg = parts[i];
        currentRel = currentRel ? `${currentRel}/${seg}` : seg;
        const folderId = `before-folder-${currentRel}`;

        if (!folderMap.has(currentRel)) {
          const newFolder: TreeNodeData = {
            id: folderId,
            name: seg,
            isFolder: true,
            fullPath: `${normalizedRoot}/${currentRel}`,
            children: [],
            fileCount: 0,
          };
          parentNode.children = parentNode.children || [];
          parentNode.children.push(newFolder);
          folderMap.set(currentRel, newFolder);
        }
        parentNode = folderMap.get(currentRel)!;
      }

      const fileNode: TreeNodeData = {
        id: `before-file-${file.file_id}`,
        name: file.filename,
        isFolder: false,
        fullPath: file.path,
        file,
        fileCount: 1,
      };
      parentNode.children = parentNode.children || [];
      parentNode.children.push(fileNode);
    }

    function finalizeNode(node: TreeNodeData): number {
      if (!node.isFolder) return 1;
      let count = 0;
      if (node.children) {
        for (const child of node.children) {
          count += finalizeNode(child);
        }
        node.children.sort((a, b) => {
          if (a.isFolder && !b.isFolder) return -1;
          if (!a.isFolder && b.isFolder) return 1;
          return a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: "base" });
        });
      }
      node.fileCount = count;
      return count;
    }

    finalizeNode(rootNode);

    return [rootNode];
  }

  function buildAfterTree(files: ClassifiedFile[], rootPath: string, categories: Category[]): TreeNodeData[] {
    const normalizedRoot = rootPath.replace(/\\/g, "/").replace(/\/+$/, "");
    const rootName = normalizedRoot ? normalizedRoot.split("/").pop() || "Pasta Raiz" : "Pasta Raiz";

    const catColorMap = new Map<string, string>();
    for (const c of categories) {
      if (c.color) catColorMap.set(c.name.toLowerCase(), c.color);
    }

    const rootNode: TreeNodeData = {
      id: "after-root",
      name: `✨ ${rootName} (Organizada)`,
      isFolder: true,
      fullPath: rootPath,
      children: [],
      fileCount: 0,
    };

    const categoryFolders = new Map<string, TreeNodeData>();

    for (const file of files) {
      const catName = file.suggested_category || "Outros";
      const catColor = file.category_color || catColorMap.get(catName.toLowerCase()) || "#3b82f6";
      const catId = `after-cat-${catName}`;

      if (!categoryFolders.has(catName)) {
        const newCatFolder: TreeNodeData = {
          id: catId,
          name: catName,
          isFolder: true,
          fullPath: `${normalizedRoot}/${catName}`,
          categoryColor: catColor,
          categoryName: catName,
          children: [],
          fileCount: 0,
        };
        categoryFolders.set(catName, newCatFolder);
        rootNode.children = rootNode.children || [];
        rootNode.children.push(newCatFolder);
      }

      const catFolder = categoryFolders.get(catName)!;
      const fileNode: TreeNodeData = {
        id: `after-file-${file.file_id}`,
        name: file.filename,
        isFolder: false,
        fullPath: `${normalizedRoot}/${catName}/${file.filename}`,
        file,
        fileCount: 1,
      };
      catFolder.children = catFolder.children || [];
      catFolder.children.push(fileNode);
    }

    function finalizeNode(node: TreeNodeData): number {
      if (!node.isFolder) return 1;
      let count = 0;
      if (node.children) {
        for (const child of node.children) {
          count += finalizeNode(child);
        }
        node.children.sort((a, b) => {
          if (a.isFolder && !b.isFolder) return -1;
          if (!a.isFolder && b.isFolder) return 1;
          return a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: "base" });
        });
      }
      node.fileCount = count;
      return count;
    }

    finalizeNode(rootNode);

    return [rootNode];
  }

  $: filteredPickerCategories = allCategories.filter((c) => {
    if (!categoryPickerSearch.trim()) return true;
    return c.name.toLowerCase().includes(categoryPickerSearch.toLowerCase());
  });

  // Toggle individual folders in Before Tree
  function toggleBeforeFolder(id: string) {
    if (expandedBeforeIds.has(id)) {
      expandedBeforeIds.delete(id);
    } else {
      expandedBeforeIds.add(id);
    }
    expandedBeforeIds = new Set(expandedBeforeIds);
  }

  function expandAllBefore() {
    const all = new Set<string>();
    collectFolderIds(beforeTree, all);
    expandedBeforeIds = all;
  }

  function collapseAllBefore() {
    expandedBeforeIds = new Set();
  }

  // Toggle individual folders in After Tree
  function toggleAfterFolder(id: string) {
    if (expandedAfterIds.has(id)) {
      expandedAfterIds.delete(id);
    } else {
      expandedAfterIds.add(id);
    }
    expandedAfterIds = new Set(expandedAfterIds);
  }

  function expandAllAfter() {
    const all = new Set<string>();
    collectFolderIds(afterTree, all);
    expandedAfterIds = all;
  }

  function collapseAllAfter() {
    expandedAfterIds = new Set();
  }

  // Right-click context menu handlers
  function openFileContextMenu(e: MouseEvent, file: ClassifiedFile) {
    contextMenu = {
      visible: true,
      x: Math.min(e.clientX, window.innerWidth - 300),
      y: Math.min(e.clientY, window.innerHeight - 340),
      file,
      folder: null,
    };
  }

  function openFolderContextMenu(e: MouseEvent, folder: TreeNodeData) {
    contextMenu = {
      visible: true,
      x: Math.min(e.clientX, window.innerWidth - 300),
      y: Math.min(e.clientY, window.innerHeight - 260),
      file: null,
      folder,
    };
  }

  function closeContextMenu() {
    contextMenu.visible = false;
  }

  function handleIgnoreFile(file: ClassifiedFile) {
    ignoredFileIds.add(file.file_id);
    ignoredFileIds = new Set(ignoredFileIds);
    closeContextMenu();
    showToast(`Arquivo '${file.filename}' ignorado da organização.`, "info");
  }

  function handleOpenChangeCategory(file: ClassifiedFile) {
    targetFileForReassign = file;
    categoryPickerSearch = "";
    closeContextMenu();
    showCategoryPickerModal = true;
  }

  function handleOpenNewTag(file: ClassifiedFile) {
    targetFileForReassign = file;
    newTagNameInput = "";
    closeContextMenu();
    showNewTagModal = true;
  }

  async function handleAlwaysRule(file: ClassifiedFile) {
    closeContextMenu();
    try {
      await recordUserCorrection(file.file_id, file.category_id, file.category_id);
      showToast($_("preview.toast.rule_saved"), "success");
    } catch (err: any) {
      showToast("Erro ao gravar regra permanente: " + err, "error");
    }
  }

  async function assignCategoryToFile(category: Category) {
    if (!targetFileForReassign) return;
    const file = targetFileForReassign;
    const oldCatId = file.category_id;

    try {
      await recordUserCorrection(file.file_id, oldCatId, category.id);
      classifiedFiles.update((list) =>
        list.map((item) =>
          item.file_id === file.file_id
            ? {
                ...item,
                suggested_category: category.name,
                category_id: category.id,
                category_color: category.color ?? "#3b82f6",
                confidence: 1.0,
                tier_used: 1,
              }
            : item
        )
      );
      showCategoryPickerModal = false;
      targetFileForReassign = null;
      showToast(`Arquivo '${file.filename}' reatribuído para '${category.name}'!`, "success");
      await reloadCategories();
    } catch (err: any) {
      showToast("Erro ao reatribuir categoria: " + err, "error");
    }
  }

  async function handleCreateNewTag() {
    if (!newTagNameInput.trim() || !targetFileForReassign) return;
    try {
      const newCat = await createCategory(newTagNameInput.trim());
      showNewTagModal = false;
      await assignCategoryToFile(newCat);
    } catch (err: any) {
      showToast("Erro ao criar nova tag: " + err, "error");
    }
  }

  async function handleApplyChanges() {
    if (!$selectedFolder || !$currentSessionId) return;
    showConfirmModal = false;
    isApplying = true;

    try {
      const root = $selectedFolder;
      const moves: FileMove[] = [];

      for (const file of filteredFiles) {
        const sanitizedCat = file.suggested_category.replace(/[<>:"/\\|?*]/g, "_").trim();
        const sep = root.includes("\\") ? "\\" : "/";
        const destPath = `${root}${sep}${sanitizedCat}${sep}${file.filename}`;

        moves.push({
          file_id: file.file_id,
          from_path: file.path,
          to_path: destPath,
        });
      }

      const summary = await applyOrganization($currentSessionId, moves);
      if (summary.failed.length === 0) {
        showToast($_("preview.toast.applied", { values: { count: summary.moved } }), "success");
      } else {
        showToast(
          `${summary.moved} movidos, ${summary.failed.length} falharam: ${summary.failed[0]}`,
          "error"
        );
      }
    } catch (err: any) {
      showToast("Erro ao aplicar organização: " + err, "error");
    } finally {
      isApplying = false;
    }
  }

  async function handleUndo() {
    isUndoing = true;
    try {
      const count = await undoLastApply($currentSessionId);
      if (count > 0) {
        showToast($_("preview.toast.undone", { values: { count } }), "success");
      } else {
        showToast("Nenhuma alteração recente encontrada para desfazer.", "info");
      }
    } catch (err: any) {
      showToast("Erro ao desfazer organização: " + err, "error");
    } finally {
      isUndoing = false;
    }
  }
</script>

<svelte:window on:click={closeContextMenu} />

<div class="preview-layout">
  <!-- Top Action & Search Bar -->
  <div class="preview-header">
    <div class="header-titles">
      <h1>{$_("preview.title")}</h1>
      <p class="subtitle">Árvores de pastas antes e depois. Clique com o botão direito em qualquer arquivo para ver ou alterar sua tag.</p>
    </div>

    <div class="header-controls">
      <!-- Search Input -->
      <div class="search-box">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        <input
          type="text"
          placeholder={$_("preview.search_placeholder")}
          bind:value={searchQuery}
        />
        {#if searchQuery}
          <button class="clear-search" on:click={() => (searchQuery = "")}>✕</button>
        {/if}
      </div>

      <!-- Action Buttons -->
      <div class="button-group">
        <button
          class="secondary-btn"
          disabled={isUndoing || isApplying}
          on:click={handleUndo}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 7v6h6"></path>
            <path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"></path>
          </svg>
          {isUndoing ? $_("preview.undoing") : $_("preview.undo")}
        </button>

        <button
          class="primary-btn"
          disabled={filteredFiles.length === 0 || isApplying || isUndoing}
          on:click={() => (showConfirmModal = true)}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20 6 9 17 4 12"></polyline>
          </svg>
          {isApplying ? $_("preview.applying") : $_("preview.apply")}
          <span class="btn-count">({filteredFiles.length})</span>
        </button>
      </div>
    </div>
  </div>

  <!-- Dual-Column Tree Structure: Antes e Depois -->
  <div class="columns-container">
    <!-- Left Column: Tree Antes (Estrutura Atual) -->
    <section class="preview-column glass-panel">
      <div class="column-header">
        <div class="column-title">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
          </svg>
          <h2>{$_("preview.column.current")}</h2>
        </div>
        <div class="column-meta-actions">
          <span class="column-meta">{filteredFiles.length} {$_("preview.total_files")}</span>
          <div class="tree-controls">
            <button class="mini-btn" on:click={expandAllBefore} title="Expandir todas as pastas">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="7 13 12 18 17 13"></polyline>
                <polyline points="7 6 12 11 17 6"></polyline>
              </svg>
            </button>
            <button class="mini-btn" on:click={collapseAllBefore} title="Recolher todas">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="17 11 12 6 7 11"></polyline>
                <polyline points="17 18 12 13 7 18"></polyline>
              </svg>
            </button>
          </div>
        </div>
      </div>

      <div class="tree-scroll-container">
        {#if beforeTree.length === 0 || filteredFiles.length === 0}
          <div class="empty-state">Nenhum arquivo ou pasta encontrado com os filtros atuais.</div>
        {:else}
          <div class="tree-root">
            {#each beforeTree as node (node.id)}
              <FileTreeNode
                {node}
                expandedIds={expandedBeforeIds}
                onToggleFolder={toggleBeforeFolder}
                onFileContextMenu={openFileContextMenu}
                onFolderContextMenu={openFolderContextMenu}
              />
            {/each}
          </div>
        {/if}
      </div>
    </section>

    <!-- Right Column: Tree Depois (Estrutura Proposta) -->
    <section class="preview-column glass-panel proposed-column">
      <div class="column-header">
        <div class="column-title">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"></path>
            <line x1="7" y1="7" x2="7.01" y2="7"></line>
          </svg>
          <h2>{$_("preview.column.proposed")}</h2>
        </div>
        <div class="column-meta-actions">
          <span class="column-meta">{filteredFiles.length} {$_("preview.total_files")}</span>
          <div class="tree-controls">
            <button class="mini-btn" on:click={expandAllAfter} title="Expandir todas as pastas">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="7 13 12 18 17 13"></polyline>
                <polyline points="7 6 12 11 17 6"></polyline>
              </svg>
            </button>
            <button class="mini-btn" on:click={collapseAllAfter} title="Recolher todas">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="17 11 12 6 7 11"></polyline>
                <polyline points="17 18 12 13 7 18"></polyline>
              </svg>
            </button>
          </div>
        </div>
      </div>

      <div class="tree-scroll-container">
        {#if afterTree.length === 0 || filteredFiles.length === 0}
          <div class="empty-state">Nenhuma categoria agrupada com os filtros atuais.</div>
        {:else}
          <div class="tree-root">
            {#each afterTree as node (node.id)}
              <FileTreeNode
                {node}
                expandedIds={expandedAfterIds}
                onToggleFolder={toggleAfterFolder}
                onFileContextMenu={openFileContextMenu}
                onFolderContextMenu={openFolderContextMenu}
              />
            {/each}
          </div>
        {/if}
      </div>
    </section>
  </div>
</div>

<!-- Context Menu on Right Click (Inspeção de Tags, Detalhes e Ações) -->
{#if contextMenu.visible}
  <div
    class="custom-context-menu"
    style="top: {contextMenu.y}px; left: {contextMenu.x}px;"
    role="menu"
    tabindex="-1"
    on:click|stopPropagation
    on:keydown|stopPropagation
  >
    {#if contextMenu.file}
      <!-- Header do Arquivo com Tag em Destaque -->
      <div class="context-card-header">
        <div class="context-file-title truncate" title={contextMenu.file.filename}>
          📄 {contextMenu.file.filename}
        </div>
        <div class="context-file-path truncate" title={contextMenu.file.path}>
          {contextMenu.file.path}
        </div>

        <!-- Tag / Categoria Atual do Arquivo -->
        <div
          class="context-tag-pill"
          style="background: {contextMenu.file.category_color || '#3b82f6'}18; border-color: {contextMenu.file.category_color || '#3b82f6'}40;"
        >
          <span class="cat-dot" style="background: {contextMenu.file.category_color || '#3b82f6'};"></span>
          <span class="context-tag-name truncate" style="color: {contextMenu.file.category_color || '#3b82f6'};">
            {contextMenu.file.suggested_category}
          </span>
          <span class="context-confidence">
            {Math.round(contextMenu.file.confidence * 100)}%
          </span>
        </div>
      </div>

      <div class="context-divider"></div>

      <!-- Ações do Arquivo -->
      <button
        class="context-item"
        role="menuitem"
        on:click={() => handleOpenChangeCategory(contextMenu.file!)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"></path>
        </svg>
        {$_("preview.context.change_category")}
      </button>

      <button
        class="context-item"
        role="menuitem"
        on:click={() => handleOpenNewTag(contextMenu.file!)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="12" y1="5" x2="12" y2="19"></line>
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
        {$_("preview.context.new_tag")}
      </button>

      <button
        class="context-item"
        role="menuitem"
        on:click={() => handleAlwaysRule(contextMenu.file!)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"></path>
        </svg>
        {$_("preview.context.always_rule")}
      </button>

      <div class="context-divider"></div>

      <button
        class="context-item text-danger"
        role="menuitem"
        on:click={() => handleIgnoreFile(contextMenu.file!)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"></circle>
          <line x1="4.93" y1="4.93" x2="19.07" y2="19.07"></line>
        </svg>
        {$_("preview.context.ignore")}
      </button>

    {:else if contextMenu.folder}
      <!-- Header da Pasta -->
      <div class="context-card-header">
        <div class="context-file-title truncate" title={contextMenu.folder.name}>
          📁 {contextMenu.folder.name}
        </div>
        <div class="context-file-path truncate">
          {contextMenu.folder.fileCount} {contextMenu.folder.fileCount === 1 ? 'arquivo' : 'arquivos'} nesta pasta
        </div>
      </div>

      <div class="context-divider"></div>

      <button
        class="context-item"
        role="menuitem"
        on:click={() => {
          if (contextMenu.folder) {
            if (contextMenu.folder.id.startsWith("before-")) {
              toggleBeforeFolder(contextMenu.folder.id);
            } else {
              toggleAfterFolder(contextMenu.folder.id);
            }
          }
          closeContextMenu();
        }}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="9 18 15 12 9 6"></polyline>
        </svg>
        Alternar Expansão da Pasta
      </button>
    {/if}
  </div>
{/if}

<!-- Modal: Confirmar Aplicação -->
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
      <h2>{$_("preview.modal.apply_title")}</h2>
      <p class="modal-subtitle">{$_("preview.modal.apply_msg", { values: { count: filteredFiles.length } })}</p>

      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showConfirmModal = false)}>
          {$_("preview.modal.cancel")}
        </button>
        <button class="primary-btn" on:click={handleApplyChanges}>
          {$_("preview.modal.confirm")}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Selecionar Categoria Existente -->
{#if showCategoryPickerModal}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (showCategoryPickerModal = false)}
    on:keydown={(e) => e.key === "Escape" && (showCategoryPickerModal = false)}
  >
    <div class="modal-card modal-card-wide">
      <h2>{$_("preview.context.change_category")}</h2>
      <p class="modal-subtitle">Escolha uma categoria para <strong>{targetFileForReassign?.filename}</strong>:</p>

      <!-- Category Filter Search Box -->
      <div class="search-box modal-search">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        <input
          type="text"
          placeholder={$_("preview.search_categories")}
          bind:value={categoryPickerSearch}
        />
        {#if categoryPickerSearch}
          <button class="clear-search" on:click={() => (categoryPickerSearch = "")}>✕</button>
        {/if}
      </div>

      <div class="category-grid-picker">
        {#if filteredPickerCategories.length === 0}
          <div class="empty-picker">Nenhuma categoria encontrada.</div>
        {:else}
          {#each filteredPickerCategories as cat (cat.id)}
            <button
              class="cat-pick-btn"
              title={cat.name}
              on:click={() => assignCategoryToFile(cat)}
            >
              <span class="cat-dot" style="background: {cat.color ?? '#3b82f6'}"></span>
              <span class="cat-pick-name">{cat.name}</span>
            </button>
          {/each}
        {/if}
      </div>

      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showCategoryPickerModal = false)}>
          {$_("preview.modal.cancel")}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Criar Nova Tag -->
{#if showNewTagModal}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (showNewTagModal = false)}
    on:keydown={(e) => e.key === "Escape" && (showNewTagModal = false)}
  >
    <div class="modal-card">
      <h2>{$_("tags.new")}</h2>
      <p class="modal-subtitle">Digite o nome da nova categoria para <strong>{targetFileForReassign?.filename}</strong>:</p>

      <input
        type="text"
        placeholder="Ex: Documentos Financeiros 2026"
        bind:value={newTagNameInput}
        class="text-input"
        on:keydown={(e) => e.key === "Enter" && handleCreateNewTag()}
      />

      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showNewTagModal = false)}>
          {$_("preview.modal.cancel")}
        </button>
        <button class="primary-btn" on:click={handleCreateNewTag}>
          {$_("tags.create")}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .preview-layout {
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

  .preview-header {
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
    color: var(--text-primary);
  }

  .subtitle {
    font-size: 0.84rem;
    color: var(--text-muted);
  }

  .header-controls {
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
    width: 260px;
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

  .button-group {
    display: flex;
    gap: 0.5rem;
    flex-shrink: 0;
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

  .primary-btn:hover:not(:disabled) {
    background: var(--accent-primary-hover);
  }

  .secondary-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.65rem 1.1rem;
    font-size: 0.88rem;
    font-weight: 600;
    border-radius: var(--radius-md);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-medium);
    color: var(--text-primary);
    flex-shrink: 0;
  }

  .secondary-btn:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .btn-count {
    opacity: 0.85;
    font-size: 0.8rem;
  }

  /* Dual Column Tree Container */
  .columns-container {
    flex: 1;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.25rem;
    overflow: hidden;
    min-height: 0;
    min-width: 0;
  }

  .preview-column {
    display: flex;
    flex-direction: column;
    border-radius: var(--radius-lg);
    overflow: hidden;
    background: var(--bg-card);
    border: 1px solid var(--border-subtle);
    min-height: 0;
    min-width: 0;
  }

  .column-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.85rem 1.15rem;
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-tertiary);
    flex-shrink: 0;
    gap: 0.5rem;
  }

  .column-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--text-primary);
    min-width: 0;
  }

  .column-title h2 {
    font-size: 0.95rem;
    font-weight: 700;
    white-space: nowrap;
  }

  .column-meta-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-shrink: 0;
  }

  .column-meta {
    font-size: 0.76rem;
    font-weight: 600;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .tree-controls {
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }

  .mini-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    font-size: 0.72rem;
    font-weight: 600;
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    transition: all var(--transition-fast);
  }

  .mini-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
    border-color: var(--border-medium);
  }

  .tree-scroll-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .tree-root {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    width: 100%;
  }

  .empty-state {
    padding: 3.5rem 1rem;
    text-align: center;
    color: var(--text-muted);
    font-size: 0.88rem;
  }

  /* Context Menu on Right Click */
  .custom-context-menu {
    position: fixed;
    z-index: 2000;
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-xl);
    padding: 0.5rem;
    width: 270px;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    animation: fadeIn 150ms ease-out;
  }

  .context-card-header {
    padding: 0.5rem 0.6rem 0.65rem 0.6rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .context-file-title {
    font-size: 0.88rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .context-file-path {
    font-size: 0.72rem;
    font-family: var(--font-mono);
    color: var(--text-muted);
  }

  .context-tag-pill {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.25rem 0.6rem;
    border-radius: var(--radius-full);
    border: 1px solid transparent;
    font-size: 0.78rem;
    font-weight: 700;
    margin-top: 0.25rem;
    width: fit-content;
    max-width: 100%;
  }

  .context-tag-name {
    flex: 1;
    min-width: 0;
  }

  .context-confidence {
    font-size: 0.72rem;
    opacity: 0.85;
    background: rgba(0, 0, 0, 0.08);
    padding: 0.05rem 0.35rem;
    border-radius: var(--radius-sm);
  }

  .cat-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .context-item {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    padding: 0.55rem 0.7rem;
    font-size: 0.84rem;
    font-weight: 500;
    color: var(--text-primary);
    background: transparent;
    border-radius: var(--radius-sm);
    text-align: left;
    width: 100%;
    flex-shrink: 0;
    transition: all var(--transition-fast);
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
    max-width: 480px;
    width: 100%;
    animation: fadeIn 200ms ease-out;
    display: flex;
    flex-direction: column;
  }

  .modal-card-wide {
    max-width: 580px;
  }

  .modal-card h2 {
    font-size: 1.25rem;
    font-weight: 700;
    margin-bottom: 0.4rem;
  }

  .modal-subtitle {
    font-size: 0.86rem;
    color: var(--text-muted);
    margin-bottom: 1.2rem;
    line-height: 1.4;
  }

  .modal-search {
    width: 100%;
    margin-bottom: 0.85rem;
  }

  .modal-search input {
    width: 100%;
  }

  .text-input {
    width: 100%;
    margin-bottom: 1.5rem;
  }

  .category-grid-picker {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 0.5rem;
    max-height: 300px;
    overflow-y: auto;
    margin-bottom: 1.5rem;
    padding: 0.25rem;
    min-height: 80px;
  }

  .empty-picker {
    grid-column: 1 / -1;
    padding: 2rem 1rem;
    text-align: center;
    color: var(--text-muted);
    font-size: 0.85rem;
  }

  .cat-pick-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.6rem 0.8rem;
    border-radius: var(--radius-md);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-primary);
    text-align: left;
    min-width: 0;
  }

  .cat-pick-btn:hover {
    background: var(--bg-hover);
    border-color: var(--accent-primary);
  }

  .cat-pick-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: block;
    flex: 1;
    min-width: 0;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.6rem;
    margin-top: 0.5rem;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(6px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
