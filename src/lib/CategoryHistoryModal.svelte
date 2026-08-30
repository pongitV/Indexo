<script lang="ts">
  import { onMount } from "svelte";
  import { getCategoryHistory, type CategoryHistoryRecord } from "./api";

  export let show: boolean = false;
  export let categoryId: string | null = null;
  export let categoryName: string = "";
  export let categoryColor: string | null = null;
  export let onClose: () => void;

  let history: CategoryHistoryRecord[] = [];
  let loading: boolean = false;
  let error: string | null = null;

  $: if (show && categoryId) {
    loadHistory(categoryId);
  }

  async function loadHistory(id: string) {
    loading = true;
    error = null;
    try {
      history = await getCategoryHistory(id);
    } catch (e: any) {
      error = e?.toString() ?? "Erro ao carregar histórico";
    } finally {
      loading = false;
    }
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
        second: "2-digit",
      });
    } catch {
      return dateStr;
    }
  }

  function getAuthorBadge(changedBy: string): { label: string; className: string } {
    switch (changedBy) {
      case "user":
        return { label: "Usuário", className: "badge-user" };
      case "ai_refinement":
        return { label: "Refinamento IA", className: "badge-ai" };
      case "merge":
        return { label: "Fusão de Tags", className: "badge-merge" };
      default:
        return { label: "Sistema", className: "badge-auto" };
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
    <div class="modal-card history-modal-card">
      <div class="modal-header-row">
        <div class="header-title-box">
          <div class="title-with-pill">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="history-icon-svg">
              <circle cx="12" cy="12" r="10"></circle>
              <polyline points="12 6 12 12 16 14"></polyline>
            </svg>
            <h2>Histórico de Mudanças</h2>
            {#if categoryColor}
              <span class="color-indicator" style="background: {categoryColor};"></span>
            {/if}
          </div>
          <p class="modal-subtitle">
            Linha do tempo de alterações e refinamentos da tag: <strong>{categoryName}</strong>
          </p>
        </div>
        <button class="close-btn" on:click={onClose}>✕</button>
      </div>

      <div class="history-modal-body">
        {#if loading}
          <div class="loading-state">
            <div class="mini-spinner"></div>
            <span>Carregando histórico...</span>
          </div>
        {:else if error}
          <div class="error-box">
            <p>{error}</p>
          </div>
        {:else if history.length === 0}
          <div class="empty-history-state">
            <div class="empty-icon">✨</div>
            <h3>Nome Original Mantido</h3>
            <p>
              Esta tag/categoria nunca teve seu nome alterado e mantém o nome original <strong>"{categoryName}"</strong> desde sua criação.
            </p>
          </div>
        {:else}
          <div class="timeline-container">
            <div class="timeline-header-info">
              <span>{history.length} {history.length === 1 ? "alteração registrada" : "alterações registradas"}</span>
            </div>

            <div class="timeline-list">
              {#each history as item, index (item.id)}
                {@const badge = getAuthorBadge(item.changed_by)}
                <div class="timeline-item">
                  <div class="timeline-node-track">
                    <div class="timeline-node" class:latest={index === 0}>
                      {index === 0 ? "★" : "•"}
                    </div>
                    {#if index < history.length - 1}
                      <div class="timeline-line"></div>
                    {/if}
                  </div>

                  <div class="timeline-content-card">
                    <div class="timeline-card-header">
                      <div class="author-tag {badge.className}">
                        <span>{badge.label}</span>
                      </div>
                      <time class="timeline-timestamp">{formatDateTime(item.changed_at)}</time>
                    </div>

                    <div class="name-transition-box">
                      <div class="name-pill old-name" title="Nome Anterior">
                        <span class="pill-label">De:</span>
                        <span class="pill-value">{item.old_name}</span>
                      </div>
                      <div class="arrow-transition">➔</div>
                      <div class="name-pill new-name" title="Novo Nome">
                        <span class="pill-label">Para:</span>
                        <span class="pill-value">{item.new_name}</span>
                      </div>
                    </div>

                    {#if item.reason}
                      <div class="reason-note">
                        <span class="reason-label">Motivo:</span>
                        <span class="reason-text">{item.reason}</span>
                      </div>
                    {/if}
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>

      <div class="modal-actions">
        <button class="primary-btn" on:click={onClose}>
          Fechar
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
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

  .history-modal-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-xl);
    padding: 1.5rem;
    max-width: 620px;
    width: 100%;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    overflow: hidden;
  }

  .modal-header-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;
    border-bottom: 1px solid var(--border-subtle);
    padding-bottom: 0.85rem;
  }

  .header-title-box {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .title-with-pill {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .history-icon-badge {
    font-size: 1.25rem;
  }

  .header-title-box h2 {
    font-size: 1.2rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .color-indicator {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    display: inline-block;
  }

  .modal-subtitle {
    font-size: 0.85rem;
    color: var(--text-muted);
    margin: 0;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 1.2rem;
    cursor: pointer;
    padding: 0.2rem 0.5rem;
    border-radius: var(--radius-sm);
    transition: all 120ms ease;
  }

  .close-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .history-modal-body {
    flex: 1;
    overflow-y: auto;
    padding: 0.25rem 0.5rem 0.25rem 0;
    scrollbar-width: thin;
    scrollbar-color: var(--border-medium) transparent;
  }

  .loading-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.75rem;
    padding: 3rem 1rem;
    color: var(--text-muted);
  }

  .mini-spinner {
    width: 24px;
    height: 24px;
    border: 2px solid rgba(59, 130, 246, 0.2);
    border-top-color: var(--accent-primary);
    border-radius: 50%;
    animation: spin 800ms linear infinite;
  }

  .error-box {
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #ef4444;
    padding: 0.75rem 1rem;
    border-radius: var(--radius-md);
    font-size: 0.88rem;
  }

  .empty-history-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 0.6rem;
    padding: 2.5rem 1.5rem;
    background: var(--bg-primary);
    border: 1px dashed var(--border-medium);
    border-radius: var(--radius-lg);
  }

  .empty-icon {
    font-size: 2.5rem;
  }

  .empty-history-state h3 {
    margin: 0;
    font-size: 1.05rem;
    color: var(--text-primary);
  }

  .empty-history-state p {
    margin: 0;
    font-size: 0.85rem;
    color: var(--text-muted);
    line-height: 1.45;
    max-width: 380px;
  }

  .timeline-header-info {
    font-size: 0.78rem;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 0.75rem;
  }

  .timeline-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .timeline-item {
    display: flex;
    gap: 0.85rem;
    align-items: flex-start;
  }

  .timeline-node-track {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 24px;
    flex-shrink: 0;
  }

  .timeline-node {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: var(--bg-tertiary);
    border: 2px solid var(--border-medium);
    color: var(--text-muted);
    font-size: 0.75rem;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .timeline-node.latest {
    background: var(--accent-primary);
    border-color: var(--accent-primary);
    color: white;
    box-shadow: 0 0 8px rgba(59, 130, 246, 0.4);
  }

  .timeline-line {
    width: 2px;
    flex: 1;
    min-height: 48px;
    background: var(--border-subtle);
    margin-top: 4px;
  }

  .timeline-content-card {
    flex: 1;
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    padding: 0.85rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    transition: border-color 150ms ease;
  }

  .timeline-content-card:hover {
    border-color: var(--border-medium);
  }

  .timeline-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .author-tag {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.75rem;
    font-weight: 700;
    padding: 0.15rem 0.55rem;
    border-radius: var(--radius-full);
  }

  .badge-user {
    background: rgba(59, 130, 246, 0.12);
    color: #3b82f6;
  }

  .badge-ai {
    background: rgba(139, 92, 246, 0.12);
    color: #8b5cf6;
  }

  .badge-merge {
    background: rgba(245, 158, 11, 0.12);
    color: #f59e0b;
  }

  .badge-auto {
    background: rgba(107, 114, 128, 0.12);
    color: var(--text-muted);
  }

  .timeline-timestamp {
    font-size: 0.75rem;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .name-transition-box {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .name-pill {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.25rem 0.6rem;
    border-radius: var(--radius-sm);
    font-size: 0.82rem;
  }

  .name-pill.old-name {
    background: rgba(239, 68, 68, 0.08);
    color: #ef4444;
    text-decoration: line-through;
    opacity: 0.85;
  }

  .name-pill.new-name {
    background: rgba(16, 185, 129, 0.1);
    color: #10b981;
    font-weight: 700;
  }

  .pill-label {
    font-size: 0.7rem;
    opacity: 0.75;
    text-transform: uppercase;
    font-weight: 600;
  }

  .arrow-transition {
    color: var(--text-muted);
    font-size: 0.85rem;
  }

  .reason-note {
    font-size: 0.78rem;
    color: var(--text-muted);
    background: var(--bg-secondary);
    padding: 0.35rem 0.6rem;
    border-radius: var(--radius-sm);
    display: flex;
    gap: 0.4rem;
  }

  .reason-label {
    font-weight: 600;
    color: var(--text-primary);
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    border-top: 1px solid var(--border-subtle);
    padding-top: 0.85rem;
  }

  .primary-btn {
    background: var(--accent-primary);
    color: white;
    border: none;
    padding: 0.5rem 1.25rem;
    border-radius: var(--radius-md);
    font-size: 0.86rem;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 150ms ease;
  }

  .primary-btn:hover {
    opacity: 0.9;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
