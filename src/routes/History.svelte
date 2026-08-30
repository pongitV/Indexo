<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import {
    getOrganizationHistory,
    undoSession,
    openInExplorer,
    type OrganizationSessionSummary,
  } from "../lib/api";
  import { currentView, showToast } from "../lib/stores";

  let sessions: OrganizationSessionSummary[] = [];
  let isLoading: boolean = true;
  let searchQuery: string = "";
  let expandedSessions: Set<string> = new Set();
  let undoingSessionId: string | null = null;
  let confirmingUndoSession: OrganizationSessionSummary | null = null;

  onMount(async () => {
    await fetchHistory();
  });

  async function fetchHistory() {
    isLoading = true;
    try {
      sessions = await getOrganizationHistory();
    } catch (err: any) {
      showToast("Erro ao carregar histórico: " + err, "error");
    } finally {
      isLoading = false;
    }
  }

  function toggleExpand(sessionId: string) {
    if (expandedSessions.has(sessionId)) {
      expandedSessions.delete(sessionId);
    } else {
      expandedSessions.add(sessionId);
    }
    expandedSessions = new Set(expandedSessions);
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

  $: filteredSessions = sessions.filter((s) => {
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase();
    return (
      s.root_path.toLowerCase().includes(q) ||
      s.categories_assigned.some((c) => c.toLowerCase().includes(q)) ||
      s.moves.some((m) => m.from_path.toLowerCase().includes(q) || m.to_path.toLowerCase().includes(q))
    );
  });
</script>

<div class="history-view">
  <!-- Header -->
  <div class="history-header">
    <div class="header-titles">
      <div class="title-with-badge">
        <h1>Histórico de Organizações</h1>
        <span class="history-count-badge">{sessions.length}</span>
      </div>
      <p class="subtitle">
        Auditoria de todas as organizações realizadas, com tags, categorias criadas e opção de desfazer (Undo).
      </p>
    </div>

    <div class="header-controls">
      <div class="search-input-wrapper">
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"></circle>
          <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
        </svg>
        <input
          type="text"
          placeholder="Buscar por pasta, tag ou arquivo..."
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
          {@const isExpanded = expandedSessions.has(session.session_id)}
          {@const isAllUndone = session.files_moved_count > 0 && session.undone_count === session.files_moved_count}
          {@const isPartialUndone = session.undone_count > 0 && session.undone_count < session.files_moved_count}

          <div class="session-card" class:is-undone={isAllUndone}>
            <!-- Top Card Bar -->
            <div class="session-card-header">
              <div class="session-main-info">
                <div class="folder-title-row">
                  <span class="folder-icon">📁</span>
                  <span class="folder-path" title={session.root_path}>{session.root_path}</span>
                  <button
                    class="icon-action-btn"
                    title="Abrir no Explorador"
                    on:click={() => handleOpenExplorer(session.root_path)}
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
                    📦 {session.files_moved_count} {session.files_moved_count === 1 ? "arquivo movido" : "arquivos movidos"}
                  </span>
                </div>
              </div>

              <!-- Status & Actions -->
              <div class="session-header-actions">
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
                    <span class="status-badge info">Varredura Concluída</span>
                  {/if}
                </div>

                {#if session.files_moved_count > 0 && !isAllUndone}
                  <button
                    class="undo-action-btn"
                    title="Desfazer todos os arquivos desta organização"
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
                  title={isExpanded ? "Recolher detalhes" : "Expandir arquivos"}
                >
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points={isExpanded ? "18 15 12 9 6 15" : "6 9 12 15 18 9"}></polyline>
                  </svg>
                </button>
              </div>
            </div>

            <!-- Categories / Tags created in this session -->
            {#if session.categories_assigned && session.categories_assigned.length > 0}
              <div class="session-categories-section">
                <span class="section-label">Tags / Categorias da Sessão:</span>
                <div class="category-tags-wrap">
                  {#each session.categories_assigned as catName}
                    <span class="category-chip">
                      <span class="chip-dot"></span>
                      {catName}
                    </span>
                  {/each}
                </div>
              </div>
            {/if}

            <!-- Expandable Move Log Details -->
            {#if isExpanded}
              <div class="session-details-drawer">
                <div class="drawer-header">
                  <h4>Arquivos Organizados ({session.moves.length})</h4>
                </div>

                <div class="moves-table-container">
                  {#if session.moves.length === 0}
                    <div class="no-moves-msg">Nenhum arquivo físico foi movido nesta sessão.</div>
                  {:else}
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
                  {/if}
                </div>
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
    gap: 0.75rem;
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
    gap: 1rem;
  }

  .session-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    overflow: hidden;
    transition: all 150ms ease;
  }

  .session-card:hover {
    border-color: var(--border-strong, #475569);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
  }

  .session-card.is-undone {
    opacity: 0.75;
    border-style: dashed;
  }

  .session-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    gap: 1rem;
    flex-wrap: wrap;
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

  .session-categories-section {
    padding: 0.5rem 1.25rem 0.85rem;
    border-top: 1px solid var(--border-subtle);
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
    background: rgba(0, 0, 0, 0.08);
  }

  .section-label {
    font-size: 0.76rem;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .category-tags-wrap {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
  }

  .category-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    color: var(--text-primary);
    font-size: 0.76rem;
    font-weight: 600;
    padding: 0.15rem 0.55rem;
    border-radius: var(--radius-full);
  }

  .chip-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--accent-primary);
  }

  .session-details-drawer {
    border-top: 1px solid var(--border-medium);
    background: var(--bg-primary);
    padding: 1rem 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    animation: fadeIn 150ms ease-out;
  }

  .drawer-header h4 {
    margin: 0;
    font-size: 0.85rem;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .moves-list {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    max-height: 260px;
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

  .no-moves-msg {
    font-size: 0.82rem;
    color: var(--text-muted);
    text-align: center;
    padding: 1.5rem;
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
