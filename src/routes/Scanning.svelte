<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { _ } from "svelte-i18n";
  import {
    onScanProgress,
    onClassifyProgress,
    type ScanProgressPayload,
    type ClassifyProgressPayload,
  } from "../lib/api";
  import {
    selectedFolder,
    currentView,
    scanSummary,
  } from "../lib/stores";

  let filesScanned = 0;
  let totalSizeBytes = 0;
  let currentFile = "";
  let currentPhase = "scanning"; // "scanning" | "heuristics" | "extracting" | "clustering" | "done"
  let processedCount = 0;
  let totalCount = 0;

  let unlistenScan: (() => void) | null = null;
  let unlistenClassify: (() => void) | null = null;

  onMount(async () => {
    unlistenScan = await onScanProgress((payload: ScanProgressPayload) => {
      filesScanned = payload.files_scanned;
      totalSizeBytes = payload.total_size_bytes;
      currentFile = payload.current_file;
      currentPhase = "scanning";
    });

    unlistenClassify = await onClassifyProgress((payload: ClassifyProgressPayload) => {
      processedCount = payload.processed;
      totalCount = payload.total;
      currentPhase = payload.current_phase;
      if (payload.item) {
        currentFile = payload.item.filename;
      }
    });
  });

  onDestroy(() => {
    if (unlistenScan) unlistenScan();
    if (unlistenClassify) unlistenClassify();
  });

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }

  function getPhaseLabel(phase: string): string {
    switch (phase) {
      case "scanning":
        return $_("scanning.progress");
      case "heuristics":
        return $_("classify.phase.heuristics");
      case "extracting":
        return $_("classify.phase.extracting");
      case "clustering":
        return $_("classify.phase.clustering");
      case "done":
        return $_("classify.phase.done");
      default:
        return $_("classify.progress");
    }
  }

  $: progressPercent = totalCount > 0
    ? Math.round((processedCount / totalCount) * 100)
    : 0;
</script>

<div class="scanning-view">
  <div class="status-card glass-panel">
    <!-- Animated Processing Pulse -->
    <div class="radar-container">
      <div class="pulse-ring ring-1"></div>
      <div class="pulse-ring ring-2"></div>
      <div class="pulse-ring ring-3"></div>
      <div class="radar-center">
        <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"></circle>
          <polyline points="12 6 12 12 16 14"></polyline>
        </svg>
      </div>
    </div>

    <!-- Title and Status -->
    <div class="status-info">
      <h2>{currentPhase === "scanning" ? $_("scanning.title") : $_("classify.title")}</h2>
      <p class="phase-text">{getPhaseLabel(currentPhase)}</p>
    </div>

    <!-- Progress Bar -->
    <div class="progress-section">
      <div class="progress-bar-track">
        {#if currentPhase === "scanning"}
          <div class="progress-bar-indeterminate"></div>
        {:else}
          <div class="progress-bar-fill" style="width: {progressPercent}%"></div>
        {/if}
      </div>
      <div class="progress-labels">
        {#if currentPhase === "scanning"}
          <span>{filesScanned} {$_("scanning.files")}</span>
          <span>{formatBytes(totalSizeBytes)}</span>
        {:else}
          <span>{processedCount} de {totalCount} {$_("preview.total_files")}</span>
          <span>{progressPercent}%</span>
        {/if}
      </div>
    </div>

    <!-- Stats Grid -->
    <div class="stats-grid">
      <div class="stat-box">
        <span class="stat-label">{$_("scanning.files")}</span>
        <span class="stat-value">{filesScanned}</span>
      </div>
      <div class="stat-box">
        <span class="stat-label">{$_("scanning.size")}</span>
        <span class="stat-value">{formatBytes(totalSizeBytes)}</span>
      </div>
      <div class="stat-box full-width" title={currentFile || $selectedFolder || "..."}>
        <span class="stat-label">{$_("scanning.current")}</span>
        <span class="stat-value mono truncate">{currentFile || $selectedFolder || "..."}</span>
      </div>
    </div>

    <button class="cancel-btn" on:click={() => currentView.set("folder-select")}>
      Cancelar Análise
    </button>
  </div>
</div>

<style>
  .scanning-view {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
    overflow-y: auto;
    min-height: 0;
    min-width: 0;
    animation: fadeIn 300ms ease-out;
  }

  .status-card {
    width: 100%;
    max-width: 580px;
    padding: 2.25rem 2.5rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    border-radius: var(--radius-lg);
    flex-shrink: 0;
  }

  /* Radar Pulse Animation */
  .radar-container {
    position: relative;
    width: 90px;
    height: 90px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 1.5rem;
  }

  .pulse-ring {
    position: absolute;
    border-radius: 50%;
    border: 2px solid var(--accent-primary);
    animation: pulseWave 2.4s cubic-bezier(0.215, 0.61, 0.355, 1) infinite;
  }

  .ring-1 { width: 100%; height: 100%; animation-delay: 0s; }
  .ring-2 { width: 100%; height: 100%; animation-delay: 0.8s; }
  .ring-3 { width: 100%; height: 100%; animation-delay: 1.6s; }

  .radar-center {
    position: relative;
    width: 60px;
    height: 60px;
    border-radius: 50%;
    background: linear-gradient(135deg, var(--accent-primary), #0284c7);
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 0 20px var(--accent-glow);
    z-index: 2;
  }

  .status-info h2 {
    font-size: 1.35rem;
    font-weight: 700;
    margin-bottom: 0.35rem;
    color: var(--text-primary);
  }

  .phase-text {
    font-size: 0.92rem;
    color: var(--accent-primary);
    font-weight: 600;
    min-height: 24px;
  }

  .progress-section {
    width: 100%;
    margin: 1.75rem 0;
  }

  .progress-bar-track {
    width: 100%;
    height: 8px;
    border-radius: var(--radius-full);
    background: var(--bg-tertiary);
    overflow: hidden;
    position: relative;
    border: 1px solid var(--border-subtle);
  }

  .progress-bar-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--accent-primary), #38bdf8);
    border-radius: var(--radius-full);
    transition: width 200ms ease;
  }

  .progress-bar-indeterminate {
    position: absolute;
    height: 100%;
    width: 40%;
    background: linear-gradient(90deg, transparent, var(--accent-primary), transparent);
    animation: indeterminateSlide 1.5s infinite linear;
  }

  .progress-labels {
    display: flex;
    justify-content: space-between;
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text-muted);
    margin-top: 0.4rem;
  }

  .stats-grid {
    width: 100%;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
    margin-bottom: 1.5rem;
  }

  .stat-box {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    padding: 0.75rem;
    border-radius: var(--radius-md);
    text-align: left;
  }

  .stat-box.full-width {
    grid-column: 1 / -1;
  }

  .stat-label {
    display: block;
    font-size: 0.72rem;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.04em;
    margin-bottom: 0.2rem;
  }

  .stat-value {
    font-size: 1rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .mono {
    font-family: var(--font-mono);
    font-size: 0.82rem;
    font-weight: 500;
  }

  .truncate {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    display: block;
  }

  .cancel-btn {
    padding: 0.6rem 1.25rem;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-muted);
    background: transparent;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
    transition: all var(--transition-fast);
  }

  .cancel-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  @keyframes pulseWave {
    0% { transform: scale(0.6); opacity: 1; }
    100% { transform: scale(1.6); opacity: 0; }
  }

  @keyframes indeterminateSlide {
    from { left: -40%; }
    to { left: 100%; }
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
