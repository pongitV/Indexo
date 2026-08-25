<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { _ } from "svelte-i18n";
  import {
    scanFolder,
    classifyScannedFiles,
    onScanProgress,
    onClassifyProgress,
  } from "../lib/api";
  import {
    selectedFolder,
    currentSessionId,
    currentView,
    scanSummary,
    classifiedFiles,
    showToast,
  } from "../lib/stores";

  let isDragging = false;

  async function handlePickFolder() {
    try {
      const path = await open({
        directory: true,
        multiple: false,
        title: $_("folder_select.cta"),
      });

      if (typeof path === "string" && path.trim().length > 0) {
        await processFolderPath(path);
      }
    } catch (err: any) {
      showToast("Erro ao abrir seletor de pasta: " + err, "error");
    }
  }

  async function processFolderPath(path: string) {
    selectedFolder.set(path);
    currentView.set("scanning");
    classifiedFiles.set([]);

    try {
      // 1. Iniciar Varredura
      const summary = await scanFolder(path);
      scanSummary.set(summary);
      currentSessionId.set(summary.session_id);

      // 2. Iniciar Classificação Semântica
      const results = await classifyScannedFiles(summary.session_id);
      classifiedFiles.set(results);

      // 3. Redirecionar para tela de Preview
      currentView.set("preview");
    } catch (err: any) {
      showToast("Erro durante a análise da pasta: " + err, "error");
      currentView.set("folder-select");
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    isDragging = true;
  }

  function handleDragLeave() {
    isDragging = false;
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    isDragging = false;
    // Tauri drag and drop event handling
    if (e.dataTransfer && e.dataTransfer.files.length > 0) {
      const file = e.dataTransfer.files[0];
      // Em WebView Tauri, o caminho pode ser acessado via file.path se disponível
      const fullPath = (file as any).path || file.name;
      if (fullPath) {
        processFolderPath(fullPath);
      }
    }
  }
</script>

<div class="folder-select-view">
  <div class="hero-section">
    <div class="hero-badge">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
      </svg>
      <span>100% Seguro & Local</span>
    </div>
    <h1>{$_("app.title")}</h1>
    <p class="hero-subtitle">{$_("app.subtitle")}</p>
  </div>

  <div
    class="drop-card glass-panel"
    class:dragging={isDragging}
    on:dragover={handleDragOver}
    on:dragleave={handleDragLeave}
    on:drop={handleDrop}
  >
    <div class="folder-graphic">
      <div class="graphic-glow"></div>
      <div class="graphic-icon">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
          <line x1="12" y1="11" x2="12" y2="17"></line>
          <polyline points="9 14 12 11 15 14"></polyline>
        </svg>
      </div>
    </div>

    <div class="cta-content">
      <h2>{$_("folder_select.hint")}</h2>
      <p class="subhint">{$_("folder_select.subhint")}</p>

      <button class="primary-btn" on:click={handlePickFolder}>
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
        </svg>
        {$_("folder_select.cta")}
      </button>
    </div>

    <div class="features-row">
      <div class="feature-item">
        <span class="dot emerald"></span>
        <span>Magic Bytes (Tipo Real)</span>
      </div>
      <div class="feature-item">
        <span class="dot blue"></span>
        <span>Clustering Semântico</span>
      </div>
      <div class="feature-item">
        <span class="dot violet"></span>
        <span>Auditoria & Undo Real</span>
      </div>
    </div>
  </div>
</div>

<style>
  .folder-select-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    max-width: 820px;
    margin: 0 auto;
    width: 100%;
    overflow-y: auto;
    min-height: 0;
    min-width: 0;
    animation: fadeIn 300ms ease-out;
  }

  .hero-section {
    text-align: center;
    margin-bottom: 2rem;
    flex-shrink: 0;
  }

  .hero-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.35rem 0.85rem;
    border-radius: var(--radius-full);
    background: var(--accent-light);
    color: var(--accent-primary);
    font-size: 0.78rem;
    font-weight: 700;
    margin-bottom: 1rem;
    border: 1px solid rgba(20, 184, 166, 0.2);
  }

  h1 {
    font-size: 2.2rem;
    font-weight: 800;
    letter-spacing: -0.03em;
    margin-bottom: 0.5rem;
    color: var(--text-primary);
  }

  .hero-subtitle {
    font-size: 1.05rem;
    color: var(--text-muted);
  }

  .drop-card {
    width: 100%;
    padding: 3rem 2rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    border: 2px dashed var(--border-medium);
    border-radius: var(--radius-lg);
    background: var(--bg-card);
    transition: all var(--transition-normal);
    flex-shrink: 0;
  }

  .drop-card.dragging {
    border-color: var(--accent-primary);
    background: var(--accent-light);
    transform: scale(1.01);
    box-shadow: 0 0 30px var(--accent-glow);
  }

  .folder-graphic {
    position: relative;
    margin-bottom: 1.5rem;
  }

  .graphic-glow {
    position: absolute;
    inset: -15px;
    border-radius: 50%;
    background: radial-gradient(circle, var(--accent-glow) 0%, transparent 70%);
    filter: blur(10px);
  }

  .graphic-icon {
    position: relative;
    width: 80px;
    height: 80px;
    border-radius: 20px;
    background: linear-gradient(135deg, var(--bg-secondary), var(--bg-tertiary));
    border: 1px solid var(--border-medium);
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--accent-primary);
    box-shadow: var(--shadow-lg);
  }

  .cta-content h2 {
    font-size: 1.25rem;
    font-weight: 700;
    margin-bottom: 0.5rem;
  }

  .subhint {
    font-size: 0.9rem;
    color: var(--text-muted);
    max-width: 500px;
    margin: 0 auto 1.75rem auto;
    line-height: 1.5;
  }

  .primary-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.85rem 2rem;
    font-size: 1rem;
    font-weight: 600;
    border-radius: var(--radius-md);
    background: var(--accent-primary);
    color: white;
    box-shadow: 0 4px 16px var(--accent-glow);
    transition: all var(--transition-fast);
  }

  .primary-btn:hover {
    background: var(--accent-primary-hover);
    transform: translateY(-2px);
    box-shadow: 0 6px 20px var(--accent-glow);
  }

  .features-row {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    margin-top: 2.5rem;
    padding-top: 1.5rem;
    border-top: 1px solid var(--border-subtle);
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .feature-item {
    display: flex;
    align-items: center;
    gap: 0.45rem;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .dot.emerald { background: var(--accent-emerald); box-shadow: 0 0 6px var(--accent-emerald); }
  .dot.blue { background: var(--accent-blue); box-shadow: 0 0 6px var(--accent-blue); }
  .dot.violet { background: var(--accent-violet); box-shadow: 0 0 6px var(--accent-violet); }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
