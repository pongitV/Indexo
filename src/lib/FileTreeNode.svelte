<script lang="ts">
  import type { ClassifiedFile } from "./api";
  import FileTreeNode from "./FileTreeNode.svelte";

  export interface TreeNodeData {
    id: string;
    name: string;
    isFolder: boolean;
    fullPath: string;
    file?: ClassifiedFile;
    children?: TreeNodeData[];
    fileCount: number;
    categoryColor?: string;
    categoryName?: string;
    isPreservedFolder?: boolean;
    isPreservedFile?: boolean;
  }

  export let node: TreeNodeData;
  export let expandedIds: Set<string>;
  export let highlightedNodeId: string | null = null;
  export let selectedFileIds: Set<string> | undefined = undefined;
  export let onToggleFolder: (id: string) => void;
  export let onFileContextMenu: (e: MouseEvent, file: ClassifiedFile) => void;
  export let onFolderContextMenu: ((e: MouseEvent, node: TreeNodeData) => void) | undefined = undefined;
  export let onFileClick: ((e: MouseEvent, file: ClassifiedFile) => void) | undefined = undefined;

  $: isExpanded = expandedIds.has(node.id);
  $: isHighlighted = highlightedNodeId === node.id;
  $: isSelected = node.file && selectedFileIds ? selectedFileIds.has(node.file.file_id) : false;

  function getFileExtension(filename: string): string {
    const parts = filename.split(".");
    return parts.length > 1 ? parts.pop()?.toLowerCase() || "" : "";
  }

  function getFileIconColor(ext: string): string {
    switch (ext) {
      case "pdf": return "#ef4444";
      case "png":
      case "jpg":
      case "jpeg":
      case "webp":
      case "gif":
      case "svg": return "#10b981";
      case "doc":
      case "docx":
      case "txt":
      case "md": return "#3b82f6";
      case "xls":
      case "xlsx":
      case "csv": return "#059669";
      case "zip":
      case "rar":
      case "7z":
      case "tar":
      case "gz": return "#f59e0b";
      case "mp3":
      case "wav":
      case "flac": return "#8b5cf6";
      case "mp4":
      case "mkv":
      case "avi": return "#ec4899";
      case "js":
      case "ts":
      case "html":
      case "css":
      case "rs":
      case "py": return "#06b6d4";
      default: return "var(--text-muted)";
    }
  }

  function handleFolderClick(e: MouseEvent) {
    e.stopPropagation();
    onToggleFolder(node.id);
  }

  function handleFolderKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onToggleFolder(node.id);
    }
  }

  function handleItemClick(e: MouseEvent) {
    e.stopPropagation();
    if (node.isFolder) {
      onToggleFolder(node.id);
    } else if (node.file) {
      if (onFileClick) {
        onFileClick(e, node.file);
      } else {
        onFileContextMenu(e, node.file);
      }
    }
  }

  function handleRightClick(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    if (node.isFolder) {
      if (onFolderContextMenu) {
        onFolderContextMenu(e, node);
      }
    } else if (node.file) {
      onFileContextMenu(e, node.file);
    }
  }
</script>

{#if node.isFolder}
  <div class="tree-branch" id={node.id}>
    <!-- Folder Row -->
    <div
      class="tree-row folder-row"
      class:expanded={isExpanded}
      class:highlighted={isHighlighted}
      role="button"
      tabindex="0"
      title="Pasta: {node.name} ({node.fileCount} {node.fileCount === 1 ? 'arquivo' : 'arquivos'})&#10;Clique para {isExpanded ? 'recolher' : 'expandir'} | Clique direito para opções"
      on:click={handleFolderClick}
      on:keydown={handleFolderKeydown}
      on:contextmenu={handleRightClick}
    >
      <!-- Chevron -->
      <div class="chevron-box" class:rotated={isExpanded}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <polyline points="9 18 15 12 9 6"></polyline>
        </svg>
      </div>

      <!-- Folder Icon -->
      <div class="row-icon folder-icon" style={node.categoryColor ? `color: ${node.categoryColor}` : ''}>
        {#if isExpanded}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
            <polyline points="2 10 22 10"></polyline>
          </svg>
        {:else}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
          </svg>
        {/if}
      </div>

      <!-- Folder Name -->
      <span class="row-label folder-label truncate">{node.name}</span>

      <!-- Preserved Folder Badge -->
      {#if node.isPreservedFolder}
        <span class="row-preserved-badge" title="Pasta já organizada detectada (Estrutura original preservada para revisão)">
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
          </svg>
          Preservada
        </span>
      {/if}

      <!-- File Count Badge -->
      <span class="row-badge folder-badge">{node.fileCount}</span>
    </div>

    <!-- Children Nodes (Nested Tree) -->
    {#if isExpanded && node.children && node.children.length > 0}
      <div class="tree-children">
        {#each node.children as child (child.id)}
          <FileTreeNode
            node={child}
            {expandedIds}
            {highlightedNodeId}
            {selectedFileIds}
            {onToggleFolder}
            {onFileContextMenu}
            {onFolderContextMenu}
            {onFileClick}
          />
        {/each}
      </div>
    {/if}
  </div>
{:else}
  <!-- File Row -->
  {@const ext = getFileExtension(node.name)}
  <div
    class="tree-row file-row"
    id={node.id}
    class:highlighted={isHighlighted}
    class:selected={isSelected}
    role="button"
    tabindex="0"
    title="{node.name}&#10;Tag / Categoria: {node.file?.suggested_category ?? 'Sem tag'}&#10;Caminho: {node.fullPath}&#10;Ctrl+Clique para multi-seleção | Clique direito para opções"
    on:contextmenu={handleRightClick}
    on:click={handleItemClick}
    on:keydown={(e) => (e.key === "Enter" || e.key === " ") && handleItemClick(e as any)}
  >
    <!-- Checkbox/Spacer for selection -->
    {#if isSelected}
      <div class="selection-check-icon">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
          <polyline points="20 6 9 17 4 12"></polyline>
        </svg>
      </div>
    {:else}
      <div class="chevron-spacer"></div>
    {/if}

    <!-- File Icon -->
    <div class="row-icon file-icon" style="color: {getFileIconColor(ext)};">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
        <polyline points="14 2 14 8 20 8"></polyline>
      </svg>
    </div>

    <!-- File Name -->
    <span class="row-label file-label truncate">{node.name}</span>

    <!-- Preserved File Badge -->
    {#if node.file?.is_already_organized || node.isPreservedFile}
      <span class="file-preserved-badge" title="Estrutura original preservada para revisão">
        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
        </svg>
        Preservado
      </span>
    {/if}

    <!-- Right-click prompt badge -->
    <span class="right-click-hint">opções</span>
  </div>
{/if}

<style>
  .tree-branch {
    display: flex;
    flex-direction: column;
    width: 100%;
    flex-shrink: 0;
  }

  .tree-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.38rem 0.6rem;
    border-radius: var(--radius-sm);
    transition: all var(--transition-fast);
    user-select: none;
    flex-shrink: 0;
    min-height: 34px;
    width: 100%;
    box-sizing: border-box;
    cursor: pointer;
  }

  .folder-row {
    color: var(--text-primary);
    font-weight: 600;
  }

  .folder-row:hover {
    background: var(--bg-hover);
  }

  .folder-row.expanded {
    background: rgba(13, 148, 136, 0.05);
  }

  .file-row {
    color: var(--text-secondary);
    border-left: 2px solid transparent;
  }

  .file-row:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
    border-left-color: var(--accent-primary);
  }

  .chevron-box {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    color: var(--text-muted);
    transition: transform 180ms ease;
    flex-shrink: 0;
  }

  .chevron-box.rotated {
    transform: rotate(90deg);
    color: var(--accent-primary);
  }

  .tree-row.selected {
    background: rgba(59, 130, 246, 0.18) !important;
    border-color: var(--accent-primary) !important;
    box-shadow: inset 0 0 0 1px var(--accent-primary);
  }

  .selection-check-icon {
    width: 16px;
    height: 16px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--accent-primary);
    color: white;
    border-radius: var(--radius-sm);
    flex-shrink: 0;
  }

  .chevron-spacer {
    width: 16px;
    height: 16px;
    flex-shrink: 0;
  }

  .row-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .folder-icon {
    color: var(--accent-primary);
  }

  .file-icon {
    opacity: 0.9;
  }

  .row-label {
    font-size: 0.84rem;
    flex: 1;
    min-width: 0;
    line-height: 1.3;
  }

  .folder-label {
    font-weight: 600;
    color: var(--text-primary);
  }

  .file-label {
    font-weight: 500;
    color: var(--text-primary);
  }

  .row-badge {
    font-size: 0.7rem;
    font-weight: 700;
    padding: 0.1rem 0.45rem;
    border-radius: var(--radius-full);
    background: var(--bg-tertiary);
    color: var(--text-muted);
    flex-shrink: 0;
    margin-left: auto;
  }

  .row-preserved-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    font-size: 0.68rem;
    font-weight: 600;
    padding: 0.12rem 0.45rem;
    border-radius: var(--radius-sm);
    background: rgba(16, 185, 129, 0.12);
    color: #10b981;
    border: 1px solid rgba(16, 185, 129, 0.25);
    flex-shrink: 0;
  }

  .file-preserved-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.2rem;
    font-size: 0.65rem;
    font-weight: 600;
    padding: 0.08rem 0.4rem;
    border-radius: var(--radius-sm);
    background: rgba(16, 185, 129, 0.1);
    color: #10b981;
    border: 1px solid rgba(16, 185, 129, 0.2);
    flex-shrink: 0;
  }

  .right-click-hint {
    font-size: 0.68rem;
    font-weight: 600;
    color: var(--text-muted);
    opacity: 0;
    transition: opacity var(--transition-fast);
    flex-shrink: 0;
    margin-left: auto;
    background: var(--bg-tertiary);
    padding: 0.1rem 0.35rem;
    border-radius: var(--radius-sm);
  }

  .file-row:hover .right-click-hint {
    opacity: 0.85;
  }

  .tree-children {
    display: flex;
    flex-direction: column;
    width: 100%;
    border-left: 1px dashed var(--border-medium);
    margin-left: 12px;
    padding-left: 8px;
    gap: 0.15rem;
  }

  .tree-row.highlighted {
    background: rgba(59, 130, 246, 0.25) !important;
    border-color: var(--accent-primary) !important;
    box-shadow: 0 0 16px var(--accent-glow) !important;
    animation: highlightPulse 1.5s ease-in-out infinite alternate;
  }

  @keyframes highlightPulse {
    0% {
      transform: scale(1);
      box-shadow: 0 0 8px var(--accent-glow);
    }
    100% {
      transform: scale(1.02);
      box-shadow: 0 0 20px var(--accent-primary);
    }
  }
</style>
