<script lang="ts">
  import { onMount } from "svelte";
  import {
    listCustomRules,
    createCustomRule,
    updateCustomRule,
    deleteCustomRule,
    toggleCustomRule,
    listAllLearnedRules,
    deleteLearnedRule,
    type CustomRule,
    type CreateCustomRuleInput,
    type LearnedRuleInfo,
  } from "../lib/api";
  import { currentView, showToast } from "../lib/stores";

  type ActiveTab = "custom" | "builtin" | "learned";
  let activeTab: ActiveTab = "custom";

  let customRules: CustomRule[] = [];
  let learnedRules: LearnedRuleInfo[] = [];
  let isLoading: boolean = true;
  let searchQuery: string = "";

  // Modal State
  let showRuleModal: boolean = false;
  let editingRuleId: string | null = null;
  let ruleForm = {
    name: "",
    condition_field: "extension" as CustomRule["condition_field"],
    condition_operator: "equals" as CustomRule["condition_operator"],
    condition_value: "",
    action_type: "move_category" as CustomRule["action_type"],
    action_value: "",
  };

  // Built-in heuristics reference
  const builtinCategories = [
    {
      name: "Finanças e Fiscais",
      desc: "Boletos, faturas, extratos bancários, notas fiscais (DANFE), comprovantes PIX, recibos e declarações de IR.",
      exts: ["pdf", "xml", "ofx", "csv"],
      keywords: ["boleto", "fatura", "danfe", "pix", "comprovante", "recibo", "extrato", "declaracao", "irpf"],
    },
    {
      name: "Documentos e Textos",
      desc: "Contratos, relatórios, atas, planilhas, apresentações e documentações.",
      exts: ["pdf", "docx", "doc", "xlsx", "xls", "pptx", "txt", "md", "odt"],
      keywords: ["contrato", "relatorio", "termo", "proposta", "artigo", "edital", "manual"],
    },
    {
      name: "Imagens e Fotografias",
      desc: "Fotos de câmeras, ilustrações, capturas de tela e banners.",
      exts: ["png", "jpg", "jpeg", "webp", "gif", "svg", "bmp", "tiff", "raw"],
      keywords: ["foto", "screenshot", "wallpaper", "banner", "logo", "arte"],
    },
    {
      name: "Vídeos e Filmagens",
      desc: "Gravações, clipes, videoaulas e produções audiovisuais.",
      exts: ["mp4", "mkv", "avi", "mov", "webm", "flv", "wmv"],
      keywords: ["video", "gravacao", "aula", "clip", "podcast"],
    },
    {
      name: "Áudio e Música",
      desc: "Faixas musicais, podcasts, efeitos sonoros e gravações de voz.",
      exts: ["mp3", "wav", "flac", "aac", "ogg", "m4a"],
      keywords: ["musica", "audio", "faixa", "sample", "voz"],
    },
    {
      name: "Código e Desenvolvimento",
      desc: "Arquivos de código-fonte, scripts, bancos de dados e configurações de desenvolvimento.",
      exts: ["rs", "js", "ts", "py", "html", "css", "json", "sql", "bat", "ps1", "sh", "toml"],
      keywords: ["script", "server", "schema", "config", "build", "api"],
    },
    {
      name: "Arquivos Compactados",
      desc: "Pacotes compactados, instaladores e arquivos de backup.",
      exts: ["zip", "rar", "7z", "tar", "gz", "bz2", "iso"],
      keywords: ["backup", "archive", "pack", "dist"],
    },
  ];

  onMount(async () => {
    await reloadData();
  });

  async function reloadData() {
    isLoading = true;
    try {
      customRules = await listCustomRules();
      learnedRules = await listAllLearnedRules();
    } catch (e: any) {
      showToast("Erro ao carregar regras: " + e, "error");
    } finally {
      isLoading = false;
    }
  }

  function openCreateModal() {
    editingRuleId = null;
    ruleForm = {
      name: "",
      condition_field: "extension",
      condition_operator: "equals",
      condition_value: "",
      action_type: "move_category",
      action_value: "",
    };
    showRuleModal = true;
  }

  function openEditModal(rule: CustomRule) {
    editingRuleId = rule.id;
    ruleForm = {
      name: rule.name,
      condition_field: rule.condition_field,
      condition_operator: rule.condition_operator,
      condition_value: rule.condition_value,
      action_type: rule.action_type,
      action_value: rule.action_value,
    };
    showRuleModal = true;
  }

  async function handleSaveRule() {
    if (!ruleForm.name.trim() || !ruleForm.condition_value.trim() || !ruleForm.action_value.trim()) {
      showToast("Preencha todos os campos da regra.", "error");
      return;
    }

    try {
      if (editingRuleId) {
        const existing = customRules.find((r) => r.id === editingRuleId);
        if (existing) {
          await updateCustomRule({
            ...existing,
            name: ruleForm.name.trim(),
            condition_field: ruleForm.condition_field,
            condition_operator: ruleForm.condition_operator,
            condition_value: ruleForm.condition_value.trim(),
            action_type: ruleForm.action_type,
            action_value: ruleForm.action_value.trim(),
          });
          showToast("Regra personalizada atualizada com sucesso!", "success");
        }
      } else {
        await createCustomRule({
          name: ruleForm.name.trim(),
          condition_field: ruleForm.condition_field,
          condition_operator: ruleForm.condition_operator,
          condition_value: ruleForm.condition_value.trim(),
          action_type: ruleForm.action_type,
          action_value: ruleForm.action_value.trim(),
        });
        showToast("Nova regra personalizada criada!", "success");
      }
      showRuleModal = false;
      await reloadData();
    } catch (e: any) {
      showToast("Erro ao salvar regra: " + e, "error");
    }
  }

  async function handleToggleRule(rule: CustomRule) {
    try {
      await toggleCustomRule(rule.id, !rule.is_enabled);
      rule.is_enabled = !rule.is_enabled;
      customRules = [...customRules];
    } catch (e: any) {
      showToast("Erro ao alternar regra: " + e, "error");
    }
  }

  async function handleDeleteCustomRule(id: string) {
    try {
      await deleteCustomRule(id);
      showToast("Regra removida!", "success");
      await reloadData();
    } catch (e: any) {
      showToast("Erro ao remover regra: " + e, "error");
    }
  }

  async function handleDeleteLearnedRule(id: string) {
    try {
      await deleteLearnedRule(id);
      showToast("Regra aprendida removida!", "success");
      await reloadData();
    } catch (e: any) {
      showToast("Erro ao remover regra aprendida: " + e, "error");
    }
  }

  function getConditionFieldLabel(field: string): string {
    switch (field) {
      case "extension": return "Extensão";
      case "filename_contains": return "Nome contém";
      case "size_greater": return "Tamanho maior que (MB)";
      case "size_smaller": return "Tamanho menor que (MB)";
      default: return field;
    }
  }

  function getConditionOperatorLabel(op: string): string {
    switch (op) {
      case "equals": return "é igual a";
      case "contains": return "contém";
      case "starts_with": return "começa com";
      case "ends_with": return "termina com";
      default: return op;
    }
  }
</script>

<div class="rules-view">
  <!-- Header -->
  <div class="rules-header">
    <div class="header-titles">
      <div class="badge-title">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="3"></circle>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
        </svg>
        <h1>Gerenciador de Regras de Classificação</h1>
      </div>
      <p class="subtitle">
        Controle as regras personalizadas prioritárias, inspecione a inteligência padrão e gerencie o aprendizado automático da IA.
      </p>
    </div>

    <div class="header-actions">
      <button class="secondary-btn" on:click={() => currentView.set("settings")}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
        Voltar para Configurações
      </button>

      {#if activeTab === "custom"}
        <button class="primary-btn" on:click={openCreateModal}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
          </svg>
          Nova Regra Personalizada
        </button>
      {/if}
    </div>
  </div>

  <!-- Segmented Tabs Bar -->
  <div class="tabs-bar">
    <button
      class="tab-btn"
      class:active={activeTab === "custom"}
      on:click={() => (activeTab = "custom")}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon>
      </svg>
      <span>Regras Personalizadas</span>
      <span class="tab-badge">{customRules.length}</span>
    </button>

    <button
      class="tab-btn"
      class:active={activeTab === "builtin"}
      on:click={() => (activeTab = "builtin")}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path>
        <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path>
      </svg>
      <span>Regras Padrão (Heurísticas)</span>
      <span class="tab-badge">{builtinCategories.length}</span>
    </button>

    <button
      class="tab-btn"
      class:active={activeTab === "learned"}
      on:click={() => (activeTab = "learned")}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83"></path>
      </svg>
      <span>Regras Aprendidas pela IA</span>
      <span class="tab-badge">{learnedRules.length}</span>
    </button>
  </div>

  <!-- Tab Contents -->
  <div class="rules-content">
    {#if isLoading}
      <div class="loading-state">
        <div class="spinner"></div>
        <span>Carregando regras...</span>
      </div>

    <!-- 1. TAB: REGRAS PERSONALIZADAS -->
    {:else if activeTab === "custom"}
      {#if customRules.length === 0}
        <div class="empty-state">
          <div class="empty-icon">
            <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"></polygon>
            </svg>
          </div>
          <h3>Nenhuma regra personalizada criada</h3>
          <p>Crie regras condicionais prioritárias para direcionar arquivos específicos para suas pastas e categorias prediletas.</p>
          <button class="primary-btn" on:click={openCreateModal}>Criar Primeira Regra</button>
        </div>
      {:else}
        <div class="rules-grid">
          {#each customRules as rule (rule.id)}
            <div class="rule-card" class:is-disabled={!rule.is_enabled}>
              <div class="rule-card-header">
                <div class="rule-title-box">
                  <span class="rule-name">{rule.name}</span>
                  <span class="priority-pill">Prioridade {rule.priority}</span>
                </div>

                <div class="rule-header-actions">
                  <label class="toggle-switch" title={rule.is_enabled ? "Desativar regra" : "Ativar regra"}>
                    <input
                      type="checkbox"
                      checked={rule.is_enabled}
                      on:change={() => handleToggleRule(rule)}
                    />
                    <span class="slider"></span>
                  </label>
                  <button class="icon-btn" title="Editar Regra" on:click={() => openEditModal(rule)}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                      <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                    </svg>
                  </button>
                  <button class="icon-btn text-danger" title="Excluir Regra" on:click={() => handleDeleteCustomRule(rule.id)}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <polyline points="3 6 5 6 21 6"></polyline>
                      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                    </svg>
                  </button>
                </div>
              </div>

              <!-- Rule Condition Pipeline -->
              <div class="rule-pipeline-box">
                <div class="condition-pill">
                  <span class="keyword-tag">SE</span>
                  <span class="field-txt">[{getConditionFieldLabel(rule.condition_field)}]</span>
                  <span class="operator-txt">{getConditionOperatorLabel(rule.condition_operator)}</span>
                  <span class="value-highlight">"{rule.condition_value}"</span>
                </div>

                <div class="arrow-down">➔</div>

                <div class="action-pill">
                  <span class="keyword-tag action">ENTÃO</span>
                  <span class="field-txt">
                    {rule.action_type === "move_category" ? "Mover para Categoria" : "Aplicar Tag"}
                  </span>
                  <span class="action-target">"{rule.action_value}"</span>
                </div>
              </div>
            </div>
          {/each}
        </div>
      {/if}

    <!-- 2. TAB: REGRAS PADRÃO -->
    {:else if activeTab === "builtin"}
      <div class="builtin-grid">
        {#each builtinCategories as cat}
          <div class="builtin-card">
            <div class="builtin-header">
              <span class="builtin-name">{cat.name}</span>
            </div>
            <p class="builtin-desc">{cat.desc}</p>

            <div class="builtin-tags-section">
              <span class="section-sub">Extensões Detectadas:</span>
              <div class="tags-row">
                {#each cat.exts as ext}
                  <span class="ext-pill">.{ext}</span>
                {/each}
              </div>
            </div>

            <div class="builtin-tags-section">
              <span class="section-sub">Palavras-Chave de Reconhecimento:</span>
              <div class="tags-row">
                {#each cat.keywords as kw}
                  <span class="kw-pill">{kw}</span>
                {/each}
              </div>
            </div>
          </div>
        {/each}
      </div>

    <!-- 3. TAB: REGRAS APRENDIDAS PELA IA -->
    {:else if activeTab === "learned"}
      {#if learnedRules.length === 0}
        <div class="empty-state">
          <div class="empty-icon">
            <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83"></path>
            </svg>
          </div>
          <h3>Nenhuma regra aprendida ainda</h3>
          <p>Quando você reatribuir ou arrastar arquivos para novas categorias no Preview, o Indexo aprenderá automaticamente seus padrões e os listará aqui.</p>
        </div>
      {:else}
        <div class="learned-table-container">
          <table class="learned-table">
            <thead>
              <tr>
                <th>Padrão Aprendido</th>
                <th>Tipo de Reconhecimento</th>
                <th>Categoria Associada</th>
                <th>Confiança</th>
                <th>Usos (Hits)</th>
                <th>Ações</th>
              </tr>
            </thead>
            <tbody>
              {#each learnedRules as r}
                <tr>
                  <td class="col-pattern"><span class="pattern-mono">"{r.pattern_value}"</span></td>
                  <td class="col-type"><span class="type-pill">{r.pattern_type}</span></td>
                  <td class="col-category">
                    <span class="cat-pill" style="border-left: 3px solid {r.category_color || 'var(--accent-primary)'};">
                      {r.category_name}
                    </span>
                  </td>
                  <td class="col-weight">{Math.round(r.confidence_weight * 100)}%</td>
                  <td class="col-hits">{r.hit_count}</td>
                  <td class="col-actions">
                    <button class="icon-btn text-danger" title="Excluir regra aprendida" on:click={() => handleDeleteLearnedRule(r.id)}>
                      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="3 6 5 6 21 6"></polyline>
                        <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                      </svg>
                    </button>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {/if}
    {/if}
  </div>
</div>

<!-- Modal: Criar / Editar Regra Personalizada -->
{#if showRuleModal}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (showRuleModal = false)}
    on:keydown={(e) => e.key === "Escape" && (showRuleModal = false)}
  >
    <div class="modal-card">
      <div class="modal-header-row">
        <h2>{editingRuleId ? "Editar Regra Personalizada" : "Nova Regra Personalizada"}</h2>
        <button class="close-btn" on:click={() => (showRuleModal = false)}>✕</button>
      </div>

      <div class="modal-body-form">
        <label class="form-field">
          <span class="field-label">Nome da Regra:</span>
          <input
            type="text"
            placeholder="Ex: Documentos Financeiros PDF ou Fotos RAW"
            bind:value={ruleForm.name}
            class="text-input"
          />
        </label>

        <div class="form-group-box">
          <span class="group-title">Condição (SE):</span>

          <div class="form-row">
            <label class="form-field">
              <span class="field-label">Campo:</span>
              <select bind:value={ruleForm.condition_field} class="select-input">
                <option value="extension">Extensão do Arquivo</option>
                <option value="filename_contains">Nome Contém</option>
                <option value="size_greater">Tamanho Maior que (MB)</option>
                <option value="size_smaller">Tamanho Menor que (MB)</option>
              </select>
            </label>

            <label class="form-field">
              <span class="field-label">Operador:</span>
              <select bind:value={ruleForm.condition_operator} class="select-input">
                <option value="equals">É igual a</option>
                <option value="contains">Contém</option>
                <option value="starts_with">Começa com</option>
                <option value="ends_with">Termina com</option>
              </select>
            </label>
          </div>

          <label class="form-field">
            <span class="field-label">Valor da Condição:</span>
            <input
              type="text"
              placeholder="Ex: pdf, contrato, 50, etc."
              bind:value={ruleForm.condition_value}
              class="text-input"
            />
          </label>
        </div>

        <div class="form-group-box">
          <span class="group-title">Ação (ENTÃO):</span>

          <div class="form-row">
            <label class="form-field">
              <span class="field-label">Tipo de Ação:</span>
              <select bind:value={ruleForm.action_type} class="select-input">
                <option value="move_category">Mover para Categoria</option>
                <option value="apply_tag">Aplicar Tag</option>
              </select>
            </label>

            <label class="form-field">
              <span class="field-label">Nome de Destino:</span>
              <input
                type="text"
                placeholder="Ex: Finanças / Contratos"
                bind:value={ruleForm.action_value}
                class="text-input"
              />
            </label>
          </div>
        </div>
      </div>

      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showRuleModal = false)}>Cancelar</button>
        <button class="primary-btn" on:click={handleSaveRule}>
          {editingRuleId ? "Salvar Alterações" : "Criar Regra"}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .rules-view {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 1.5rem 2rem;
    gap: 1.25rem;
    overflow: hidden;
    min-height: 0;
    animation: fadeIn 200ms ease-out;
  }

  .rules-header {
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

  .badge-title {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    color: var(--accent-primary);
  }

  .badge-title h1 {
    font-size: 1.45rem;
    font-weight: 800;
    color: var(--text-primary);
    margin: 0;
    letter-spacing: -0.02em;
  }

  .subtitle {
    font-size: 0.86rem;
    color: var(--text-muted);
    margin: 0;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .tabs-bar {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    border-bottom: 1px solid var(--border-subtle);
    padding-bottom: 0.25rem;
  }

  .tab-btn {
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--radius-md);
    padding: 0.5rem 0.85rem;
    font-size: 0.84rem;
    font-weight: 600;
    color: var(--text-muted);
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 0.45rem;
    transition: all 120ms ease;
  }

  .tab-btn:hover {
    background: var(--bg-secondary);
    color: var(--text-primary);
  }

  .tab-btn.active {
    background: var(--bg-secondary);
    border-color: var(--border-medium);
    color: var(--text-primary);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.06);
  }

  .tab-badge {
    background: var(--bg-tertiary);
    border-radius: var(--radius-full);
    padding: 0.05rem 0.45rem;
    font-size: 0.72rem;
    font-weight: 700;
  }

  .rules-content {
    flex: 1;
    overflow-y: auto;
    padding-right: 0.35rem;
    scrollbar-width: thin;
  }

  .loading-state, .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 4rem 2rem;
    gap: 1rem;
    color: var(--text-muted);
  }

  .empty-state {
    background: var(--bg-secondary);
    border: 1px dashed var(--border-medium);
    border-radius: var(--radius-xl);
  }

  .empty-icon {
    background: var(--bg-tertiary);
    padding: 1.25rem;
    border-radius: 50%;
  }

  .rules-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
    gap: 1rem;
  }

  .rule-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    transition: all 150ms ease;
  }

  .rule-card.is-disabled {
    opacity: 0.5;
    background: var(--bg-tertiary);
  }

  .rule-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .rule-title-box {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .rule-name {
    font-weight: 700;
    font-size: 0.92rem;
    color: var(--text-primary);
  }

  .priority-pill {
    font-size: 0.68rem;
    font-weight: 700;
    background: var(--bg-tertiary);
    color: var(--text-muted);
    padding: 0.1rem 0.4rem;
    border-radius: var(--radius-full);
  }

  .rule-header-actions {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .rule-pipeline-box {
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 0.65rem 0.85rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.8rem;
  }

  .condition-pill, .action-pill {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    flex-wrap: wrap;
  }

  .keyword-tag {
    background: rgba(59, 130, 246, 0.15);
    color: #3b82f6;
    font-weight: 800;
    font-size: 0.7rem;
    padding: 0.1rem 0.35rem;
    border-radius: var(--radius-sm);
  }

  .keyword-tag.action {
    background: rgba(16, 185, 129, 0.15);
    color: #10b981;
  }

  .field-txt {
    color: var(--text-muted);
    font-weight: 600;
  }

  .value-highlight {
    font-family: var(--font-mono);
    color: #8b5cf6;
    font-weight: 700;
  }

  .action-target {
    font-family: var(--font-mono);
    color: #10b981;
    font-weight: 700;
  }

  .arrow-down {
    color: var(--text-muted);
    font-size: 0.75rem;
    margin-left: 0.5rem;
  }

  /* Built-in Cards */
  .builtin-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
    gap: 1rem;
  }

  .builtin-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
  }

  .builtin-name {
    font-weight: 800;
    font-size: 0.92rem;
    color: var(--text-primary);
  }

  .builtin-desc {
    font-size: 0.8rem;
    color: var(--text-muted);
    margin: 0;
    line-height: 1.4;
  }

  .builtin-tags-section {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .section-sub {
    font-size: 0.72rem;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }

  .tags-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
  }

  .ext-pill {
    background: var(--bg-primary);
    border: 1px solid var(--border-medium);
    padding: 0.1rem 0.4rem;
    border-radius: var(--radius-sm);
    font-family: var(--font-mono);
    font-size: 0.72rem;
    color: var(--accent-primary);
    font-weight: 700;
  }

  .kw-pill {
    background: var(--bg-tertiary);
    padding: 0.1rem 0.4rem;
    border-radius: var(--radius-sm);
    font-size: 0.72rem;
    color: var(--text-muted);
  }

  /* Learned Table */
  .learned-table-container {
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    overflow-x: auto;
    background: var(--bg-secondary);
  }

  .learned-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.8rem;
    text-align: left;
  }

  .learned-table th {
    background: rgba(0, 0, 0, 0.08);
    padding: 0.55rem 0.75rem;
    color: var(--text-muted);
    font-size: 0.74rem;
    text-transform: uppercase;
    border-bottom: 1px solid var(--border-subtle);
  }

  .learned-table td {
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border-subtle);
  }

  .pattern-mono {
    font-family: var(--font-mono);
    font-weight: 700;
    color: var(--text-primary);
  }

  .type-pill {
    font-size: 0.7rem;
    font-weight: 700;
    background: var(--bg-tertiary);
    color: var(--text-muted);
    padding: 0.1rem 0.4rem;
    border-radius: var(--radius-full);
  }

  .cat-pill {
    padding-left: 0.45rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  /* Toggle Switch */
  .toggle-switch {
    position: relative;
    display: inline-block;
    width: 32px;
    height: 18px;
  }

  .toggle-switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider {
    position: absolute;
    cursor: pointer;
    inset: 0;
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border-medium);
    transition: 150ms;
    border-radius: 34px;
  }

  .slider:before {
    position: absolute;
    content: "";
    height: 12px;
    width: 12px;
    left: 2px;
    bottom: 2px;
    background-color: var(--text-muted);
    transition: 150ms;
    border-radius: 50%;
  }

  input:checked + .slider {
    background-color: var(--accent-primary);
    border-color: var(--accent-primary);
  }

  input:checked + .slider:before {
    transform: translateX(14px);
    background-color: white;
  }

  .icon-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 0.25rem;
    border-radius: var(--radius-sm);
  }

  .icon-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .text-danger:hover {
    color: #ef4444 !important;
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
    max-width: 520px;
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    box-shadow: var(--shadow-xl);
  }

  .modal-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .modal-header-row h2 {
    font-size: 1.15rem;
    margin: 0;
    color: var(--text-primary);
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 1rem;
    cursor: pointer;
  }

  .modal-body-form {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  .form-group-box {
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
  }

  .group-title {
    font-size: 0.76rem;
    font-weight: 700;
    color: var(--accent-primary);
    text-transform: uppercase;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.5rem;
  }

  .form-field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .field-label {
    font-size: 0.76rem;
    color: var(--text-muted);
    font-weight: 600;
  }

  .text-input, .select-input {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-sm);
    padding: 0.4rem 0.65rem;
    font-size: 0.82rem;
    color: var(--text-primary);
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: 0.5rem;
  }

  .secondary-btn {
    background: var(--bg-tertiary);
    border: 1px solid var(--border-medium);
    color: var(--text-primary);
    padding: 0.45rem 0.85rem;
    border-radius: var(--radius-md);
    font-size: 0.82rem;
    font-weight: 600;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
  }

  .primary-btn {
    background: var(--accent-primary);
    color: white;
    border: none;
    padding: 0.45rem 1rem;
    border-radius: var(--radius-md);
    font-size: 0.82rem;
    font-weight: 600;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid rgba(59, 130, 246, 0.2);
    border-top-color: var(--accent-primary);
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
