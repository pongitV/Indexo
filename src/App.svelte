<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import {
    currentView,
    theme,
    language,
    selectedFolder,
    classifiedFiles,
    toastMessages,
  } from "./lib/stores";
  import { getSetting, saveSetting } from "./lib/api";

  import FolderSelect from "./routes/FolderSelect.svelte";
  import Scanning from "./routes/Scanning.svelte";
  import Preview from "./routes/Preview.svelte";
  import TagManager from "./routes/TagManager.svelte";
  import Settings from "./routes/Settings.svelte";

  onMount(async () => {
    // Carregar configuracoes salvas do SQLite
    try {
      const savedTheme = await getSetting("theme");
      if (savedTheme && (savedTheme === "light" || savedTheme === "dark" || savedTheme === "system")) {
        theme.set(savedTheme as any);
      }
      const savedLang = await getSetting("language");
      if (savedLang && (savedLang === "pt-BR" || savedLang === "en-US")) {
        language.set(savedLang as any);
      }
    } catch (_) {}
  });

  // Atualizar tema no HTML
  $: {
    if (typeof document !== "undefined") {
      if ($theme === "system") {
        document.documentElement.removeAttribute("data-theme");
      } else {
        document.documentElement.setAttribute("data-theme", $theme);
      }
    }
  }

  function toggleTheme() {
    const nextTheme = $theme === "dark" ? "light" : $theme === "light" ? "system" : "dark";
    theme.set(nextTheme);
    saveSetting("theme", nextTheme).catch(() => {});
  }
</script>

<div class="app-layout">
  <!-- Top Navigation Bar -->
  <header class="app-header">
    <div class="header-left">
      <div
        class="logo-box"
        role="button"
        tabindex="0"
        on:click={() => currentView.set("folder-select")}
        on:keydown={(e) => (e.key === "Enter" || e.key === " ") && currentView.set("folder-select")}
      >
        <div class="logo-icon">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
          </svg>
        </div>
        <div class="logo-text">
          <span class="brand-title">{$_("app.title")}</span>
          <span class="offline-badge">
            <svg width="10" height="10" viewBox="0 0 24 24" fill="currentColor">
              <circle cx="12" cy="12" r="10"></circle>
            </svg>
            100% Offline
          </span>
        </div>
      </div>
    </div>

    <!-- Navigation Pills -->
    <nav class="header-nav">
      <button
        class="nav-btn"
        class:active={$currentView === "folder-select" || $currentView === "scanning"}
        on:click={() => currentView.set("folder-select")}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
        </svg>
        {$_("nav.folder")}
      </button>

      <button
        class="nav-btn"
        class:active={$currentView === "preview"}
        disabled={$classifiedFiles.length === 0 && $currentView !== "preview"}
        on:click={() => currentView.set("preview")}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
          <polyline points="14 2 14 8 20 8"></polyline>
          <line x1="16" y1="13" x2="8" y2="13"></line>
          <line x1="16" y1="17" x2="8" y2="17"></line>
          <polyline points="10 9 9 9 8 9"></polyline>
        </svg>
        {$_("nav.preview")}
        {#if $classifiedFiles.length > 0}
          <span class="counter-badge">{$classifiedFiles.length}</span>
        {/if}
      </button>

      <button
        class="nav-btn"
        class:active={$currentView === "tags"}
        on:click={() => currentView.set("tags")}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"></path>
          <line x1="7" y1="7" x2="7.01" y2="7"></line>
        </svg>
        {$_("nav.tags")}
      </button>

      <button
        class="nav-btn"
        class:active={$currentView === "settings"}
        on:click={() => currentView.set("settings")}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="3"></circle>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
        </svg>
        {$_("nav.settings")}
      </button>
    </nav>

    <!-- Header Actions -->
    <div class="header-right">
      <button class="icon-toggle-btn" title="Alternar Tema ({$theme})" on:click={toggleTheme}>
        {#if $theme === "dark"}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
          </svg>
        {:else if $theme === "light"}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="5"></circle>
            <line x1="12" y1="1" x2="12" y2="3"></line>
            <line x1="12" y1="21" x2="12" y2="23"></line>
            <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
            <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
            <line x1="1" y1="12" x2="3" y2="12"></line>
            <line x1="21" y1="12" x2="23" y2="12"></line>
            <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
            <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
          </svg>
        {:else}
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect>
            <line x1="8" y1="21" x2="16" y2="21"></line>
            <line x1="12" y1="17" x2="12" y2="21"></line>
          </svg>
        {/if}
      </button>
    </div>
  </header>

  <!-- Main View Container -->
  <main class="app-content">
    {#if $currentView === "folder-select"}
      <FolderSelect />
    {:else if $currentView === "scanning"}
      <Scanning />
    {:else if $currentView === "preview"}
      <Preview />
    {:else if $currentView === "tags"}
      <TagManager />
    {:else if $currentView === "settings"}
      <Settings />
    {/if}
  </main>

  <!-- Toast Notification System -->
  {#if $toastMessages.length > 0}
    <div class="toast-container">
      {#each $toastMessages as toast (toast.id)}
        <div class="toast-item {toast.type}">
          {#if toast.type === "success"}
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
          {:else if toast.type === "error"}
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="15" y1="9" x2="9" y2="15"></line>
              <line x1="9" y1="9" x2="15" y2="15"></line>
            </svg>
          {:else}
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"></circle>
              <line x1="12" y1="16" x2="12" y2="12"></line>
              <line x1="12" y1="8" x2="12.01" y2="8"></line>
            </svg>
          {/if}
          <span>{toast.message}</span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .app-layout {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--bg-primary);
    color: var(--text-primary);
    overflow: hidden;
  }

  .app-header {
    height: 64px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1.5rem;
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-card);
    backdrop-filter: blur(12px);
    z-index: 50;
    flex-shrink: 0;
  }

  .header-left {
    display: flex;
    align-items: center;
  }

  .logo-box {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    cursor: pointer;
  }

  .logo-icon {
    width: 36px;
    height: 36px;
    border-radius: var(--radius-md);
    background: linear-gradient(135deg, var(--accent-primary), #0284c7);
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 4px 12px var(--accent-glow);
  }

  .logo-text {
    display: flex;
    flex-direction: column;
  }

  .brand-title {
    font-weight: 700;
    font-size: 1.05rem;
    letter-spacing: -0.02em;
    background: linear-gradient(135deg, var(--text-primary), var(--accent-primary));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .offline-badge {
    font-size: 0.68rem;
    font-weight: 600;
    color: var(--accent-emerald);
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }

  .header-nav {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: var(--bg-tertiary);
    padding: 0.3rem;
    border-radius: var(--radius-full);
    border: 1px solid var(--border-subtle);
  }

  .nav-btn {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.45rem 1rem;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-secondary);
    background: transparent;
    border-radius: var(--radius-full);
    transition: all var(--transition-fast);
  }

  .nav-btn:hover:not(:disabled) {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .nav-btn.active {
    background: var(--accent-primary);
    color: white;
    box-shadow: 0 2px 8px var(--accent-glow);
  }

  .nav-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .counter-badge {
    background: rgba(255, 255, 255, 0.25);
    color: white;
    font-size: 0.7rem;
    font-weight: 700;
    padding: 0.1rem 0.4rem;
    border-radius: var(--radius-full);
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .icon-toggle-btn {
    width: 38px;
    height: 38px;
    border-radius: var(--radius-md);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition-fast);
  }

  .icon-toggle-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
    border-color: var(--border-medium);
  }

  .app-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
    min-height: 0;
    min-width: 0;
  }

  /* Toast Container */
  .toast-container {
    position: fixed;
    bottom: 1.5rem;
    right: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    z-index: 1000;
    pointer-events: none;
  }

  .toast-item {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.75rem 1.25rem;
    border-radius: var(--radius-md);
    font-size: 0.88rem;
    font-weight: 500;
    background: var(--bg-secondary);
    color: var(--text-primary);
    border: 1px solid var(--border-subtle);
    box-shadow: var(--shadow-xl);
    pointer-events: auto;
    animation: toastSlideIn 250ms ease-out forwards;
  }

  .toast-item.success {
    border-left: 4px solid var(--accent-emerald);
    color: var(--text-primary);
  }

  .toast-item.error {
    border-left: 4px solid var(--accent-rose);
  }

  @keyframes toastSlideIn {
    from {
      opacity: 0;
      transform: translateY(12px) scale(0.95);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }
</style>
