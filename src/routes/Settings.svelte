<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { locale, _ } from "svelte-i18n";
  import { theme, language, showToast } from "../lib/stores";
  import {
    exportProfile,
    importProfile,
    saveSetting,
  } from "../lib/api";

  async function handleThemeChange(t: "light" | "dark" | "system") {
    theme.set(t);
    try {
      await saveSetting("theme", t);
    } catch (_) {}
  }

  async function handleLanguageChange(l: "pt-BR" | "en-US") {
    language.set(l);
    locale.set(l);
    try {
      await saveSetting("language", l);
    } catch (_) {}
  }

  async function handleExportProfile() {
    try {
      const destination = await save({
        filters: [{ name: "Zip Archive", extensions: ["zip"] }],
        defaultPath: "organizador-perfil-backup.zip",
        title: $_("settings.backup_profile"),
      });

      if (typeof destination === "string" && destination.trim()) {
        await exportProfile(destination);
        showToast($_("settings.toast.backup_success"), "success");
      }
    } catch (err: any) {
      showToast("Erro ao exportar backup: " + err, "error");
    }
  }

  async function handleRestoreProfile() {
    try {
      const source = await open({
        filters: [{ name: "Zip Archive", extensions: ["zip"] }],
        multiple: false,
        title: $_("settings.restore_profile"),
      });

      if (typeof source === "string" && source.trim()) {
        await importProfile(source);
        showToast($_("settings.toast.restore_success"), "success");
      }
    } catch (err: any) {
      showToast("Erro ao restaurar perfil: " + err, "error");
    }
  }
</script>

<div class="settings-view">
  <div class="settings-header">
    <h1>{$_("settings.title")}</h1>
    <p class="subtitle">{$_("settings.subtitle")}</p>
  </div>

  <div class="settings-content">
    <!-- Theme Section -->
    <section class="settings-card glass-panel">
      <div class="card-header">
        <div class="card-icon">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
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
        </div>
        <div>
          <h2>{$_("settings.theme")}</h2>
          <p class="card-desc">Selecione o esquema de cores preferido para a interface.</p>
        </div>
      </div>

      <div class="theme-options-grid">
        <button
          class="theme-card"
          class:active={$theme === "light"}
          on:click={() => handleThemeChange("light")}
        >
          <div class="theme-preview light">
            <div class="preview-bar"></div>
            <div class="preview-body"></div>
          </div>
          <span>{$_("settings.theme.light")}</span>
        </button>

        <button
          class="theme-card"
          class:active={$theme === "dark"}
          on:click={() => handleThemeChange("dark")}
        >
          <div class="theme-preview dark">
            <div class="preview-bar"></div>
            <div class="preview-body"></div>
          </div>
          <span>{$_("settings.theme.dark")}</span>
        </button>

        <button
          class="theme-card"
          class:active={$theme === "system"}
          on:click={() => handleThemeChange("system")}
        >
          <div class="theme-preview system">
            <div class="preview-bar"></div>
            <div class="preview-body"></div>
          </div>
          <span>{$_("settings.theme.system")}</span>
        </button>
      </div>
    </section>

    <!-- Language Section -->
    <section class="settings-card glass-panel">
      <div class="card-header">
        <div class="card-icon">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="2" y1="12" x2="22" y2="12"></line>
            <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"></path>
          </svg>
        </div>
        <div>
          <h2>{$_("settings.language")}</h2>
          <p class="card-desc">Alterne o idioma de exibição do aplicativo e categorias.</p>
        </div>
      </div>

      <div class="lang-buttons-row">
        <button
          class="lang-btn"
          class:active={$language === "pt-BR"}
          on:click={() => handleLanguageChange("pt-BR")}
        >
          <span class="flag">🇧🇷</span>
          <span>{$_("settings.language.pt")}</span>
        </button>

        <button
          class="lang-btn"
          class:active={$language === "en-US"}
          on:click={() => handleLanguageChange("en-US")}
        >
          <span class="flag">🇺🇸</span>
          <span>{$_("settings.language.en")}</span>
        </button>
      </div>
    </section>

    <!-- Backup & Portability Section -->
    <section class="settings-card glass-panel">
      <div class="card-header">
        <div class="card-icon">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
            <polyline points="17 8 12 3 7 8"></polyline>
            <line x1="12" y1="3" x2="12" y2="15"></line>
          </svg>
        </div>
        <div>
          <h2>{$_("settings.backup_section")}</h2>
          <p class="card-desc">{$_("settings.backup_desc")}</p>
        </div>
      </div>

      <div class="backup-buttons-row">
        <button class="primary-btn" on:click={handleExportProfile}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
            <polyline points="7 10 12 15 17 10"></polyline>
            <line x1="12" y1="15" x2="12" y2="3"></line>
          </svg>
          {$_("settings.backup_profile")}
        </button>

        <button class="secondary-btn" on:click={handleRestoreProfile}>
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
            <polyline points="17 8 12 3 7 8"></polyline>
            <line x1="12" y1="3" x2="12" y2="15"></line>
          </svg>
          {$_("settings.restore_profile")}
        </button>
      </div>
    </section>

    <!-- Safety & Offline Guarantee Section -->
    <section class="settings-card glass-panel safety-card">
      <div class="card-header">
        <div class="card-icon emerald">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
          </svg>
        </div>
        <div>
          <h2>{$_("settings.safety_section")}</h2>
          <p class="card-desc">{$_("settings.safety_desc")}</p>
        </div>
      </div>
    </section>
  </div>
</div>

<style>
  .settings-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 1.25rem 1.5rem;
    gap: 1.25rem;
    overflow-y: auto;
    overflow-x: hidden;
    max-width: 800px;
    margin: 0 auto;
    width: 100%;
    min-height: 0;
    min-width: 0;
    animation: fadeIn 250ms ease-out;
  }

  .settings-header {
    flex-shrink: 0;
  }

  .settings-header h1 {
    font-size: 1.4rem;
    font-weight: 800;
    letter-spacing: -0.02em;
  }

  .subtitle {
    font-size: 0.84rem;
    color: var(--text-muted);
  }

  .settings-content {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    padding-bottom: 1.5rem;
  }

  .settings-card {
    padding: 1.5rem;
    border-radius: var(--radius-lg);
    background: var(--bg-card);
    border: 1px solid var(--border-subtle);
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    flex-shrink: 0;
  }

  .card-header {
    display: flex;
    align-items: flex-start;
    gap: 0.85rem;
  }

  .card-icon {
    width: 40px;
    height: 40px;
    border-radius: var(--radius-md);
    background: var(--bg-tertiary);
    color: var(--accent-primary);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .card-icon.emerald {
    background: var(--accent-light);
    color: var(--accent-emerald);
  }

  .card-header h2 {
    font-size: 1.05rem;
    font-weight: 700;
    margin-bottom: 0.2rem;
  }

  .card-desc {
    font-size: 0.84rem;
    color: var(--text-muted);
    line-height: 1.4;
  }

  /* Theme Options */
  .theme-options-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 1rem;
  }

  .theme-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.6rem;
    padding: 0.85rem;
    border-radius: var(--radius-md);
    background: var(--bg-tertiary);
    border: 2px solid var(--border-subtle);
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-primary);
    transition: all var(--transition-fast);
  }

  .theme-card:hover {
    border-color: var(--border-medium);
  }

  .theme-card.active {
    border-color: var(--accent-primary);
    background: var(--accent-light);
  }

  .theme-preview {
    width: 100%;
    height: 44px;
    border-radius: var(--radius-sm);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border-subtle);
  }

  .theme-preview.light { background: #f8fafc; }
  .theme-preview.light .preview-bar { height: 12px; background: #e2e8f0; }

  .theme-preview.dark { background: #0f172a; }
  .theme-preview.dark .preview-bar { height: 12px; background: #1e293b; }

  .theme-preview.system { background: linear-gradient(135deg, #f8fafc 50%, #0f172a 50%); }
  .theme-preview.system .preview-bar { height: 12px; background: #64748b; }

  /* Language Buttons */
  .lang-buttons-row, .backup-buttons-row {
    display: flex;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .lang-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.6rem;
    padding: 0.75rem 1.25rem;
    font-size: 0.9rem;
    font-weight: 600;
    border-radius: var(--radius-md);
    background: var(--bg-tertiary);
    border: 2px solid var(--border-subtle);
    color: var(--text-primary);
    transition: all var(--transition-fast);
  }

  .lang-btn.active {
    border-color: var(--accent-primary);
    background: var(--accent-light);
  }

  .flag {
    font-size: 1.1rem;
  }

  .primary-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.4rem;
    font-size: 0.9rem;
    font-weight: 600;
    border-radius: var(--radius-md);
    background: var(--accent-primary);
    color: white;
    box-shadow: 0 4px 12px var(--accent-glow);
  }

  .primary-btn:hover {
    background: var(--accent-primary-hover);
  }

  .secondary-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem 1.4rem;
    font-size: 0.9rem;
    font-weight: 600;
    border-radius: var(--radius-md);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-medium);
    color: var(--text-primary);
  }

  .secondary-btn:hover {
    background: var(--bg-hover);
  }

  .safety-card {
    border-left: 4px solid var(--accent-emerald);
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(6px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
