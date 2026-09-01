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
    aiFolderSuggestions,
  } from "./lib/stores";
  import { getSetting, saveSetting } from "./lib/api";

  import FolderSelect from "./routes/FolderSelect.svelte";
  import Scanning from "./routes/Scanning.svelte";
  import Preview from "./routes/Preview.svelte";
  import Renamer from "./routes/Renamer.svelte";
  import Duplicates from "./routes/Duplicates.svelte";
  import TagManager from "./routes/TagManager.svelte";
  import CategoryManager from "./routes/CategoryManager.svelte";
  import History from "./routes/History.svelte";
  import RulesManager from "./routes/RulesManager.svelte";
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

    // Escutar sugestões semânticas da IA
    try {
      await onClassifySuggestions((suggestions) => {
        aiFolderSuggestions.update((existing) => {
          const paths = new Set(existing.map((s) => s.folder_path));
          const newOnes = suggestions.filter((s) => !paths.has(s.folder_path));
          return [...existing, ...newOnes];
        });
      });
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

  let openDropdown: "functions" | "manage" | null = null;
  let showEasterEgg: boolean = false;
  let mouseAnimKey: number = 0;

  function toggleDropdown(name: "functions" | "manage") {
    openDropdown = openDropdown === name ? null : name;
  }

  function closeDropdowns() {
    openDropdown = null;
  }

  function triggerEasterEgg() {
    showEasterEgg = true;
    mouseAnimKey += 1;
    closeDropdowns();
  }

  function toggleTheme() {
    const nextTheme = $theme === "dark" ? "light" : $theme === "light" ? "system" : "dark";
    theme.set(nextTheme);
    saveSetting("theme", nextTheme).catch(() => {});
  }
</script>

<svelte:window
  on:click={(e) => {
    const target = e.target;
    if (target && !target.closest('.dropdown-container')) {
      closeDropdowns();
    }
  }}
  on:keydown={(e) => {
    if (e.key === "Escape") {
      closeDropdowns();
      showEasterEgg = false;
    }
  }}
/>

<div class="app-layout">
  <!-- Top Navigation Bar -->
  <header class="app-header">
    <div class="header-left">
      <div
        class="logo-box"
        role="button"
        tabindex="0"
        title="Indexo - Clique para descobrir um segredo!"
        on:click={triggerEasterEgg}
        on:keydown={(e) => (e.key === "Enter" || e.key === " ") && triggerEasterEgg()}
      >
        <div class="logo-icon">
          <svg width="32" height="26" viewBox="0 0 100 80" fill="none">
            <!-- Bookmark tab on top left -->
            <path d="M16 2h18v22l-9-6l-9 6V2z" fill="#D14D41" stroke="#AF3029" stroke-width="1.5" />
            <line x1="25" y1="5" x2="25" y2="15" stroke="#D0A215" stroke-width="1.5" />

            <!-- 5 Book Spines standing side-by-side -->
            <!-- Book 1 (Teal) -->
            <rect x="6" y="16" width="16" height="58" rx="3" fill="#24837B" stroke="#1B635D" stroke-width="1.5" />
            <line x1="9" y1="24" x2="19" y2="24" stroke="#D0A215" stroke-width="1.5" />
            <line x1="9" y1="64" x2="19" y2="64" stroke="#D0A215" stroke-width="1.5" />

            <!-- Book 2 (Lapis Blue) -->
            <rect x="24" y="16" width="16" height="58" rx="3" fill="#4385BE" stroke="#205EA6" stroke-width="1.5" />
            <line x1="27" y1="30" x2="37" y2="30" stroke="#FFFCF0" stroke-width="1.2" />
            <line x1="27" y1="34" x2="37" y2="34" stroke="#FFFCF0" stroke-width="1.2" />

            <!-- Book 3 (Amber/Ochre) -->
            <rect x="42" y="16" width="16" height="58" rx="3" fill="#DA702C" stroke="#BC5215" stroke-width="1.5" />
            <line x1="45" y1="24" x2="55" y2="24" stroke="#D0A215" stroke-width="1.5" />
            <line x1="45" y1="64" x2="55" y2="64" stroke="#D0A215" stroke-width="1.5" />

            <!-- Book 4 (Sage Laurel) -->
            <rect x="60" y="16" width="16" height="58" rx="3" fill="#879A39" stroke="#66800B" stroke-width="1.5" />
            <line x1="63" y1="36" x2="73" y2="36" stroke="#FFFCF0" stroke-width="1.2" />

            <!-- Book 5 (Royal Purple) -->
            <rect x="78" y="16" width="16" height="58" rx="3" fill="#8B7EC8" stroke="#5E409D" stroke-width="1.5" />
            <line x1="81" y1="24" x2="91" y2="24" stroke="#D0A215" stroke-width="1.5" />
            <line x1="81" y1="64" x2="91" y2="64" stroke="#D0A215" stroke-width="1.5" />

            <!-- Bottom shelf support / base -->
            <rect x="3" y="73" width="94" height="6" rx="3" fill="#543A22" stroke="#382412" stroke-width="1.5" />
          </svg>
        </div>
        <div class="logo-text">
          <span class="brand-title">{$_("app.title")}</span>
        </div>
      </div>
    </div>

    <!-- Navigation Pills with Dropdowns -->
    <nav class="header-nav">
      <!-- 1. Selecionar Pasta -->
      <button
        class="nav-btn"
        class:active={$currentView === "folder-select" || $currentView === "scanning"}
        on:click={() => { currentView.set("folder-select"); closeDropdowns(); }}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
        </svg>
        <span>Selecionar Pasta</span>
      </button>

      <!-- 2. Preview (Visível quando há arquivos escaneados ou quando ativo) -->
      {#if $classifiedFiles.length > 0 || $currentView === "preview"}
        <button
          class="nav-btn"
          class:active={$currentView === "preview"}
          on:click={() => { currentView.set("preview"); closeDropdowns(); }}
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path>
            <polyline points="14 2 14 8 20 8"></polyline>
          </svg>
          <span>Visualizar</span>
          {#if $classifiedFiles.length > 0}
            <span class="counter-badge">{$classifiedFiles.length}</span>
          {/if}
        </button>
      {/if}

      <!-- 3. Dropdown Funções (Renomear, Duplicatas, Histórico) -->
      <div class="dropdown-container">
        <button
          class="nav-btn dropdown-trigger"
          class:active={$currentView === "renamer" || $currentView === "duplicates" || $currentView === "history"}
          class:is-open={openDropdown === "functions"}
          on:click|stopPropagation={() => toggleDropdown("functions")}
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polygon points="12 2 2 7 12 12 22 7 12 2"></polygon>
            <polyline points="2 17 12 22 22 17"></polyline>
            <polyline points="2 12 12 17 22 12"></polyline>
          </svg>
          <span>Funções</span>
          <svg
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            class="chevron-icon"
            class:open={openDropdown === "functions"}
          >
            <polyline points="6 9 12 15 18 9"></polyline>
          </svg>
        </button>

        {#if openDropdown === "functions"}
          <div class="dropdown-menu glass-panel">
            <button
              class="dropdown-item"
              class:selected={$currentView === "renamer"}
              on:click={() => { currentView.set("renamer"); closeDropdowns(); }}
            >
              <div class="dropdown-item-icon">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                  <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                </svg>
              </div>
              <div class="item-text">
                <span class="item-title">Renomear</span>
                <span class="item-desc">Padronização semântica por IA</span>
              </div>
            </button>

            <button
              class="dropdown-item"
              class:selected={$currentView === "duplicates"}
              on:click={() => { currentView.set("duplicates"); closeDropdowns(); }}
            >
              <div class="dropdown-item-icon">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                </svg>
              </div>
              <div class="item-text">
                <span class="item-title">Duplicatas</span>
                <span class="item-desc">Detecção view-first com descarte seguro</span>
              </div>
            </button>

            <button
              class="dropdown-item"
              class:selected={$currentView === "history"}
              on:click={() => { currentView.set("history"); closeDropdowns(); }}
            >
              <div class="dropdown-item-icon">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"></circle>
                  <polyline points="12 6 12 12 16 14"></polyline>
                </svg>
              </div>
              <div class="item-text">
                <span class="item-title">Histórico</span>
                <span class="item-desc">Auditoria, árvores e estatísticas</span>
              </div>
            </button>
          </div>
        {/if}
      </div>

      <!-- 2. Dropdown Ferramentas (Renomear, Duplicatas, Histórico) -->
      <div class="dropdown-container">
        <button
          class="nav-btn dropdown-trigger"
          class:active={$currentView === "renamer" || $currentView === "duplicates" || $currentView === "history"}
          class:is-open={openDropdown === "tools"}
          on:click|stopPropagation={() => toggleDropdown("tools")}
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon>
          </svg>
          <span>Ferramentas</span>
          <svg
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            class="chevron-icon"
            class:open={openDropdown === "tools"}
          >
            <polyline points="6 9 12 15 18 9"></polyline>
          </svg>
        </button>

        {#if openDropdown === "tools"}
          <div class="dropdown-menu glass-panel">
            <button
              class="dropdown-item"
              class:selected={$currentView === "renamer"}
              on:click={() => { currentView.set("renamer"); closeDropdowns(); }}
            >
              <div class="dropdown-item-icon">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                  <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                </svg>
              </div>
              <div class="item-text">
                <span class="item-title">Renomear</span>
                <span class="item-desc">Padronização semântica por IA</span>
              </div>
            </button>

            <button
              class="dropdown-item"
              class:selected={$currentView === "duplicates"}
              on:click={() => { currentView.set("duplicates"); closeDropdowns(); }}
            >
              <div class="dropdown-item-icon">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
                  <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
                </svg>
              </div>
              <div class="item-text">
                <span class="item-title">Duplicatas</span>
                <span class="item-desc">Detecção view-first com descarte seguro</span>
              </div>
            </button>

            <button
              class="dropdown-item"
              class:selected={$currentView === "history"}
              on:click={() => { currentView.set("history"); closeDropdowns(); }}
            >
              <div class="dropdown-item-icon">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <circle cx="12" cy="12" r="10"></circle>
                  <polyline points="12 6 12 12 16 14"></polyline>
                </svg>
              </div>
              <div class="item-text">
                <span class="item-title">Histórico</span>
                <span class="item-desc">Auditoria, árvores e estatísticas</span>
              </div>
            </button>
          </div>
        {/if}
      </div>

      <!-- 3. Dropdown Gerenciar (Regras, Categorias, Tags) -->
      <div class="dropdown-container">
        <button
          class="nav-btn dropdown-trigger"
          class:active={$currentView === "tags" || $currentView === "categories" || $currentView === "rules"}
          class:is-open={openDropdown === "manage"}
          on:click|stopPropagation={() => toggleDropdown("manage")}
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="4" y1="21" x2="4" y2="14"></line>
            <line x1="4" y1="10" x2="4" y2="3"></line>
            <line x1="12" y1="21" x2="12" y2="12"></line>
            <line x1="12" y1="8" x2="12" y2="3"></line>
            <line x1="20" y1="21" x2="20" y2="16"></line>
            <line x1="20" y1="12" x2="20" y2="3"></line>
            <line x1="1" y1="14" x2="7" y2="14"></line>
            <line x1="9" y1="8" x2="15" y2="8"></line>
            <line x1="17" y1="16" x2="23" y2="16"></line>
          </svg>
          <span>Gerenciar</span>
          {#if $aiFolderSuggestions.length > 0}
            <span class="nav-alert-dot" title="Novas sugestões de pastas disponíveis"></span>
          {/if}
          <svg
            width="12"
            height="12"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            class="chevron-icon"
            class:open={openDropdown === "manage"}
          >
            <polyline points="6 9 12 15 18 9"></polyline>
          </svg>
        </button>

        {#if openDropdown === "manage"}
          <div class="dropdown-menu glass-panel">
            <button
              class="dropdown-item"
              class:selected={$currentView === "rules"}
              on:click={() => { currentView.set("rules"); closeDropdowns(); }}
            >
              <div class="dropdown-item-icon">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path>
                  <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path>
                </svg>
              </div>
              <div class="item-text">
                <div class="item-title-with-badge">
                  <span class="item-title">Regras & Heurísticas</span>
                  {#if $aiFolderSuggestions.length > 0}
                    <span class="menu-alert-pill">{$aiFolderSuggestions.length}</span>
                  {/if}
                </div>
                <span class="item-desc">Extensões, subpastas e regras padrão</span>
              </div>
            </button>

            <button
              class="dropdown-item"
              class:selected={$currentView === "categories"}
              on:click={() => { currentView.set("categories"); closeDropdowns(); }}
            >
              <div class="dropdown-item-icon">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
                  <polyline points="2 10 22 10"></polyline>
                </svg>
              </div>
              <div class="item-text">
                <span class="item-title">Categorias</span>
                <span class="item-desc">Pastas de destino e mesclagem</span>
              </div>
            </button>

            <button
              class="dropdown-item"
              class:selected={$currentView === "tags"}
              on:click={() => { currentView.set("tags"); closeDropdowns(); }}
            >
              <div class="dropdown-item-icon">
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"></path>
                  <line x1="7" y1="7" x2="7.01" y2="7"></line>
                </svg>
              </div>
              <div class="item-text">
                <span class="item-title">Tags</span>
                <span class="item-desc">Etiquetas semânticas e cores</span>
              </div>
            </button>
          </div>
        {/if}
      </div>

      <!-- 5. Configurações -->
      <button
        class="nav-btn"
        class:active={$currentView === "settings"}
        on:click={() => { currentView.set("settings"); closeDropdowns(); }}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="3"></circle>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
        </svg>
        <span>Configurações</span>
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
    {:else if $currentView === "renamer"}
      <Renamer />
    {:else if $currentView === "duplicates"}
      <Duplicates />
    {:else if $currentView === "tags"}
      <TagManager />
    {:else if $currentView === "categories"}
      <CategoryManager />
    {:else if $currentView === "history"}
      <History />
    {:else if $currentView === "rules"}
      <RulesManager />
    {:else if $currentView === "settings"}
      <Settings />
    {/if}
  </main>

  <!-- Easter Egg Modal: Biblioteca de Alexandria & Ratinho Acenando -->
  {#if showEasterEgg}
    <div
      class="easter-egg-backdrop"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      on:click|self={() => (showEasterEgg = false)}
      on:keydown={(e) => e.key === "Escape" && (showEasterEgg = false)}
    >
      <div class="easter-egg-card glass-panel">
        <button class="modal-close-btn" title="Fechar" on:click={() => (showEasterEgg = false)}>✕</button>

        <!-- Alexandria Library & Animated Mouse Stage -->
        <div class="alexandria-stage-wrapper">
          <div class="stage-glow"></div>

          <!-- Scene Container (Folder + Mouse) -->
          <div class="mouse-stage" key={mouseAnimKey}>
            <!-- The Mouse Actor (Peeks out from behind the folder, waves and goes back) -->
            <div class="mouse-actor">
              <svg width="84" height="84" viewBox="0 0 100 100" fill="none">
                <!-- Left Ear -->
                <circle cx="28" cy="28" r="16" fill="#94a3b8" />
                <circle cx="28" cy="28" r="10" fill="#f472b6" opacity="0.8" />

                <!-- Right Ear -->
                <circle cx="72" cy="28" r="16" fill="#94a3b8" />
                <circle cx="72" cy="28" r="10" fill="#f472b6" opacity="0.8" />

                <!-- Head -->
                <ellipse cx="50" cy="52" rx="26" ry="24" fill="#94a3b8" />
                <!-- Cheeks / Snout area -->
                <ellipse cx="50" cy="58" rx="16" ry="12" fill="#e2e8f0" />

                <!-- Left Eye -->
                <circle cx="41" cy="48" r="3.5" fill="#0f172a" />
                <circle cx="42" cy="46.5" r="1.2" fill="#ffffff" />

                <!-- Right Eye (Winking / Twinkling) -->
                <circle cx="59" cy="48" r="3.5" fill="#0f172a" />
                <circle cx="60" cy="46.5" r="1.2" fill="#ffffff" />

                <!-- Nose -->
                <polygon points="50,56 46,53 54,53" fill="#f43f5e" />

                <!-- Whiskers -->
                <line x1="28" y1="56" x2="42" y2="56" stroke="#475569" stroke-width="1.5" stroke-linecap="round" class="whisker-left" />
                <line x1="26" y1="60" x2="42" y2="58" stroke="#475569" stroke-width="1.5" stroke-linecap="round" class="whisker-left" />
                <line x1="72" y1="56" x2="58" y2="56" stroke="#475569" stroke-width="1.5" stroke-linecap="round" class="whisker-right" />
                <line x1="74" y1="60" x2="58" y2="58" stroke="#475569" stroke-width="1.5" stroke-linecap="round" class="whisker-right" />

                <!-- Smile -->
                <path d="M47 60 Q50 63 53 60" stroke="#0f172a" stroke-width="1.5" fill="none" stroke-linecap="round" />

                <!-- Waving Right Paw -->
                <g class="mouse-waving-paw">
                  <ellipse cx="76" cy="44" rx="6" ry="5" fill="#e2e8f0" stroke="#94a3b8" stroke-width="1.5" />
                  <circle cx="78" cy="41" r="1.5" fill="#f472b6" />
                  <circle cx="75" cy="40" r="1.5" fill="#f472b6" />
                </g>
              </svg>
            </div>

            <!-- Front Bookshelf/Folder Cover that conceals the mouse when it is down -->
            <div class="folder-front-cover">
              <svg width="110" height="84" viewBox="0 0 100 80" fill="none">
                <!-- Bookmark tab on top left -->
                <path d="M16 2h18v22l-9-6l-9 6V2z" fill="#D14D41" stroke="#AF3029" stroke-width="1.5" />
                <line x1="25" y1="5" x2="25" y2="15" stroke="#D0A215" stroke-width="1.5" />

                <!-- 5 Book Spines standing side-by-side -->
                <!-- Book 1 (Teal) -->
                <rect x="6" y="16" width="16" height="58" rx="3" fill="#24837B" stroke="#1B635D" stroke-width="1.5" />
                <line x1="9" y1="24" x2="19" y2="24" stroke="#D0A215" stroke-width="1.5" />
                <line x1="9" y1="64" x2="19" y2="64" stroke="#D0A215" stroke-width="1.5" />

                <!-- Book 2 (Lapis Blue) -->
                <rect x="24" y="16" width="16" height="58" rx="3" fill="#4385BE" stroke="#205EA6" stroke-width="1.5" />
                <line x1="27" y1="30" x2="37" y2="30" stroke="#FFFCF0" stroke-width="1.2" />
                <line x1="27" y1="34" x2="37" y2="34" stroke="#FFFCF0" stroke-width="1.2" />

                <!-- Book 3 (Amber/Ochre) -->
                <rect x="42" y="16" width="16" height="58" rx="3" fill="#DA702C" stroke="#BC5215" stroke-width="1.5" />
                <line x1="45" y1="24" x2="55" y2="24" stroke="#D0A215" stroke-width="1.5" />
                <line x1="45" y1="64" x2="55" y2="64" stroke="#D0A215" stroke-width="1.5" />

                <!-- Book 4 (Sage Laurel) -->
                <rect x="60" y="16" width="16" height="58" rx="3" fill="#879A39" stroke="#66800B" stroke-width="1.5" />
                <line x1="63" y1="36" x2="73" y2="36" stroke="#FFFCF0" stroke-width="1.2" />

                <!-- Book 5 (Royal Purple) -->
                <rect x="78" y="16" width="16" height="58" rx="3" fill="#8B7EC8" stroke="#5E409D" stroke-width="1.5" />
                <line x1="81" y1="24" x2="91" y2="24" stroke="#D0A215" stroke-width="1.5" />
                <line x1="81" y1="64" x2="91" y2="64" stroke="#D0A215" stroke-width="1.5" />

                <!-- Bottom shelf support / base -->
                <rect x="3" y="73" width="94" height="6" rx="3" fill="#543A22" stroke="#382412" stroke-width="1.5" />
              </svg>
            </div>
          </div>
        </div>

        <!-- App Info & Legendary Quote -->
        <div class="easter-egg-info">
          <div class="app-identity-row">
            <h2 class="app-name-title">Indexo</h2>
            <span class="version-pill">v0.1.0</span>
          </div>

          <blockquote class="alexandria-quote">
            “Index and organize like the library of Alexandria”
          </blockquote>

          <div class="easter-explanation-card">
            <div class="explanation-item">
              <span class="exp-badge">Objetivo</span>
              <p>Estruturar, categorizar e catalogar seus arquivos de forma inteligente, determinística e segura, eliminando o caos digital e transformando pastas desordenadas em um arquivo pessoal impecável.</p>
            </div>
            <div class="explanation-item">
              <span class="exp-badge">Como faz</span>
              <p>Analisa tipos reais (Magic Bytes) e contexto semântico sem mover nada sem sua aprovação prévia, sugere árvores de pastas temáticas e nomes padronizados com preview total, detecta duplicatas por hash exato e oferece auditoria com reversão em um clique.</p>
            </div>
          </div>

          <div class="easter-actions">
            <button class="primary-btn close-modal-action" on:click={() => (showEasterEgg = false)}>
              Entendido
            </button>
          </div>
        </div>
      </div>
    </div>
  {/if}

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
    padding: 0.35rem 0.5rem;
    border-radius: var(--radius-md);
    transition: transform 120ms ease, background 120ms ease;
  }

  .logo-box:hover {
    background: var(--bg-hover);
    transform: scale(1.02);
  }

  .logo-icon {
    width: 38px;
    height: 38px;
    border-radius: var(--radius-md);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: var(--shadow-sm);
    padding: 2px;
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

  /* Navigation Bar & Dropdowns */
  .header-nav {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    background: var(--bg-tertiary);
    padding: 0.3rem 0.45rem;
    border-radius: var(--radius-full);
    border: 1px solid var(--border-subtle);
  }

  .nav-btn {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.45rem 0.9rem;
    font-size: 0.84rem;
    font-weight: 600;
    color: var(--text-secondary);
    background: transparent;
    border: none;
    border-radius: var(--radius-full);
    cursor: pointer;
    transition: all var(--transition-fast);
    user-select: none;
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

  .dropdown-container {
    position: relative;
    display: flex;
    align-items: center;
  }

  .dropdown-trigger {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
  }

  .chevron-icon {
    transition: transform 180ms ease;
  }

  .chevron-icon.open {
    transform: rotate(180deg);
  }

  .dropdown-menu {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    min-width: 220px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    padding: 0.4rem;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.3), 0 8px 10px -6px rgba(0, 0, 0, 0.2);
    z-index: 100;
    animation: dropdownSlide 140ms ease-out forwards;
  }

  @keyframes dropdownSlide {
    from {
      opacity: 0;
      transform: translateY(-6px) scale(0.97);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  .dropdown-item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.6rem 0.75rem;
    background: transparent;
    border: none;
    border-radius: var(--radius-md);
    cursor: pointer;
    text-align: left;
    width: 100%;
    color: var(--text-primary);
    transition: all 120ms ease;
  }

  .dropdown-item:hover {
    background: var(--bg-hover);
  }

  .dropdown-item.selected {
    background: rgba(59, 130, 246, 0.14);
    border-left: 3px solid var(--accent-primary);
  }

  .dropdown-item-icon {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--radius-sm);
    background: var(--bg-tertiary);
    color: var(--accent-primary);
    flex-shrink: 0;
  }

  .item-text {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    min-width: 0;
  }

  .item-title {
    font-size: 0.84rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .item-desc {
    font-size: 0.72rem;
    color: var(--text-muted);
    line-height: 1.2;
  }

  .item-title-with-badge {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .nav-alert-dot {
    width: 7px;
    height: 7px;
    background: #d14d41;
    border-radius: 50%;
    margin-left: -0.1rem;
    margin-right: 0.1rem;
  }

  .menu-alert-pill {
    background: #d14d41;
    color: #fff;
    font-size: 0.65rem;
    font-weight: 700;
    padding: 0.05rem 0.35rem;
    border-radius: var(--radius-full);
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
    cursor: pointer;
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

  /* Easter Egg Modal & Alexandria Mouse Animation */
  .easter-egg-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 4000;
    padding: 1rem;
    animation: fadeIn 150ms ease-out;
  }

  .easter-egg-card {
    position: relative;
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-xl);
    padding: 2.25rem 2rem 1.75rem 2rem;
    max-width: 440px;
    width: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    box-shadow: var(--shadow-xl);
  }

  .modal-close-btn {
    position: absolute;
    top: 1rem;
    right: 1rem;
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 1.1rem;
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    border-radius: var(--radius-sm);
    transition: color 120ms ease;
  }

  .modal-close-btn:hover {
    color: var(--text-primary);
  }

  .alexandria-stage-wrapper {
    position: relative;
    width: 140px;
    height: 120px;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    margin-bottom: 1rem;
  }

  .stage-glow {
    position: absolute;
    bottom: 0;
    width: 100px;
    height: 60px;
    background: radial-gradient(circle, rgba(59, 130, 246, 0.35) 0%, transparent 70%);
    border-radius: 50%;
    pointer-events: none;
  }

  .mouse-stage {
    position: relative;
    width: 100px;
    height: 110px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-end;
  }

  .mouse-actor {
    position: absolute;
    bottom: 24px;
    animation: mousePeekAndWave 3.5s ease-in-out forwards;
    transform-origin: bottom center;
    z-index: 1;
  }

  .mouse-waving-paw {
    transform-origin: 70px 48px;
    animation: pawWaving 600ms ease-in-out 1.2s 3 alternate;
  }

  .folder-front-cover {
    position: relative;
    z-index: 2;
    filter: drop-shadow(0 6px 12px rgba(0, 0, 0, 0.3));
  }

  @keyframes mousePeekAndWave {
    0% {
      transform: translateY(45px) scale(0.7);
      opacity: 0;
    }
    18% {
      transform: translateY(0px) scale(1);
      opacity: 1;
    }
    30% {
      transform: translateY(-8px) scale(1.02);
      opacity: 1;
    }
    68% {
      transform: translateY(-8px) scale(1.02);
      opacity: 1;
    }
    82% {
      transform: translateY(0px) scale(1);
      opacity: 1;
    }
    100% {
      transform: translateY(45px) scale(0.7);
      opacity: 0;
    }
  }

  @keyframes pawWaving {
    0% {
      transform: rotate(0deg);
    }
    100% {
      transform: rotate(-28deg);
    }
  }

  .app-identity-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    margin-bottom: 0.6rem;
  }

  .app-name-title {
    font-size: 1.4rem;
    font-weight: 800;
    margin: 0;
    letter-spacing: -0.02em;
    background: linear-gradient(135deg, var(--text-primary), var(--accent-primary));
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .version-pill {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-medium);
    color: var(--text-muted);
    font-size: 0.74rem;
    font-weight: 700;
    padding: 0.15rem 0.5rem;
    border-radius: var(--radius-full);
  }

  .alexandria-quote {
    font-family: Georgia, serif;
    font-style: italic;
    font-size: 1.05rem;
    color: var(--text-primary);
    line-height: 1.45;
    margin: 0.4rem 0 0.6rem 0;
    padding: 0.4rem 0.8rem;
    border-left: 3px solid var(--accent-primary);
    background: rgba(59, 130, 246, 0.05);
    border-radius: var(--radius-sm);
  }

  .easter-explanation-card {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    padding: 0.9rem 1rem;
    margin: 0.75rem 0 1.25rem 0;
    text-align: left;
  }

  .explanation-item {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .exp-badge {
    align-self: flex-start;
    font-size: 0.72rem;
    font-weight: 800;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 0.15rem 0.45rem;
    border-radius: var(--radius-sm);
    background: var(--accent-light);
    color: var(--accent-primary);
  }

  .explanation-item p {
    font-size: 0.82rem;
    color: var(--text-secondary);
    line-height: 1.4;
    margin: 0;
  }

  .easter-actions {
    display: flex;
    align-items: center;
    width: 100%;
    justify-content: center;
  }

  .close-modal-action {
    background: var(--accent-primary);
    color: white;
    border: none;
    padding: 0.6rem 2rem;
    border-radius: var(--radius-md);
    font-size: 0.85rem;
    font-weight: 700;
    cursor: pointer;
    box-shadow: 0 4px 12px var(--accent-glow);
    transition: all var(--transition-fast);
  }

  .close-modal-action:hover {
    background: var(--accent-primary-hover);
    transform: translateY(-1px);
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
    border-left: 4px solid #10b981;
    color: var(--text-primary);
  }

  .toast-item.error {
    border-left: 4px solid #ef4444;
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

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(4px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
