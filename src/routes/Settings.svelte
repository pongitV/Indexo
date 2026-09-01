<script lang="ts">
  import { open, save } from "@tauri-apps/plugin-dialog";
  import { locale, _ } from "svelte-i18n";
  import {
    currentView,
    theme,
    language,
    showToast,
    selectedFolder,
    classifiedFiles,
    aiFolderSuggestions,
  } from "../lib/stores";
  import {
    exportProfile,
    importProfile,
    saveSetting,
    clearAllUserData,
  } from "../lib/api";

  let showClearModal = false;
  let clearConfirmInput = "";
  let isClearing = false;

  async function handleExecuteClearAll() {
    if (clearConfirmInput.trim().toLowerCase() !== "sim") {
      showToast("Digite 'sim' para autorizar a exclusão.", "error");
      return;
    }

    isClearing = true;
    try {
      await clearAllUserData("sim");
      classifiedFiles.set([]);
      selectedFolder.set("");
      aiFolderSuggestions.set([]);
      showClearModal = false;
      showToast("Todos os dados do perfil foram limpos e resetados com sucesso!", "success");
      currentView.set("folder-select");
    } catch (err: any) {
      showToast("Erro ao limpar dados: " + err, "error");
    } finally {
      isClearing = false;
    }
  }

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
          <span class="lang-code-pill">PT</span>
          <span>{$_("settings.language.pt")}</span>
        </button>

        <button
          class="lang-btn"
          class:active={$language === "en-US"}
          on:click={() => handleLanguageChange("en-US")}
        >
          <span class="lang-code-pill">EN</span>
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

    <!-- Danger Zone: Reset & Limpar Dados do Usuário -->
    <section class="settings-card glass-panel danger-card">
      <div class="card-header">
        <div class="card-icon red">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="3 6 5 6 21 6"></polyline>
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
            <line x1="10" y1="11" x2="10" y2="17"></line>
            <line x1="14" y1="11" x2="14" y2="17"></line>
          </svg>
        </div>
        <div>
          <h2>Limpar Todos os Dados do Usuário</h2>
          <p class="card-desc">Redefine o banco de dados do Indexo, removendo regras aprendidas, histórico, categorias criadas e configurações.</p>
        </div>
      </div>

      <div class="danger-action-row">
        <button class="danger-btn" on:click={() => { showClearModal = true; clearConfirmInput = ""; }}>
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
          </svg>
          Limpar Todos os Dados...
        </button>
      </div>
    </section>
  </div>
</div>

<!-- Modal: Confirmação de Limpeza Total -->
{#if showClearModal}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (showClearModal = false)}
    on:keydown={(e) => e.key === "Escape" && (showClearModal = false)}
  >
    <div class="modal-card">
      <div class="modal-header-danger">
        <div class="danger-badge-icon">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="#d14d41" stroke-width="2">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="8" x2="12" y2="12"></line>
            <line x1="12" y1="16" x2="12.01" y2="16"></line>
          </svg>
        </div>
        <h2>Limpar Todos os Dados do Perfil?</h2>
      </div>

      <p class="modal-warning-text">
        Esta ação apagará permanentemente:
      </p>

      <ul class="danger-points-list">
        <li>Todas as regras personalizadas e histórico de versões</li>
        <li>Todo o aprendizado e padrões acumulados pela IA</li>
        <li>Histórico de sessões de organização e logs de auditoria</li>
        <li>Categorias e tags customizadas criadas pelo usuário</li>
      </ul>

      <div class="confirmation-input-box">
        <label for="confirm-clear-text">
          Para confirmar a exclusão permanente, digite <strong>sim</strong> no campo abaixo:
        </label>
        <input
          id="confirm-clear-text"
          type="text"
          placeholder="Digite sim para confirmar"
          bind:value={clearConfirmInput}
          class="text-input"
          autocomplete="off"
        />
      </div>

      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showClearModal = false)}>Cancelar</button>
        <button
          class="danger-btn-solid"
          disabled={clearConfirmInput.trim().toLowerCase() !== "sim" || isClearing}
          on:click={handleExecuteClearAll}
        >
          {#if isClearing}
            Apagando...
          {:else}
            Confirmar e Apagar Tudo
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

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

  .lang-code-pill {
    background: var(--bg-tertiary);
    font-size: 0.72rem;
    font-weight: 800;
    padding: 0.1rem 0.35rem;
    border-radius: var(--radius-sm);
    color: var(--accent-primary);
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

  .theme-preview.light {
    background: #EFE5D3;
    border: 1px solid #D7C8AE;
  }
  .theme-preview.light .preview-bar {
    height: 12px;
    background: #BC5215;
  }
  .theme-preview.light .preview-body {
    flex: 1;
    background: #E4D8C2;
  }

  .theme-preview.dark {
    background: #100F0F;
    border: 1px solid #282726;
  }
  .theme-preview.dark .preview-bar {
    height: 12px;
    background: #DA702C;
  }
  .theme-preview.dark .preview-body {
    flex: 1;
    background: #1C1B1A;
  }

  .theme-preview.system {
    background: linear-gradient(135deg, #EFE5D3 50%, #100F0F 50%);
    border: 1px solid #343331;
  }
  .theme-preview.system .preview-bar {
    height: 12px;
    background: linear-gradient(90deg, #BC5215 50%, #DA702C 50%);
  }
  .theme-preview.system .preview-body {
    flex: 1;
    background: linear-gradient(135deg, #E4D8C2 50%, #1C1B1A 50%);
  }

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

  .danger-card {
    border: 1px solid rgba(209, 77, 65, 0.35);
    background: rgba(209, 77, 65, 0.04);
  }

  .card-icon.red {
    background: rgba(209, 77, 65, 0.15);
    color: #d14d41;
  }

  .danger-action-row {
    margin-top: 0.5rem;
  }

  .danger-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.65rem 1.25rem;
    font-size: 0.85rem;
    font-weight: 700;
    border-radius: var(--radius-md);
    background: rgba(209, 77, 65, 0.12);
    border: 1px solid rgba(209, 77, 65, 0.4);
    color: #d14d41;
    cursor: pointer;
    transition: all 150ms ease;
  }

  .danger-btn:hover {
    background: rgba(209, 77, 65, 0.22);
    border-color: #d14d41;
  }

  /* Danger Confirmation Modal */
  .modal-backdrop {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(16, 15, 15, 0.75);
    backdrop-filter: blur(4px);
    z-index: 3000;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
  }

  .modal-card {
    background: var(--bg-primary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-xl);
    width: 520px;
    max-width: 95vw;
    display: flex;
    flex-direction: column;
    padding: 1.5rem;
    gap: 1rem;
  }

  .modal-header-danger {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .modal-header-danger h2 {
    font-size: 1.15rem;
    font-weight: 700;
    color: #d14d41;
    margin: 0;
  }

  .danger-badge-icon {
    background: rgba(209, 77, 65, 0.15);
    padding: 0.4rem;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal-warning-text {
    font-size: 0.85rem;
    color: var(--text-primary);
    margin: 0;
    font-weight: 600;
  }

  .danger-points-list {
    margin: 0;
    padding-left: 1.25rem;
    font-size: 0.82rem;
    color: var(--text-muted);
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .confirmation-input-box {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    padding: 0.85rem;
    margin-top: 0.25rem;
  }

  .confirmation-input-box label {
    font-size: 0.82rem;
    color: var(--text-primary);
  }

  .text-input {
    background: var(--bg-primary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-sm);
    padding: 0.5rem 0.75rem;
    color: var(--text-primary);
    font-size: 0.88rem;
    outline: none;
  }

  .text-input:focus {
    border-color: #d14d41;
  }

  .modal-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: 0.5rem;
  }

  .danger-btn-solid {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    background: #d14d41;
    color: white;
    border: none;
    border-radius: var(--radius-md);
    padding: 0.55rem 1.1rem;
    font-weight: 700;
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 120ms ease;
  }

  .danger-btn-solid:hover:not(:disabled) {
    background: #b83a2f;
  }

  .danger-btn-solid:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(6px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
