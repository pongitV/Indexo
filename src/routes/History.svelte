<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import {
    getOrganizationHistory,
    undoSession,
    openInExplorer,
    type OrganizationSessionSummary,
    type SessionFileInfo,
    type SessionCategoryInfo,
    type SessionRenameInfo,
  } from "../lib/api";
  import FileTreeNode, { type TreeNodeData } from "../lib/FileTreeNode.svelte";
  import { currentView, showToast } from "../lib/stores";

  let sessions: OrganizationSessionSummary[] = [];
  let isLoading: boolean = true;
  let searchQuery: string = "";

  // Session active tab: map from session_id to active tab
  // tab options: 'tree' | 'categories' | 'tags' | 'moves' | 'renames'
  type TabType = "tree" | "categories" | "tags" | "moves" | "renames";
  let sessionTabs: Record<string, TabType> = {};
  let sessionExpanded: Record<string, boolean> = {};

  // Track expanded folder nodes inside each session's side-by-side trees
  let sessionExpandedTrees: Record<string, Set<string>> = {};

  // Filter for renames tab in each session: 'all' | 'applied' | 'preview'
  let renameFilters: Record<string, "all" | "applied" | "preview"> = {};

  let undoingSessionId: string | null = null;
  let confirmingUndoSession: OrganizationSessionSummary | null = null;

  onMount(async () => {
    await fetchHistory();
  });

  async function fetchHistory() {
    isLoading = true;
    try {
      sessions = await getOrganizationHistory();
      // Inicializar abas default para cada sessão (todas fechadas por padrão)
      for (const s of sessions) {
        if (!sessionTabs[s.session_id]) {
          sessionTabs[s.session_id] = s.files_moved_count > 0 ? "moves" : "tree";
        }
        if (sessionExpanded[s.session_id] === undefined) {
          sessionExpanded[s.session_id] = false; // Iniciar FECHADO conforme solicitado
        }
        if (!renameFilters[s.session_id]) {
          renameFilters[s.session_id] = "all";
        }
      }
    } catch (err: any) {
      showToast("Erro ao carregar histórico: " + err, "error");
    } finally {
      isLoading = false;
    }
  }

  function setTab(sessionId: string, tab: TabType) {
    sessionTabs[sessionId] = tab;
    sessionExpanded[sessionId] = true;
    sessionTabs = { ...sessionTabs };
    sessionExpanded = { ...sessionExpanded };
  }

  function toggleExpand(sessionId: string) {
    sessionExpanded[sessionId] = !sessionExpanded[sessionId];
    sessionExpanded = { ...sessionExpanded };
  }

  function toggleAllSessions(expand: boolean) {
    for (const s of sessions) {
      sessionExpanded[s.session_id] = expand;
    }
    sessionExpanded = { ...sessionExpanded };
  }

  function formatDateTime(dateStr: string): string {
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

  function formatBytes(bytes: number): string {
    if (!bytes || bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
  }

  function promptUndo(session: OrganizationSessionSummary) {
    confirmingUndoSession = session;
  }

  async function handleConfirmUndo() {
    if (!confirmingUndoSession) return;
    const sId = confirmingUndoSession.session_id;
    undoingSessionId = sId;
    confirmingUndoSession = null;

    try {
      const count = await undoSession(sId);
      showToast(`${count} arquivo(s) restaurados para a pasta original!`, "success");
      await fetchHistory();
    } catch (err: any) {
      showToast("Erro ao desfazer organização: " + err, "error");
    } finally {
      undoingSessionId = null;
    }
  }

  async function handleOpenExplorer(path: string) {
    try {
      await openInExplorer(path);
    } catch (e: any) {
      showToast("Erro ao abrir pasta: " + e, "error");
    }
  }

  function getSessionTreeExpanded(sessionId: string, rootPath: string): Set<string> {
    if (!sessionExpandedTrees[sessionId]) {
      sessionExpandedTrees[sessionId] = new Set([
        `hist-before-root-${rootPath}`,
        `hist-after-root-${rootPath}`,
      ]);
    }
    return sessionExpandedTrees[sessionId];
  }

  function toggleSessionTreeFolder(sessionId: string, nodeId: string) {
    const set = sessionExpandedTrees[sessionId] || new Set();
    if (set.has(nodeId)) {
      set.delete(nodeId);
    } else {
      set.add(nodeId);
    }
    sessionExpandedTrees[sessionId] = new Set(set);
    sessionExpandedTrees = { ...sessionExpandedTrees };
  }

  function expandAllSessionTree(sessionId: string, nodes: TreeNodeData[]) {
    const set = sessionExpandedTrees[sessionId] || new Set();
    function collect(list: TreeNodeData[]) {
      for (const n of list) {
        if (n.isFolder) {
          set.add(n.id);
          if (n.children) collect(n.children);
        }
      }
    }
    collect(nodes);
    sessionExpandedTrees[sessionId] = new Set(set);
    sessionExpandedTrees = { ...sessionExpandedTrees };
  }

  function collapseAllSessionTree(sessionId: string, rootId: string) {
    sessionExpandedTrees[sessionId] = new Set([rootId]);
    sessionExpandedTrees = { ...sessionExpandedTrees };
  }

  // Constrói Árvore "Antes" (Estrutura Original)
  function buildSessionBeforeTree(files: SessionFileInfo[], rootPath: string): TreeNodeData[] {
    const normalizedRoot = rootPath.replace(/\\/g, "/").replace(/\/+$/, "");
    const rootName = normalizedRoot ? normalizedRoot.split("/").pop() || "Pasta Raiz" : "Pasta Raiz";

    const rootNode: TreeNodeData = {
      id: `hist-before-root-${rootPath}`,
      name: rootName,
      isFolder: true,
      fullPath: rootPath,
      children: [],
      fileCount: 0,
    };

    const folderMap = new Map<string, TreeNodeData>();
    folderMap.set("", rootNode);

    for (const file of files) {
      const normalizedFilePath = file.original_path.replace(/\\/g, "/");
      let relPath = "";
      if (normalizedFilePath.startsWith(normalizedRoot)) {
        relPath = normalizedFilePath.substring(normalizedRoot.length).replace(/^\/+/, "");
      } else {
        relPath = file.filename;
      }

      const parts = relPath.split("/");
      parts.pop();

      let currentRel = "";
      let parentNode = rootNode;

      for (const segment of parts) {
        currentRel = currentRel ? `${currentRel}/${segment}` : segment;
        if (!folderMap.has(currentRel)) {
          const newFolder: TreeNodeData = {
            id: `hist-before-dir-${rootPath}-${currentRel}`,
            name: segment,
            isFolder: true,
            fullPath: `${normalizedRoot}/${currentRel}`,
            isPreservedFolder: file.is_already_organized,
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
        id: `hist-before-file-${file.file_id}`,
        name: file.filename,
        isFolder: false,
        fullPath: file.original_path,
        fileCount: 1,
        categoryName: file.category_name,
        categoryColor: file.category_color || undefined,
        isPreservedFile: file.is_already_organized,
      };

      parentNode.children = parentNode.children || [];
      parentNode.children.push(fileNode);
    }

    function calculateCounts(node: TreeNodeData): number {
      if (!node.isFolder) return 1;
      let sum = 0;
      if (node.children) {
        for (const c of node.children) {
          sum += calculateCounts(c);
        }
      }
      node.fileCount = sum;
      return sum;
    }
    calculateCounts(rootNode);

    return [rootNode];
  }

  // Constrói Árvore "Depois" (Estrutura Proposta com Categorias/Subpastas)
  function buildSessionAfterTree(files: SessionFileInfo[], rootPath: string): TreeNodeData[] {
    const normalizedRoot = rootPath.replace(/\\/g, "/").replace(/\/+$/, "");
    const rootName = normalizedRoot ? normalizedRoot.split("/").pop() || "Pasta Raiz" : "Pasta Raiz";

    const rootNode: TreeNodeData = {
      id: `hist-after-root-${rootPath}`,
      name: rootName,
      isFolder: true,
      fullPath: rootPath,
      children: [],
      fileCount: 0,
    };

    const folderMap = new Map<string, TreeNodeData>();
    folderMap.set("", rootNode);

    for (const file of files) {
      const catPath = file.category_name || "Outros Arquivos";
      const parts = catPath.replace(/\\/g, "/").split("/").map((s) => s.trim()).filter(Boolean);

      let currentRel = "";
      let parentNode = rootNode;

      for (const segment of parts) {
        currentRel = currentRel ? `${currentRel}/${segment}` : segment;
        if (!folderMap.has(currentRel)) {
          const newFolder: TreeNodeData = {
            id: `hist-after-dir-${rootPath}-${currentRel}`,
            name: segment,
            isFolder: true,
            fullPath: `${normalizedRoot}/${currentRel}`,
            isPreservedFolder: file.is_already_organized,
            categoryColor: file.category_color || undefined,
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
        id: `hist-after-file-${file.file_id}`,
        name: file.proposed_name || file.filename,
        isFolder: false,
        fullPath: file.original_path,
        fileCount: 1,
        categoryName: file.category_name,
        categoryColor: file.category_color || undefined,
        isPreservedFile: file.is_already_organized,
      };

      parentNode.children = parentNode.children || [];
      parentNode.children.push(fileNode);
    }

    function calculateCounts(node: TreeNodeData): number {
      if (!node.isFolder) return 1;
      let sum = 0;
      if (node.children) {
        for (const c of node.children) {
          sum += calculateCounts(c);
        }
      }
      node.fileCount = sum;
      return sum;
    }
    calculateCounts(rootNode);

    return [rootNode];
  }

  function getFilteredRenames(renames: SessionRenameInfo[], filter: string = "all"): SessionRenameInfo[] {
    if (filter === "applied") return renames.filter((r) => r.applied);
    if (filter === "preview") return renames.filter((r) => !r.applied);
    return renames;
  }

  $: anySessionOpen = Object.values(sessionExpanded).some(Boolean);

  $: filteredSessions = sessions.filter((s) => {
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase();
    return (
      s.root_path.toLowerCase().includes(q) ||
      s.categories_assigned.some((c) => c.name.toLowerCase().includes(q)) ||
      s.files.some((f) => f.filename.toLowerCase().includes(q) || f.category_name.toLowerCase().includes(q)) ||
      s.moves.some((m) => m.from_path.toLowerCase().includes(q) || m.to_path.toLowerCase().includes(q))
    );
  });
</script>

<div class="history-view">
  <!-- Header -->
  <div class="history-header">
    <div class="header-titles">
      <div class="title-with-badge">
        <span class="history-main-icon">📜</span>
        <h1>Histórico de Organizações</h1>
        <span class="history-count-badge">{sessions.length}</span>
      </div>
      <p class="subtitle">
        Auditoria completa de sessões: visualize árvores navegáveis lado a lado, categorias, tags, arquivos movidos e histórico de renomeações.
      </p>
    </div>

    <div class="header-controls">
      <!-- Botão Global de Expandir/Recolher Sessões -->
      <button
        class="toggle-all-sessions-btn"
        title={anySessionOpen ? "Recolher todas as sessões" : "Expandir todas as sessões"}
        on:click={() => toggleAllSessions(!anySessionOpen)}
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          {#if anySessionOpen}
            <polyline points="18 15 12 9 6 15"></polyline>
          {:else}
            <polyline points="6 9 12 15 18 9"></polyline>
          {/if}
        </svg>
        <span>{anySessionOpen ? "Recolher Sessões" : "Expandir Sessões"}</span>
      </button>

      <div class="search-input-wrapper">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        <input
          type="text"
          placeholder="Buscar por pasta, tag, categoria ou arquivo..."
          bind:value={searchQuery}
          class="search-input"
        />
        {#if searchQuery}
          <button class="clear-search-btn" on:click={() => (searchQuery = "")}>✕</button>
        {/if}
      </div>

      <button class="refresh-btn" title="Atualizar Histórico" on:click={fetchHistory}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="23 4 23 10 17 10"></polyline>
          <polyline points="1 20 1 14 7 14"></polyline>
          <path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"></path>
        </svg>
      </button>
    </div>
  </div>

  <!-- Content -->
  <div class="history-content">
    {#if isLoading}
      <div class="loading-state">
        <div class="spinner"></div>
        <span>Carregando histórico de organizações...</span>
      </div>
    {:else if filteredSessions.length === 0}
      <div class="empty-state">
        <div class="empty-icon-wrap">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <circle cx="12" cy="12" r="10"></circle>
            <polyline points="12 6 12 12 16 14"></polyline>
          </svg>
        </div>
        {#if searchQuery}
          <h2>Nenhuma organização encontrada</h2>
          <p>Nenhuma sessão corresponde ao filtro de busca "{searchQuery}".</p>
          <button class="primary-btn" on:click={() => (searchQuery = "")}>Limpar Busca</button>
        {:else}
          <h2>Nenhuma organização registrada</h2>
          <p>
            Quando você organizar pastas e arquivos no Indexo, o registro completo com tags e histórico de movimentações ficará disponível aqui.
          </p>
          <button class="primary-btn" on:click={() => currentView.set("folder-select")}>
            Selecionar Pasta para Organizar
          </button>
        {/if}
      </div>
    {:else}
      <div class="sessions-list">
        {#each filteredSessions as session (session.session_id)}
          {@const isExpanded = sessionExpanded[session.session_id] ?? false}
          {@const currentTab = sessionTabs[session.session_id] || (session.files_moved_count > 0 ? "moves" : "tree")}
          {@const isAllUndone = session.files_moved_count > 0 && session.undone_count === session.files_moved_count}
          {@const isPartialUndone = session.undone_count > 0 && session.undone_count < session.files_moved_count}

          <div class="session-card" class:is-undone={isAllUndone}>
            <!-- Top Card Bar -->
            <div
              class="session-card-header"
              role="button"
              tabindex="0"
              on:click={() => toggleExpand(session.session_id)}
              on:keydown={(e) => (e.key === "Enter" || e.key === " ") && toggleExpand(session.session_id)}
            >
              <div class="session-main-info">
                <div class="folder-title-row">
                  <span class="folder-icon">📁</span>
                  <span class="folder-path" title={session.root_path}>{session.root_path}</span>
                  <button
                    class="icon-action-btn"
                    title="Abrir no Explorador de Arquivos"
                    on:click|stopPropagation={() => handleOpenExplorer(session.root_path)}
                  >
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                      <polyline points="15 3 21 3 21 9"></polyline>
                      <line x1="10" y1="14" x2="21" y2="3"></line>
                    </svg>
                  </button>
                </div>

                <div class="session-meta-row">
                  <span class="meta-time">🕒 {formatDateTime(session.started_at)}</span>
                  <span class="meta-separator">•</span>
                  <span class="meta-files">
                    📄 {session.files.length} {session.files.length === 1 ? "arquivo escaneado" : "arquivos escaneados"}
                  </span>
                  {#if session.files_moved_count > 0}
                    <span class="meta-separator">•</span>
                    <span class="meta-files moved-count">
                      📦 {session.files_moved_count} {session.files_moved_count === 1 ? "movido" : "movidos"}
                    </span>
                  {/if}
                </div>
              </div>

              <!-- Status & Actions -->
              <div
                class="session-header-actions"
                role="presentation"
                on:click|stopPropagation
                on:keydown|stopPropagation
              >
                <div class="status-badge-container">
                  {#if isAllUndone}
                    <span class="status-badge undone">
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="1 4 1 10 7 10"></polyline>
                        <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"></path>
                      </svg>
                      Desfeito
                    </span>
                  {:else if isPartialUndone}
                    <span class="status-badge partial">
                      Parcial ({session.undone_count}/{session.files_moved_count} desfeitos)
                    </span>
                  {:else if session.files_moved_count > 0}
                    <span class="status-badge done">
                      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="20 6 9 17 4 12"></polyline>
                      </svg>
                      Concluído
                    </span>
                  {:else}
                    <span class="status-badge info">Análise / Sugestão</span>
                  {/if}
                </div>

                {#if session.files_moved_count > 0 && !isAllUndone}
                  <button
                    class="undo-action-btn"
                    title="Desfazer e mover arquivos de volta para a pasta de origem"
                    disabled={undoingSessionId === session.session_id}
                    on:click={() => promptUndo(session)}
                  >
                    {#if undoingSessionId === session.session_id}
                      <span class="mini-spin"></span>
                      Desfazendo...
                    {:else}
                      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="1 4 1 10 7 10"></polyline>
                        <path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"></path>
                      </svg>
                      Desfazer Organização
                    {/if}
                  </button>
                {/if}

                <button
                  class="expand-btn"
                  class:active={isExpanded}
                  on:click={() => toggleExpand(session.session_id)}
                  title={isExpanded ? "Recolher detalhes" : "Expandir detalhes da sessão"}
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points={isExpanded ? "18 15 12 9 6 15" : "6 9 12 15 18 9"}></polyline>
                  </svg>
                </button>
              </div>
            </div>

            <!-- Session Navigation Tabs Bar (3 botões obrigatórios + 4º se movido + 5º renomeações) -->
            <div class="session-tabs-bar">
              <button
                class="session-tab-btn"
                class:active={currentTab === "tree" && isExpanded}
                on:click={() => setTab(session.session_id, "tree")}
              >
                <span>🌳</span>
                <span>Árvore Navegável</span>
                <span class="tab-badge">{session.files.length}</span>
              </button>

              <button
                class="session-tab-btn"
                class:active={currentTab === "categories" && isExpanded}
                on:click={() => setTab(session.session_id, "categories")}
              >
                <span>📁</span>
                <span>Categorias</span>
                <span class="tab-badge">{session.categories_assigned.length}</span>
              </button>

              <button
                class="session-tab-btn"
                class:active={currentTab === "tags" && isExpanded}
                on:click={() => setTab(session.session_id, "tags")}
              >
                <span>🏷️</span>
                <span>Tags</span>
                <span class="tab-badge">{session.categories_assigned.length}</span>
              </button>

              {#if session.files_moved_count > 0}
                <button
                  class="session-tab-btn highlight-moves"
                  class:active={currentTab === "moves" && isExpanded}
                  on:click={() => setTab(session.session_id, "moves")}
                >
                  <span>📦</span>
                  <span>Arquivos Movidos</span>
                  <span class="tab-badge">{session.moves.length}</span>
                </button>
              {/if}

              <button
                class="session-tab-btn"
                class:active={currentTab === "renames" && isExpanded}
                on:click={() => setTab(session.session_id, "renames")}
              >
                <span>✏️</span>
                <span>Nomes Alterados</span>
                <span class="tab-badge">{session.renames.length}</span>
              </button>
            </div>

            <!-- Expandable Body Content according to selected tab -->
            {#if isExpanded}
              <div class="session-tab-body">
                <!-- 1. TAB: ÁRVORE PROPOSTA NAVEGÁVEL LADO A LADO -->
                {#if currentTab === "tree"}
                  {@const beforeTree = buildSessionBeforeTree(session.files, session.root_path)}
                  {@const afterTree = buildSessionAfterTree(session.files, session.root_path)}
                  {@const treeExpandedIds = getSessionTreeExpanded(session.session_id, session.root_path)}

                  <div class="dual-tree-tab-content">
                    <div class="dual-tree-header-row">
                      <span class="tab-info-text">
                        Navegue lado a lado pela estrutura de pastas original e pela estrutura sugerida pelo Indexo:
                      </span>

                      <div class="dual-tree-controls">
                        <button
                          class="tree-action-btn"
                          title="Expandir todas as pastas de ambas as colunas"
                          on:click={() => {
                            expandAllSessionTree(session.session_id, [...beforeTree, ...afterTree]);
                          }}
                        >
                          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                            <polyline points="7 13 12 18 17 13"></polyline>
                            <polyline points="7 6 12 11 17 6"></polyline>
                          </svg>
                          Expandir Pastas
                        </button>
                        <button
                          class="tree-action-btn"
                          title="Recolher todas as pastas"
                          on:click={() => {
                            sessionExpandedTrees[session.session_id] = new Set([
                              `hist-before-root-${session.root_path}`,
                              `hist-after-root-${session.root_path}`,
                            ]);
                            sessionExpandedTrees = { ...sessionExpandedTrees };
                          }}
                        >
                          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                            <polyline points="17 11 12 6 7 11"></polyline>
                            <polyline points="17 18 12 13 7 18"></polyline>
                          </svg>
                          Recolher Pastas
                        </button>
                      </div>
                    </div>

                    <!-- Colunas Lado a Lado -->
                    <div class="dual-tree-container">
                      <!-- Coluna Esquerda: Estrutura Atual / Original -->
                      <div class="tree-column-panel">
                        <div class="tree-column-header">
                          <div class="tree-col-title">
                            <span class="tree-col-icon">📂</span>
                            <span>Estrutura Original</span>
                          </div>
                          <span class="tree-col-meta">{session.files.length} arquivos</span>
                        </div>

                        <div class="tree-column-scroll">
                          {#each beforeTree as node (node.id)}
                            <FileTreeNode
                              {node}
                              expandedIds={treeExpandedIds}
                              onToggleFolder={(id) => toggleSessionTreeFolder(session.session_id, id)}
                              onFileContextMenu={() => {}}
                              onFolderContextMenu={() => {}}
                            />
                          {/each}
                        </div>
                      </div>

                      <!-- Coluna Direita: Estrutura Proposta Organizada -->
                      <div class="tree-column-panel proposed">
                        <div class="tree-column-header">
                          <div class="tree-col-title">
                            <span class="tree-col-icon">✨</span>
                            <span>Estrutura Proposta / Organizada</span>
                          </div>
                          <span class="tree-col-meta">{session.files.length} arquivos</span>
                        </div>

                        <div class="tree-column-scroll">
                          {#each afterTree as node (node.id)}
                            <FileTreeNode
                              {node}
                              expandedIds={treeExpandedIds}
                              onToggleFolder={(id) => toggleSessionTreeFolder(session.session_id, id)}
                              onFileContextMenu={() => {}}
                              onFolderContextMenu={() => {}}
                            />
                          {/each}
                        </div>
                      </div>
                    </div>
                  </div>

                <!-- 2. TAB: CATEGORIAS -->
                {:else if currentTab === "categories"}
                  <div class="categories-tab-content">
                    <div class="tab-info-header">
                      <span>Categorias identificadas ou criadas automaticamente para esta sessão:</span>
                    </div>

                    {#if session.categories_assigned.length === 0}
                      <div class="empty-tab-state">
                        Nenhuma categoria extra criada (pastas e arquivos preservados em suas posições originais).
                      </div>
                    {:else}
                      <div class="categories-grid">
                        {#each session.categories_assigned as cat}
                          <div class="category-card-mini">
                            <div class="cat-card-top">
                              <span class="cat-dot" style="background: {cat.color || 'var(--accent-primary)'};"></span>
                              <span class="cat-name">{cat.name}</span>
                              <span class="cat-type-pill {cat.created_by}">
                                {cat.created_by === "user" ? "Manual" : "Auto IA"}
                              </span>
                            </div>
                            <div class="cat-card-bottom">
                              <span class="cat-file-count">📊 {cat.file_count} {cat.file_count === 1 ? "arquivo" : "arquivos"}</span>
                            </div>
                          </div>
                        {/each}
                      </div>
                    {/if}
                  </div>

                <!-- 3. TAB: TAGS -->
                {:else if currentTab === "tags"}
                  <div class="tags-tab-content">
                    <div class="tab-info-header">
                      <span>Tags semânticas associadas aos arquivos desta pasta:</span>
                    </div>

                    {#if session.categories_assigned.length === 0}
                      <div class="empty-tab-state">
                        Nenhuma tag extra gerada para arquivos preservados.
                      </div>
                    {:else}
                      <div class="tags-cloud-wrap">
                        {#each session.categories_assigned as tag}
                          <div class="tag-badge-pill" style="border-color: {tag.color || 'var(--border-medium)'};">
                            <span class="tag-bullet" style="background: {tag.color || 'var(--accent-primary)'};"></span>
                            <span class="tag-title">{tag.name}</span>
                            <span class="tag-count-bubble">{tag.file_count}</span>
                          </div>
                        {/each}
                      </div>
                    {/if}
                  </div>

                <!-- 4. TAB: ARQUIVOS MOVIDOS -->
                {:else if currentTab === "moves"}
                  <div class="moves-tab-content">
                    <div class="tab-info-header">
                      <span>Registro físico de arquivos movidos no disco:</span>
                    </div>

                    <div class="moves-list">
                      {#each session.moves as move}
                        <div class="move-item-row" class:is-move-undone={move.undone === 1}>
                          <div class="move-icon">
                            {#if move.undone === 1}
                              <span title="Revertido / Desfeito">↩️</span>
                            {:else}
                              <span title="Movido">📄</span>
                            {/if}
                          </div>
                          <div class="move-paths">
                            <div class="path-from" title={move.from_path}>
                              <span class="path-tag">Origem:</span>
                              <span class="path-text">{move.from_path}</span>
                            </div>
                            <div class="path-to" title={move.to_path}>
                              <span class="path-tag">Destino:</span>
                              <span class="path-text">{move.to_path}</span>
                            </div>
                          </div>
                          <div class="move-status">
                            {#if move.undone === 1}
                              <span class="undone-pill">Revertido</span>
                            {:else}
                              <span class="active-pill">Ativo</span>
                            {/if}
                          </div>
                        </div>
                      {/each}
                    </div>
                  </div>

                <!-- 5. TAB: NOMES ALTERADOS (PREVIEW E REALMENTE ALTERADOS) -->
                {:else if currentTab === "renames"}
                  <div class="renames-tab-content">
                    <div class="renames-tab-header">
                      <span class="tab-info-text">
                        Auditoria de renomeação: veja propostas semânticas geradas (Preview) e nomes aplicados no disco.
                      </span>

                      <!-- Sub-filter -->
                      <div class="rename-subfilters">
                        <button
                          class="subfilter-btn"
                          class:active={renameFilters[session.session_id] === "all"}
                          on:click={() => { renameFilters[session.session_id] = "all"; renameFilters = { ...renameFilters }; }}
                        >
                          Todos ({session.renames.length})
                        </button>
                        <button
                          class="subfilter-btn"
                          class:active={renameFilters[session.session_id] === "applied"}
                          on:click={() => { renameFilters[session.session_id] = "applied"; renameFilters = { ...renameFilters }; }}
                        >
                          Realmente Alterados ({session.renames.filter((r) => r.applied).length})
                        </button>
                        <button
                          class="subfilter-btn"
                          class:active={renameFilters[session.session_id] === "preview"}
                          on:click={() => { renameFilters[session.session_id] = "preview"; renameFilters = { ...renameFilters }; }}
                        >
                          Apenas Preview ({session.renames.filter((r) => !r.applied).length})
                        </button>
                      </div>
                    </div>

                    {#if getFilteredRenames(session.renames, renameFilters[session.session_id]).length === 0}
                      <div class="no-renames-msg">
                        Nenhum arquivo correspondente ao filtro selecionado.
                      </div>
                    {:else}
                      <div class="renames-table-container">
                        <table class="renames-table">
                          <thead>
                            <tr>
                              <th>Nome Original</th>
                              <th>Preview Sugerido</th>
                              <th>Nome Final Aplicado</th>
                              <th>Status</th>
                            </tr>
                          </thead>
                          <tbody>
                            {#each getFilteredRenames(session.renames, renameFilters[session.session_id]) as r}
                              <tr>
                                <td class="col-original">
                                  <span class="filename-mono">{r.original_name}</span>
                                </td>
                                <td class="col-preview">
                                  <span class="preview-mono">{r.proposed_name}</span>
                                </td>
                                <td class="col-final">
                                  {#if r.applied && r.final_name}
                                    <span class="applied-mono">{r.final_name}</span>
                                  {:else}
                                    <span class="not-applied-label">— (Não aplicado)</span>
                                  {/if}
                                </td>
                                <td class="col-status">
                                  {#if r.undone}
                                    <span class="rename-badge undone">Desfeito</span>
                                  {:else if r.applied}
                                    <span class="rename-badge applied">Alterado no Disco</span>
                                  {:else}
                                    <span class="rename-badge preview">Sugestão / Preview</span>
                                  {/if}
                                </td>
                              </tr>
                            {/each}
                          </tbody>
                        </table>
                      </div>
                    {/if}
                  </div>
                {/if}
              </div>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- Modal de Confirmação de Undo -->
{#if confirmingUndoSession}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (confirmingUndoSession = null)}
    on:keydown={(e) => e.key === "Escape" && (confirmingUndoSession = null)}
  >
    <div class="modal-card">
      <div class="modal-card-icon">↩️</div>
      <h2>Desfazer Esta Organização?</h2>
      <p class="modal-subtitle">
        Todos os <strong>{confirmingUndoSession.files_moved_count} arquivos</strong> movidos na sessão de <strong>{confirmingUndoSession.root_path}</strong> serão retornados com segurança para suas posições originais.
      </p>
      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (confirmingUndoSession = null)}>Cancelar</button>
        <button class="primary-btn undo-confirm-btn" on:click={handleConfirmUndo}>
          Sim, Desfazer e Mover de Volta
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .history-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 1.5rem 2rem;
    gap: 1.25rem;
    overflow: hidden;
    min-height: 0;
    animation: fadeIn 200ms ease-out;
  }

  .history-header {
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

  .title-with-badge {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .history-main-icon {
    font-size: 1.4rem;
  }

  .title-with-badge h1 {
    font-size: 1.45rem;
    font-weight: 800;
    color: var(--text-primary);
    margin: 0;
    letter-spacing: -0.02em;
  }

  .history-count-badge {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-medium);
    color: var(--text-muted);
    font-size: 0.78rem;
    font-weight: 700;
    padding: 0.15rem 0.55rem;
    border-radius: var(--radius-full);
  }

  .subtitle {
    font-size: 0.86rem;
    color: var(--text-muted);
    margin: 0;
  }

  .header-controls {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .toggle-all-sessions-btn {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    padding: 0.5rem 0.85rem;
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-muted);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    transition: all 120ms ease;
  }

  .toggle-all-sessions-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
    border-color: var(--border-strong, #475569);
  }

  .search-input-wrapper {
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-input-wrapper svg {
    position: absolute;
    left: 0.75rem;
    color: var(--text-muted);
    pointer-events: none;
  }

  .search-input {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    padding: 0.5rem 2rem 0.5rem 2.25rem;
    font-size: 0.85rem;
    color: var(--text-primary);
    width: 280px;
    transition: all 150ms ease;
  }

  .search-input:focus {
    outline: none;
    border-color: var(--accent-primary);
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.15);
  }

  .clear-search-btn {
    position: absolute;
    right: 0.6rem;
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 0.8rem;
    padding: 0.2rem;
  }

  .refresh-btn {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    padding: 0.5rem;
    color: var(--text-muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 120ms ease;
  }

  .refresh-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .history-content {
    flex: 1;
    overflow-y: auto;
    padding-right: 0.4rem;
    scrollbar-width: thin;
    scrollbar-color: var(--border-medium) transparent;
  }

  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    padding: 5rem 1rem;
    color: var(--text-muted);
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid rgba(59, 130, 246, 0.2);
    border-top-color: var(--accent-primary);
    border-radius: 50%;
    animation: spin 800ms linear infinite;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 4rem 2rem;
    background: var(--bg-secondary);
    border: 1px dashed var(--border-medium);
    border-radius: var(--radius-xl);
    gap: 1rem;
  }

  .empty-icon-wrap {
    color: var(--text-muted);
    background: var(--bg-tertiary);
    padding: 1.25rem;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .empty-state h2 {
    font-size: 1.15rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .empty-state p {
    font-size: 0.88rem;
    color: var(--text-muted);
    max-width: 440px;
    line-height: 1.5;
    margin: 0;
  }

  .sessions-list {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .session-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    overflow: hidden;
    transition: all 150ms ease;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.04);
  }

  .session-card:hover {
    border-color: var(--border-strong, #475569);
  }

  .session-card.is-undone {
    opacity: 0.8;
  }

  .session-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    gap: 1rem;
    flex-wrap: wrap;
    background: var(--bg-secondary);
    cursor: pointer;
    user-select: none;
  }

  .session-main-info {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    min-width: 0;
    flex: 1;
  }

  .folder-title-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }

  .folder-icon {
    font-size: 1.1rem;
  }

  .folder-path {
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 550px;
    font-family: var(--font-mono);
  }

  .icon-action-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 0.2rem 0.35rem;
    border-radius: var(--radius-sm);
    transition: all 120ms ease;
  }

  .icon-action-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .session-meta-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .moved-count {
    color: var(--accent-primary);
    font-weight: 600;
  }

  .session-header-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.76rem;
    font-weight: 700;
    padding: 0.25rem 0.65rem;
    border-radius: var(--radius-full);
  }

  .status-badge.done {
    background: rgba(16, 185, 129, 0.12);
    color: #10b981;
    border: 1px solid rgba(16, 185, 129, 0.25);
  }

  .status-badge.undone {
    background: rgba(245, 158, 11, 0.12);
    color: #f59e0b;
    border: 1px solid rgba(245, 158, 11, 0.25);
  }

  .status-badge.partial {
    background: rgba(59, 130, 246, 0.12);
    color: #3b82f6;
    border: 1px solid rgba(59, 130, 246, 0.25);
  }

  .status-badge.info {
    background: var(--bg-tertiary);
    color: var(--text-muted);
    border: 1px solid var(--border-subtle);
  }

  .undo-action-btn {
    background: rgba(239, 68, 68, 0.1);
    color: #ef4444;
    border: 1px solid rgba(239, 68, 68, 0.25);
    padding: 0.35rem 0.75rem;
    border-radius: var(--radius-md);
    font-size: 0.78rem;
    font-weight: 600;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    transition: all 120ms ease;
  }

  .undo-action-btn:hover:not(:disabled) {
    background: #ef4444;
    color: white;
  }

  .undo-action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .expand-btn {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    padding: 0.35rem 0.55rem;
    color: var(--text-muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 120ms ease;
  }

  .expand-btn:hover, .expand-btn.active {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  /* Segmented Session Tabs Bar */
  .session-tabs-bar {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.35rem 1rem;
    background: var(--bg-primary);
    border-top: 1px solid var(--border-subtle);
    border-bottom: 1px solid var(--border-subtle);
    overflow-x: auto;
  }

  .session-tab-btn {
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    padding: 0.35rem 0.65rem;
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text-muted);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    transition: all 120ms ease;
    white-space: nowrap;
  }

  .session-tab-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .session-tab-btn.active {
    background: var(--bg-secondary);
    border-color: var(--border-medium);
    color: var(--text-primary);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
  }

  .session-tab-btn.highlight-moves.active {
    border-color: rgba(59, 130, 246, 0.4);
    color: var(--accent-primary);
  }

  .tab-badge {
    background: var(--bg-tertiary);
    border-radius: var(--radius-full);
    padding: 0.05rem 0.4rem;
    font-size: 0.72rem;
    font-weight: 700;
  }

  /* Tab Body Container */
  .session-tab-body {
    padding: 1rem 1.25rem;
    background: var(--bg-primary);
    animation: fadeIn 150ms ease-out;
  }

  .tab-info-header {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text-muted);
    margin-bottom: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  /* 1. Dual Side-by-Side Tree View Styles */
  .dual-tree-tab-content {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .dual-tree-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .dual-tree-controls {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .tree-action-btn {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-sm);
    padding: 0.25rem 0.55rem;
    font-size: 0.76rem;
    color: var(--text-muted);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-weight: 600;
    transition: all 120ms ease;
  }

  .tree-action-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
    border-color: var(--border-strong, #475569);
  }

  .dual-tree-container {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1rem;
    min-height: 280px;
    max-height: 480px;
  }

  @media (max-width: 820px) {
    .dual-tree-container {
      grid-template-columns: 1fr;
    }
  }

  .tree-column-panel {
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .tree-column-panel.proposed {
    border-color: rgba(59, 130, 246, 0.25);
  }

  .tree-column-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.55rem 0.85rem;
    background: rgba(0, 0, 0, 0.08);
    border-bottom: 1px solid var(--border-subtle);
    font-size: 0.82rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .tree-col-title {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .tree-col-icon {
    font-size: 0.95rem;
  }

  .tree-col-meta {
    font-size: 0.74rem;
    color: var(--text-muted);
    font-weight: normal;
  }

  .tree-column-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
    scrollbar-width: thin;
  }

  .empty-tab-state {
    text-align: center;
    color: var(--text-muted);
    font-size: 0.85rem;
    padding: 2rem;
  }

  /* 2. Categories Grid Styles */
  .categories-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 0.75rem;
  }

  .category-card-mini {
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 0.75rem 0.85rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .cat-card-top {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .cat-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .cat-name {
    font-size: 0.88rem;
    font-weight: 700;
    color: var(--text-primary);
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cat-type-pill {
    font-size: 0.68rem;
    font-weight: 600;
    padding: 0.1rem 0.4rem;
    border-radius: var(--radius-full);
    background: var(--bg-tertiary);
    color: var(--text-muted);
  }

  .cat-file-count {
    font-size: 0.76rem;
    color: var(--text-muted);
  }

  /* 3. Tags Cloud Styles */
  .tags-cloud-wrap {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .tag-badge-pill {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    padding: 0.35rem 0.75rem;
    border-radius: var(--radius-full);
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .tag-bullet {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .tag-count-bubble {
    background: var(--bg-tertiary);
    font-size: 0.72rem;
    padding: 0.05rem 0.4rem;
    border-radius: var(--radius-full);
    color: var(--text-muted);
    font-weight: 700;
  }

  /* 4. Moves List Styles */
  .moves-list {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    max-height: 320px;
    overflow-y: auto;
    padding-right: 0.25rem;
    scrollbar-width: thin;
  }

  .move-item-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 0.45rem 0.75rem;
    font-size: 0.8rem;
    font-family: var(--font-mono);
  }

  .move-item-row.is-move-undone {
    opacity: 0.6;
    text-decoration: line-through;
  }

  .move-icon {
    font-size: 0.9rem;
    flex-shrink: 0;
  }

  .move-paths {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
  }

  .path-from, .path-to {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    overflow: hidden;
  }

  .path-tag {
    font-size: 0.68rem;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    flex-shrink: 0;
  }

  .path-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-primary);
  }

  .path-from .path-text {
    color: var(--text-muted);
  }

  .undone-pill {
    background: rgba(245, 158, 11, 0.15);
    color: #f59e0b;
    font-size: 0.7rem;
    font-weight: 700;
    padding: 0.15rem 0.45rem;
    border-radius: var(--radius-sm);
  }

  .active-pill {
    background: rgba(16, 185, 129, 0.15);
    color: #10b981;
    font-size: 0.7rem;
    font-weight: 700;
    padding: 0.15rem 0.45rem;
    border-radius: var(--radius-sm);
  }

  /* 5. Renames Table Styles */
  .renames-tab-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    margin-bottom: 0.75rem;
    flex-wrap: wrap;
  }

  .tab-info-text {
    font-size: 0.8rem;
    color: var(--text-muted);
  }

  .rename-subfilters {
    display: flex;
    align-items: center;
    gap: 0.35rem;
  }

  .subfilter-btn {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-sm);
    padding: 0.25rem 0.55rem;
    font-size: 0.76rem;
    color: var(--text-muted);
    cursor: pointer;
    font-weight: 600;
    transition: all 120ms ease;
  }

  .subfilter-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .subfilter-btn.active {
    background: var(--accent-primary);
    border-color: var(--accent-primary);
    color: white;
  }

  .renames-table-container {
    max-height: 320px;
    overflow-y: auto;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    background: var(--bg-secondary);
  }

  .renames-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.8rem;
    text-align: left;
  }

  .renames-table th {
    background: rgba(0, 0, 0, 0.12);
    padding: 0.55rem 0.75rem;
    color: var(--text-muted);
    font-size: 0.74rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    border-bottom: 1px solid var(--border-subtle);
  }

  .renames-table td {
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border-subtle);
    vertical-align: middle;
  }

  .renames-table tr:last-child td {
    border-bottom: none;
  }

  .filename-mono {
    font-family: var(--font-mono);
    color: var(--text-primary);
  }

  .preview-mono {
    font-family: var(--font-mono);
    color: #8b5cf6;
    font-weight: 600;
  }

  .applied-mono {
    font-family: var(--font-mono);
    color: #10b981;
    font-weight: 700;
  }

  .not-applied-label {
    color: var(--text-muted);
    font-style: italic;
  }

  .rename-badge {
    display: inline-flex;
    align-items: center;
    font-size: 0.72rem;
    font-weight: 700;
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-full);
  }

  .rename-badge.applied {
    background: rgba(16, 185, 129, 0.12);
    color: #10b981;
  }

  .rename-badge.preview {
    background: rgba(139, 92, 246, 0.12);
    color: #8b5cf6;
  }

  .rename-badge.undone {
    background: rgba(245, 158, 11, 0.12);
    color: #f59e0b;
  }

  .no-renames-msg {
    text-align: center;
    color: var(--text-muted);
    font-size: 0.85rem;
    padding: 2rem;
  }

  /* Modal */
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
    padding: 1.5rem;
    max-width: 440px;
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 0.75rem;
    box-shadow: var(--shadow-xl);
  }

  .modal-card-icon {
    font-size: 2rem;
  }

  .modal-card h2 {
    margin: 0;
    font-size: 1.15rem;
    color: var(--text-primary);
  }

  .modal-subtitle {
    margin: 0;
    font-size: 0.85rem;
    color: var(--text-muted);
    line-height: 1.45;
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
  }

  .undo-confirm-btn {
    background: #ef4444;
  }

  .mini-spin {
    display: inline-block;
    width: 12px;
    height: 12px;
    border: 2px solid rgba(239, 68, 68, 0.3);
    border-top-color: #ef4444;
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
