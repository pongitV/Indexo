<script lang="ts">
  import { _ } from "svelte-i18n";
  import type { FilePreviewData } from "./api";

  export let show: boolean = false;
  export let loading: boolean = false;
  export let data: FilePreviewData | null = null;
  export let categoryName: string | undefined = undefined;
  export let categoryColor: string | undefined = undefined;
  export let confidence: number | undefined = undefined;

  export let onClose: () => void;
  export let onOpenWithDefaultApp: (path: string) => void;
  export let onOpenInExplorer: (path: string) => void;

  let zoomLevel = 1.0;
  let copySuccess = false;

  $: if (!show) {
    zoomLevel = 1.0;
    copySuccess = false;
  }

  function handleZoomIn() {
    zoomLevel = Math.min(3.0, zoomLevel + 0.25);
  }

  function handleZoomOut() {
    zoomLevel = Math.max(0.5, zoomLevel - 0.25);
  }

  function handleZoomReset() {
    zoomLevel = 1.0;
  }

  function formatBytes(bytes: number): string {
    if (!bytes || bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
  }

  async function handleCopyText() {
    if (data?.text_content) {
      try {
        await navigator.clipboard.writeText(data.text_content);
        copySuccess = true;
        setTimeout(() => {
          copySuccess = false;
        }, 2500);
      } catch (_) {}
    }
  }

  // Parseia linhas de planilha tabuladas para exibição em tabela HTML
  function parseSpreadsheetRows(text: string): { headers: string[]; rows: string[][] } {
    const lines = text.split("\n").filter((l) => l.trim().length > 0);
    if (lines.length === 0) return { headers: [], rows: [] };

    const parsed: string[][] = [];
    for (const line of lines) {
      if (line.startsWith("Planilha:")) continue;
      const cols = line.split("|").map((c) => c.trim());
      parsed.push(cols);
    }

    if (parsed.length === 0) return { headers: [], rows: [] };

    const headers = parsed[0];
    const rows = parsed.slice(1);
    return { headers, rows };
  }
</script>

{#if show}
  <div
    class="preview-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={onClose}
    on:keydown={(e) => e.key === "Escape" && onClose()}
  >
    <div class="preview-modal-card glass-panel">
      <!-- Modal Header -->
      <div class="preview-header">
        <div class="header-left">
          <div class="file-icon-box">
            {#if data?.file_type === "image"}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#10b981" stroke-width="2">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                <circle cx="8.5" cy="8.5" r="1.5"></circle>
                <polyline points="21 15 16 10 5 21"></polyline>
              </svg>
            {:else if data?.file_type === "pdf"}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#ef4444" stroke-width="2">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                <polyline points="14 2 14 8 20 8"></polyline>
              </svg>
            {:else if data?.file_type === "spreadsheet"}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#059669" stroke-width="2">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                <line x1="3" y1="9" x2="21" y2="9"></line>
                <line x1="3" y1="15" x2="21" y2="15"></line>
                <line x1="9" y1="3" x2="9" y2="21"></line>
                <line x1="15" y1="3" x2="15" y2="21"></line>
              </svg>
            {:else if data?.file_type === "audio"}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#8b5cf6" stroke-width="2">
                <path d="M9 18V5l12-2v13"></path>
                <circle cx="6" cy="18" r="3"></circle>
                <circle cx="18" cy="16" r="3"></circle>
              </svg>
            {:else if data?.file_type === "video"}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#ec4899" stroke-width="2">
                <polygon points="23 7 16 12 23 17 23 7"></polygon>
                <rect x="1" y="5" width="15" height="14" rx="2" ry="2"></rect>
              </svg>
            {:else if data?.file_type === "code"}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="#06b6d4" stroke-width="2">
                <polyline points="16 18 22 12 16 6"></polyline>
                <polyline points="8 6 2 12 8 18"></polyline>
              </svg>
            {:else}
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
                <polyline points="14 2 14 8 20 8"></polyline>
              </svg>
            {/if}
          </div>
          <div class="header-titles">
            <h2 class="filename-title truncate" title={data?.filename ?? "Visualização de Arquivo"}>
              {data?.filename ?? "Carregando..."}
            </h2>
            {#if data}
              <div class="meta-row">
                <span class="meta-pill">{formatBytes(data.size_bytes)}</span>
                <span class="meta-pill mime-pill">{data.mime_type}</span>
                {#if data.dimensions}
                  <span class="meta-pill highlight-pill">
                    <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
                    </svg>
                    {data.dimensions}
                  </span>
                {/if}
                {#if data.exif_date}
                  <span class="meta-pill date-pill">
                    <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <rect x="3" y="4" width="18" height="18" rx="2" ry="2"></rect>
                      <line x1="16" y1="2" x2="16" y2="6"></line>
                      <line x1="8" y1="2" x2="8" y2="6"></line>
                      <line x1="3" y1="10" x2="21" y2="10"></line>
                    </svg>
                    EXIF: {data.exif_date}
                  </span>
                {/if}
                {#if data.line_count}
                  <span class="meta-pill">{data.line_count} linhas</span>
                {/if}
              </div>
            {/if}
          </div>
        </div>

        <div class="header-actions">
          {#if data}
            <button
              class="action-btn primary-action"
              title="Abrir no programa padrão registrado no Windows"
              on:click={() => onOpenWithDefaultApp(data.path)}
            >
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                <polyline points="15 3 21 3 21 9"></polyline>
                <line x1="10" y1="14" x2="21" y2="3"></line>
              </svg>
              Abrir no App Padrão
            </button>

            <button
              class="action-btn secondary-action"
              title="Revelar pasta no Windows Explorer"
              on:click={() => onOpenInExplorer(data.path)}
            >
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
              </svg>
              Explorador
            </button>
          {/if}

          <button class="close-icon-btn" title="Fechar (Esc)" on:click={onClose}>
            ✕
          </button>
        </div>
      </div>

      <!-- AI Classification Context Pill (Se disponível) -->
      {#if categoryName}
        <div class="ai-classification-bar">
          <div class="ai-tag-badge" style="border-left: 3px solid {categoryColor || '#3b82f6'};">
            <span class="ai-sparkle">✨</span>
            <span class="ai-label">Classificação Inteligente:</span>
            <strong style="color: {categoryColor || '#3b82f6'};">{categoryName}</strong>
            {#if confidence}
              <span class="ai-conf-pill">{Math.round(confidence * 100)}% confiança</span>
            {/if}
          </div>
        </div>
      {/if}

      <!-- Modal Body -->
      <div class="preview-body">
        {#if loading}
          <div class="state-container">
            <div class="spinner"></div>
            <span>Lendo e renderizando visualização do arquivo...</span>
          </div>
        {:else if data?.error}
          <div class="state-container error-state">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="#ef4444" stroke-width="2">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="8" x2="12" y2="12"></line>
              <line x1="12" y1="16" x2="12.01" y2="16"></line>
            </svg>
            <p>{data.error}</p>
          </div>
        {:else if data}
          <!-- Renderizador de Imagens -->
          {#if data.file_type === "image" && data.data_url}
            <div class="image-viewer-wrapper">
              <div class="zoom-toolbar">
                <button class="zoom-btn" on:click={handleZoomOut} title="Reduzir Zoom">-</button>
                <span class="zoom-indicator">{Math.round(zoomLevel * 100)}%</span>
                <button class="zoom-btn" on:click={handleZoomIn} title="Aumentar Zoom">+</button>
                <button class="zoom-btn reset-btn" on:click={handleZoomReset} title="Restaurar tamanho">Ajustar</button>
              </div>
              <div class="image-canvas">
                <img
                  src={data.data_url}
                  alt={data.filename}
                  style="transform: scale({zoomLevel});"
                  class="rendered-image"
                />
              </div>
            </div>

          <!-- Renderizador de Áudio -->
          {:else if data.file_type === "audio" && data.data_url}
            <div class="media-container audio-box">
              <div class="media-big-icon">
                <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="#8b5cf6" stroke-width="1.5">
                  <path d="M9 18V5l12-2v13"></path>
                  <circle cx="6" cy="18" r="3"></circle>
                  <circle cx="18" cy="16" r="3"></circle>
                </svg>
              </div>
              <audio controls src={data.data_url} class="audio-control"></audio>
            </div>

          <!-- Renderizador de Vídeo -->
          {:else if data.file_type === "video" && data.data_url}
            <div class="media-container video-box">
              <video controls src={data.data_url} class="video-control">
                <track kind="captions" />
              </video>
            </div>

          <!-- Renderizador de Planilhas em Tabela HTML -->
          {:else if data.file_type === "spreadsheet" && data.text_content}
            {@const tableData = parseSpreadsheetRows(data.text_content)}
            <div class="spreadsheet-container">
              {#if tableData.headers.length > 0}
                <div class="table-scroll">
                  <table class="spreadsheet-table">
                    <thead>
                      <tr>
                        <th class="row-num-col">#</th>
                        {#each tableData.headers as header}
                          <th>{header}</th>
                        {/each}
                      </tr>
                    </thead>
                    <tbody>
                      {#each tableData.rows as row, idx}
                        <tr>
                          <td class="row-num-col">{idx + 1}</td>
                          {#each row as cell}
                            <td>{cell}</td>
                          {/each}
                        </tr>
                      {/each}
                    </tbody>
                  </table>
                </div>
              {:else}
                <pre class="code-pre"><code>{data.text_content}</code></pre>
              {/if}
            </div>

          <!-- Renderizador de Documentos, Texto e Código -->
          {:else if data.text_content}
            <div class="text-viewer-wrapper">
              <div class="text-toolbar">
                <span class="text-info-badge">{data.file_type === "code" ? "Código Fonte" : "Texto / Documento"}</span>
                <button class="copy-btn" on:click={handleCopyText}>
                  {#if copySuccess}
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="#10b981" stroke-width="2.5">
                      <polyline points="20 6 9 17 4 12"></polyline>
                    </svg>
                    Copiado!
                  {:else}
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                    </svg>
                    Copiar Texto
                  {/if}
                </button>
              </div>
              <div class="code-editor-style">
                <pre class="code-pre"><code>{data.text_content}</code></pre>
              </div>
            </div>

          <!-- Formato Binário / Sem preview direto -->
          {:else}
            <div class="binary-fallback">
              <div class="binary-icon">
                <svg width="44" height="44" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                  <polyline points="21 8 21 21 3 21 3 8"></polyline>
                  <rect x="1" y="3" width="22" height="5"></rect>
                  <line x1="10" y1="12" x2="14" y2="12"></line>
                </svg>
              </div>
              <h3>Formato Binário ou Arquivo de Grande Porte</h3>
              <p>Este arquivo não possui visualização de texto direto integrada.</p>
              <button class="action-btn primary-action large" on:click={() => onOpenWithDefaultApp(data.path)}>
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                  <polyline points="15 3 21 3 21 9"></polyline>
                  <line x1="10" y1="14" x2="21" y2="3"></line>
                </svg>
                Abrir no Aplicativo Padrão do Windows
              </button>
            </div>
          {/if}
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="preview-footer">
        <div class="footer-path truncate" title={data?.path ?? ""}>
          {data?.path ?? ""}
        </div>
        <button class="secondary-btn" on:click={onClose}>
          Fechar
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .preview-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    padding: 1.5rem;
    animation: fadeIn 150ms ease-out;
  }

  .preview-modal-card {
    background: var(--bg-card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    width: 100%;
    max-width: 960px;
    height: 85vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
    overflow: hidden;
  }

  .preview-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-subtle);
    gap: 1rem;
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 0.85rem;
    min-width: 0;
  }

  .file-icon-box {
    font-size: 1.75rem;
    flex-shrink: 0;
  }

  .header-titles {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
  }

  .filename-title {
    font-size: 1.05rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .meta-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-wrap: wrap;
  }

  .meta-pill {
    font-size: 0.72rem;
    padding: 0.15rem 0.45rem;
    border-radius: var(--radius-sm);
    background: var(--bg-primary);
    color: var(--text-secondary);
    border: 1px solid var(--border-subtle);
  }

  .mime-pill {
    font-family: monospace;
  }

  .highlight-pill {
    color: #06b6d4;
    border-color: rgba(6, 182, 212, 0.3);
  }

  .date-pill {
    color: #10b981;
    border-color: rgba(16, 185, 129, 0.3);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-shrink: 0;
  }

  .action-btn {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.45rem 0.85rem;
    border-radius: var(--radius-md);
    font-size: 0.82rem;
    font-weight: 600;
    cursor: pointer;
    transition: all var(--transition-fast);
    border: none;
  }

  .primary-action {
    background: var(--primary);
    color: white;
  }

  .primary-action:hover {
    background: var(--primary-hover);
  }

  .primary-action.large {
    padding: 0.75rem 1.25rem;
    font-size: 0.92rem;
    margin-top: 1rem;
  }

  .secondary-action {
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--border-subtle);
  }

  .secondary-action:hover {
    background: var(--bg-hover);
  }

  .close-icon-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 1.1rem;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-md);
    cursor: pointer;
  }

  .close-icon-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .ai-classification-bar {
    padding: 0.4rem 1.25rem;
    background: rgba(13, 148, 136, 0.06);
    border-bottom: 1px solid var(--border-subtle);
  }

  .ai-tag-badge {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    font-size: 0.8rem;
    padding-left: 0.5rem;
  }

  .ai-label {
    color: var(--text-muted);
  }

  .ai-conf-pill {
    font-size: 0.72rem;
    background: rgba(255, 255, 255, 0.1);
    padding: 0.1rem 0.35rem;
    border-radius: 4px;
    color: var(--text-secondary);
  }

  .preview-body {
    flex: 1;
    overflow: auto;
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .state-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    color: var(--text-muted);
  }

  .spinner {
    width: 36px;
    height: 36px;
    border: 3px solid rgba(255, 255, 255, 0.1);
    border-top-color: var(--primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  .image-viewer-wrapper {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.75rem;
    min-height: 0;
  }

  .zoom-toolbar {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    background: var(--bg-secondary);
    padding: 0.25rem 0.5rem;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
  }

  .zoom-btn {
    background: transparent;
    border: none;
    color: var(--text-primary);
    font-weight: bold;
    cursor: pointer;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
  }

  .zoom-btn:hover {
    background: var(--bg-hover);
  }

  .zoom-indicator {
    font-size: 0.75rem;
    color: var(--text-muted);
    min-width: 40px;
    text-align: center;
  }

  .image-canvas {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: auto;
    width: 100%;
    background: rgba(0, 0, 0, 0.25);
    border-radius: var(--radius-md);
    padding: 1rem;
  }

  .rendered-image {
    max-width: 100%;
    max-height: 55vh;
    object-fit: contain;
    border-radius: var(--radius-sm);
    transition: transform 150ms ease-out;
  }

  .media-container {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1.5rem;
  }

  .media-big-icon {
    font-size: 3.5rem;
  }

  .audio-control {
    width: 100%;
    max-width: 500px;
  }

  .video-control {
    max-width: 100%;
    max-height: 55vh;
    border-radius: var(--radius-md);
  }

  .spreadsheet-container {
    flex: 1;
    overflow: auto;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
  }

  .table-scroll {
    overflow: auto;
    max-height: 100%;
  }

  .spreadsheet-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.82rem;
    text-align: left;
  }

  .spreadsheet-table th,
  .spreadsheet-table td {
    padding: 0.45rem 0.75rem;
    border: 1px solid var(--border-subtle);
  }

  .spreadsheet-table th {
    background: var(--bg-secondary);
    font-weight: 600;
    color: var(--text-primary);
    position: sticky;
    top: 0;
  }

  .row-num-col {
    width: 40px;
    text-align: center;
    background: var(--bg-secondary);
    color: var(--text-muted);
    font-size: 0.75rem;
  }

  .spreadsheet-table tbody tr:nth-child(even) {
    background: rgba(255, 255, 255, 0.02);
  }

  .text-viewer-wrapper {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
    gap: 0.5rem;
  }

  .text-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
  }

  .text-info-badge {
    font-size: 0.75rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .copy-btn {
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    color: var(--text-primary);
    padding: 0.25rem 0.6rem;
    border-radius: var(--radius-sm);
    font-size: 0.75rem;
    cursor: pointer;
  }

  .code-editor-style {
    flex: 1;
    overflow: auto;
    background: #0d1117;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
    padding: 1rem;
  }

  .code-pre {
    margin: 0;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.82rem;
    line-height: 1.6;
    color: #e6edf3;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .binary-fallback {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    color: var(--text-muted);
    gap: 0.5rem;
  }

  .binary-icon {
    font-size: 3rem;
  }

  .preview-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1.25rem;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-subtle);
    gap: 1rem;
    flex-shrink: 0;
  }

  .footer-path {
    font-size: 0.78rem;
    color: var(--text-muted);
    font-family: monospace;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
</style>
