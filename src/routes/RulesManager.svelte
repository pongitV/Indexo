<script lang="ts">
  import { onMount } from "svelte";
  import {
    listCustomRules,
    createCustomRule,
    updateCustomRule,
    deleteCustomRule,
    toggleCustomRule,
    restoreCustomRuleOriginal,
    getCustomRuleHistory,
    getBuiltinRulesConfig,
    saveBuiltinRuleConfig,
    resetBuiltinRuleConfig,
    resetAllBuiltinRulesConfig,
    listAllLearnedRules,
    deleteLearnedRule,
    createCategory,
    type CustomRule,
    type CreateCustomRuleInput,
    type CustomRuleHistoryRecord,
    type BuiltinCategoryConfig,
    type LearnedRuleInfo,
  } from "../lib/api";
  import { currentView, showToast, aiFolderSuggestions, type FolderSuggestion } from "../lib/stores";

  type ActiveTab = "builtin" | "custom" | "global-folders" | "suggestions" | "learned";
  let activeTab: ActiveTab = "builtin";

  let customRules: CustomRule[] = [];
  let builtinRulesList: BuiltinCategoryConfig[] = [];
  let learnedRules: LearnedRuleInfo[] = [];
  let isLoading: boolean = true;

  // Modal State para Regras Customizadas
  let showRuleModal: boolean = false;
  let editingRuleId: string | null = null;
  let ruleForm = {
    name: "",
    condition_field: "extension" as CustomRule["condition_field"],
    condition_operator: "equals" as CustomRule["condition_operator"],
    condition_value: "",
    action_type: "move_category" as CustomRule["action_type"],
    action_value: "",
    subfolder_behavior: "auto" as CustomRule["subfolder_behavior"],
    priority: 50,
  };

  // Modal State para Histórico da Regra Customizada
  let showHistoryModal: boolean = false;
  let selectedRuleForHistory: CustomRule | null = null;
  let ruleHistoryList: CustomRuleHistoryRecord[] = [];
  let isLoadingHistory: boolean = false;

  // Modal State para Edição de Heurística / Regra Padrão
  let showBuiltinEditModal: boolean = false;
  let editingBuiltin: BuiltinCategoryConfig | null = null;
  let newExtInput: string = "";
  let newKwInput: string = "";
  let newSubInput: string = "";

  // Modal State para Nova Pasta Global / Subpasta Customizada
  let showFolderModal: boolean = false;
  let folderForm = {
    parentGlobal: "Executaveis",
    name: "",
    desc: "",
  };

  // Definição de Pastas Globais
  interface GlobalFolderDef {
    id: string;
    name: string;
    desc: string;
    subfolders: string[];
    is_default: boolean;
    color: string;
  }

  const defaultGlobalFolders: GlobalFolderDef[] = [
    {
      id: "Executaveis",
      name: "Executaveis",
      desc: "Jogos instalados, aplicativos, navegadores, IDEs e instaladores",
      subfolders: [
        "Jogos-Steam",
        "Jogos-Epicgames",
        "Jogos-Indies-Portateis",
        "Jogos-Emuladores-ROMs",
        "Aplicativos-Navegadores",
        "Aplicativos-IDEs",
        "Aplicativos-Design-Edicao",
        "Aplicativos-Utilitarios",
        "Instaladores-Setups",
        "Instaladores-Drivers",
        "Instaladores-ISOs",
      ],
      is_default: true,
      color: "#d14d41",
    },
    {
      id: "Media",
      name: "Media",
      desc: "Fotografias, wallpapers, capturas, vídeos, gravações e faixas de áudio",
      subfolders: [
        "Imagens-Fotografias",
        "Videos-Gravacoes",
        "Audios-Musicas",
      ],
      is_default: true,
      color: "#24837b",
    },
    {
      id: "Documentos",
      name: "Documentos",
      desc: "Classificação por conteúdo: fiscais/pessoais, relatórios de trabalho e estudos",
      subfolders: [
        "Fiscais-Pessoais",
        "Trabalho",
        "Estudos",
      ],
      is_default: true,
      color: "#da702c",
    },
    {
      id: "Projetos",
      name: "Projetos",
      desc: "Repositórios Git GitHub e Locais, modelos 3D/CAD e scripts de automação",
      subfolders: [
        "Repositorios-GitHub",
        "Repositorios-Locais",
        "Modelos-3D-CAD",
        "Scripts-Automacoes",
      ],
      is_default: true,
      color: "#8b7ec8",
    },
    {
      id: "Compactados-Backups",
      name: "Compactados-Backups",
      desc: "Arquivos compactados (.zip, .rar, .7z) e backups de segurança",
      subfolders: [
        "Backups",
        "Arquivos-ZIP",
      ],
      is_default: true,
      color: "#4385be",
    },
    {
      id: "Fontes-Tipografia",
      name: "Fontes-Tipografia",
      desc: "Arquivos de fontes (.ttf, .otf, .woff) e famílias tipográficas",
      subfolders: [
        "Fontes-Principais",
        "Icones-Fontes",
      ],
      is_default: true,
      color: "#879a39",
    },
  ];

  let userCustomGlobalFolders: GlobalFolderDef[] = [];

  onMount(async () => {
    await reloadData();
  });

  async function reloadData() {
    isLoading = true;
    try {
      customRules = await listCustomRules();
      builtinRulesList = await getBuiltinRulesConfig();
      learnedRules = await listAllLearnedRules();
    } catch (e: any) {
      showToast("Erro ao carregar regras: " + e, "error");
    } finally {
      isLoading = false;
    }
  }

  // ==========================================
  // REGRAS PERSONALIZADAS
  // ==========================================

  function openCreateModal() {
    editingRuleId = null;
    ruleForm = {
      name: "",
      condition_field: "extension",
      condition_operator: "equals",
      condition_value: "",
      action_type: "move_category",
      action_value: "",
      subfolder_behavior: "auto",
      priority: 50,
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
      subfolder_behavior: rule.subfolder_behavior || "auto",
      priority: rule.priority,
    };
    showRuleModal = true;
  }

  async function handleSaveRule() {
    if (!ruleForm.name.trim() || !ruleForm.condition_value.trim() || !ruleForm.action_value.trim()) {
      showToast("Preencha todos os campos obrigatórios da regra.", "error");
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
            action_value: ruleForm.action_value.trim().replace(/\s+/g, "-"),
            subfolder_behavior: ruleForm.subfolder_behavior,
            priority: ruleForm.priority,
          });
          showToast("Regra personalizada atualizada com nova versão salva no histórico!", "success");
        }
      } else {
        await createCustomRule({
          name: ruleForm.name.trim(),
          condition_field: ruleForm.condition_field,
          condition_operator: ruleForm.condition_operator,
          condition_value: ruleForm.condition_value.trim(),
          action_type: ruleForm.action_type,
          action_value: ruleForm.action_value.trim().replace(/\s+/g, "-"),
          subfolder_behavior: ruleForm.subfolder_behavior,
          priority: ruleForm.priority,
        });
        showToast("Regra personalizada criada com sucesso!", "success");
      }
      showRuleModal = false;
      await reloadData();
    } catch (e: any) {
      showToast("Erro ao salvar regra: " + e, "error");
    }
  }

  async function handleOpenHistory(rule: CustomRule) {
    selectedRuleForHistory = rule;
    isLoadingHistory = true;
    showHistoryModal = true;
    try {
      ruleHistoryList = await getCustomRuleHistory(rule.id);
    } catch (e: any) {
      showToast("Erro ao carregar histórico da regra: " + e, "error");
    } finally {
      isLoadingHistory = false;
    }
  }

  async function handleRestoreRuleOriginal(rule: CustomRule) {
    if (!confirm(`Deseja restaurar a regra '${rule.name}' para a sua configuração inicial original?`)) {
      return;
    }

    try {
      await restoreCustomRuleOriginal(rule.id);
      showToast(`Regra '${rule.name}' restaurada para a versão original com sucesso!`, "success");
      if (showHistoryModal && selectedRuleForHistory?.id === rule.id) {
        ruleHistoryList = await getCustomRuleHistory(rule.id);
      }
      await reloadData();
    } catch (e: any) {
      showToast("Erro ao restaurar regra original: " + e, "error");
    }
  }

  async function handleToggleRule(rule: CustomRule) {
    try {
      await toggleCustomRule(rule.id, !rule.is_enabled);
      rule.is_enabled = !rule.is_enabled;
      customRules = [...customRules];
      showToast(rule.is_enabled ? "Regra ativada!" : "Regra desativada!", "info");
    } catch (e: any) {
      showToast("Erro ao alternar regra: " + e, "error");
    }
  }

  async function handleDeleteCustomRule(id: string) {
    try {
      await deleteCustomRule(id);
      customRules = customRules.filter((r) => r.id !== id);
      showToast("Regra personalizada excluída.", "success");
    } catch (e: any) {
      showToast("Erro ao excluir regra: " + e, "error");
    }
  }

  const factoryDefaultsMap: Record<string, BuiltinCategoryConfig> = {
    "media-images": {
      id: "media-images",
      group_name: "Media",
      display_name: "Imagens e Fotografias",
      target_path: "Media/Imagens-Fotografias",
      description: "Fotografias, capturas de tela, wallpapers, artes vetoriais e gráficos",
      extensions: ["jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "tiff", "ico", "raw", "heic", "psd", "ai", "eps", "xcf", "cr2", "nef", "arw"],
      keywords: ["screenshot", "wallpaper", "foto", "foto_", "img_", "captura", "arte", "banner"],
      subfolders: ["Fotografias", "Wallpapers", "Capturas-Tela", "Design-Vetores"],
      subfolder_behavior: "auto",
      is_enabled: true,
      is_customized: false,
    },
    "media-videos": {
      id: "media-videos",
      group_name: "Media",
      display_name: "Vídeos e Gravações",
      target_path: "Media/Videos-Gravacoes",
      description: "Filmes, gravações de tela, clipes de jogos e tutoriais",
      extensions: ["mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "3gp", "ts", "mpg", "mpeg"],
      keywords: ["video", "gravacao", "clip", "replay", "screen", "gameplay", "filme"],
      subfolders: ["Gravacoes-Tela", "Clipes-Jogos", "Filmes-Series", "Tutoriais"],
      subfolder_behavior: "auto",
      is_enabled: true,
      is_customized: false,
    },
    "media-audio": {
      id: "media-audio",
      group_name: "Media",
      display_name: "Áudios e Músicas",
      target_path: "Media/Audios-Musicas",
      description: "Músicas, podcasts, efeitos sonoros (SFX) e gravações de voz",
      extensions: ["mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "mid", "midi", "opus", "aiff", "alac"],
      keywords: ["musica", "audio", "podcast", "sfx", "sound", "track", "faixa", "voz"],
      subfolders: ["Albuns-Artistas", "Podcasts", "Efeitos-Sonoros-SFX", "Gravacoes-Voz"],
      subfolder_behavior: "auto",
      is_enabled: true,
      is_customized: false,
    },
    "exec-roms": {
      id: "exec-roms",
      group_name: "Executaveis",
      display_name: "Jogos, Emuladores e ROMs",
      target_path: "Executaveis/Jogos-Emuladores-ROMs",
      description: "ROMs e imagens de console separadas por plataforma",
      extensions: ["nes", "sfc", "smc", "gba", "gbc", "gb", "nds", "3ds", "cia", "n64", "z64", "v64", "nsp", "xci", "rvz", "wbfs", "wad", "gcz", "pbp", "cso", "gen", "smd", "cdi", "gdi", "chd"],
      keywords: ["rom", "emulator", "emulador", "save", "bios", "patch"],
      subfolders: ["Nintendo-NES", "Super-Nintendo-SNES", "Game-Boy-Advance-GBA", "Game-Boy-Color-GBC", "Nintendo-DS", "Nintendo-3DS", "Nintendo-64", "Nintendo-Switch", "Nintendo-Wii-GameCube", "PlayStation-PSP", "PlayStation-1", "PlayStation-2", "PlayStation-3", "Sega-MegaDrive", "Sega-Dreamcast", "Sega-Saturn"],
      subfolder_behavior: "auto",
      is_enabled: true,
      is_customized: false,
    },
    "exec-apps": {
      id: "exec-apps",
      group_name: "Executaveis",
      display_name: "Aplicativos e Softwares",
      target_path: "Executaveis/Aplicativos-Utilitarios",
      description: "Aplicativos instalados, navegadores, IDEs e utilitários",
      extensions: ["exe", "msi", "appimage", "dmg", "pkg", "deb", "rpm", "apk"],
      keywords: ["chrome", "firefox", "edge", "vscode", "idea", "photoshop", "figma", "driver"],
      subfolders: ["Aplicativos-Navegadores", "Aplicativos-IDEs", "Aplicativos-Design-Edicao", "Aplicativos-Utilitarios"],
      subfolder_behavior: "auto",
      is_enabled: true,
      is_customized: false,
    },
    "exec-installers": {
      id: "exec-installers",
      group_name: "Executaveis",
      display_name: "Instaladores e Imagens de Disco",
      target_path: "Executaveis/Instaladores-Setups",
      description: "Setups de instalação, drivers de hardware e arquivos ISO",
      extensions: ["exe", "msi", "iso", "img", "vhd", "vhdx", "bin", "cue"],
      keywords: ["setup", "installer", "instalador", "driver", "geforce", "realtek"],
      subfolders: ["Instaladores-Setups", "Instaladores-Drivers", "Instaladores-ISOs"],
      subfolder_behavior: "auto",
      is_enabled: true,
      is_customized: false,
    },
    "docs-finance": {
      id: "docs-finance",
      group_name: "Documentos",
      display_name: "Documentos Fiscais e Pessoais",
      target_path: "Documentos/Fiscais-Pessoais",
      description: "Boletos, faturas, comprovantes, recibos, DANFE, notas fiscais e contratos",
      extensions: ["pdf", "xml", "docx", "doc", "xlsx", "xls", "txt"],
      keywords: ["boleto", "fatura", "recibo", "comprovante", "danfe", "nota_fiscal", "contrato", "declaracao", "imposto"],
      subfolders: ["Boletos-Faturas", "Comprovantes-Pagamento", "Notas-Fiscais-DANFE", "Contratos-Declaracoes"],
      subfolder_behavior: "by_year",
      is_enabled: true,
      is_customized: false,
    },
    "docs-work": {
      id: "docs-work",
      group_name: "Documentos",
      display_name: "Relatórios e Trabalho",
      target_path: "Documentos/Trabalho",
      description: "Apresentações, planilhas corporativas, relatórios de reuniões e planejamentos",
      extensions: ["pdf", "docx", "doc", "xlsx", "xls", "pptx", "ppt", "csv", "txt", "md"],
      keywords: ["relatorio", "report", "reuniao", "projeto", "apresentacao", "slides", "cronograma", "orcamento", "proposta"],
      subfolders: ["Relatorios", "Planilhas-Orcamentos", "Apresentacoes", "Reunioes-Atas"],
      subfolder_behavior: "by_year",
      is_enabled: true,
      is_customized: false,
    },
    "docs-study": {
      id: "docs-study",
      group_name: "Documentos",
      display_name: "Estudos e Acadêmico",
      target_path: "Documentos/Estudos",
      description: "Livros digitais, apostilas, artigos acadêmicos, TCC, teses e resumos",
      extensions: ["pdf", "epub", "mobi", "azw3", "cbr", "cbz", "docx", "txt", "md"],
      keywords: ["aula", "exercicio", "tcc", "artigo", "tese", "monografia", "livro", "apostila", "resumo", "curso"],
      subfolders: ["Livros-Ebooks", "Artigos-Teses", "Aulas-Apostilas", "Cursos-Certificados"],
      subfolder_behavior: "auto",
      is_enabled: true,
      is_customized: false,
    },
    "proj-repos": {
      id: "proj-repos",
      group_name: "Projetos",
      display_name: "Repositórios e Código-Fonte",
      target_path: "Projetos/Repositorios-Locais",
      description: "Repositórios de código, scripts de automação e arquivos de desenvolvimento",
      extensions: ["rs", "js", "ts", "py", "html", "css", "cpp", "c", "h", "java", "go", "php", "sh", "bat", "ps1", "sql", "json", "yaml", "toml"],
      keywords: ["github", "repo", "script", "automacao", "api", "backend", "frontend"],
      subfolders: ["Repositorios-GitHub", "Repositorios-Locais", "Scripts-Automacoes"],
      subfolder_behavior: "auto",
      is_enabled: true,
      is_customized: false,
    },
    "proj-3d": {
      id: "proj-3d",
      group_name: "Projetos",
      display_name: "Modelos 3D e CAD",
      target_path: "Projetos/Modelos-3D-CAD",
      description: "Cenas do Blender, modelos para impressão 3D e projetos de engenharia",
      extensions: ["blend", "stl", "step", "obj", "fbx", "dae", "3ds", "iges", "dwg", "dxf", "blend1", "skp", "ply"],
      keywords: ["model", "render", "print3d", "cad", "blender", "peca", "malha"],
      subfolders: ["Projetos-Blender", "Impressao-3D", "Projetos-CAD-DWG"],
      subfolder_behavior: "auto",
      is_enabled: true,
      is_customized: false,
    },
    "archives-backups": {
      id: "archives-backups",
      group_name: "Compactados-Backups",
      display_name: "Compactados e Backups",
      target_path: "Compactados-Backups",
      description: "Pacotes ZIP, RAR, 7Z e cópias de segurança",
      extensions: ["zip", "rar", "7z", "tar", "gz", "bz2", "xz", "tgz", "cab", "lz", "zst", "bak"],
      keywords: ["backup", "archive", "pack", "dist", "bkp"],
      subfolders: ["Backups", "Arquivos-ZIP", "Arquivos-RAR", "Arquivos-7Z"],
      subfolder_behavior: "auto",
      is_enabled: true,
      is_customized: false,
    },
    "fonts-typography": {
      id: "fonts-typography",
      group_name: "Fontes-Tipografia",
      display_name: "Fontes e Tipografia",
      target_path: "Fontes-Tipografia",
      description: "Fontes TrueType, OpenType, WebFonts e ícones tipográficos",
      extensions: ["ttf", "otf", "woff", "woff2", "eot", "fon"],
      keywords: ["font", "type", "sans", "serif", "mono", "regular", "bold"],
      subfolders: ["Fontes-Principais", "Icones-Fontes", "Web-Fonts"],
      subfolder_behavior: "auto",
      is_enabled: true,
      is_customized: false,
    },
  };

  function restoreFieldExtensions() {
    if (!editingBuiltin) return;
    const def = factoryDefaultsMap[editingBuiltin.id];
    if (def) {
      editingBuiltin.extensions = [...def.extensions];
      editingBuiltin = { ...editingBuiltin };
      showToast("Extensões restauradas para o padrão de fábrica!", "info");
    }
  }

  function restoreFieldSubfolders() {
    if (!editingBuiltin) return;
    const def = factoryDefaultsMap[editingBuiltin.id];
    if (def) {
      editingBuiltin.subfolders = [...def.subfolders];
      editingBuiltin.subfolder_behavior = def.subfolder_behavior;
      editingBuiltin = { ...editingBuiltin };
      showToast("Subpastas e agrupamento restaurados para o padrão de fábrica!", "info");
    }
  }

  function restoreFieldKeywords() {
    if (!editingBuiltin) return;
    const def = factoryDefaultsMap[editingBuiltin.id];
    if (def) {
      editingBuiltin.keywords = [...def.keywords];
      editingBuiltin = { ...editingBuiltin };
      showToast("Palavras-chave restauradas para o padrão de fábrica!", "info");
    }
  }

  function restoreFieldDestination() {
    if (!editingBuiltin) return;
    const def = factoryDefaultsMap[editingBuiltin.id];
    if (def) {
      editingBuiltin.display_name = def.display_name;
      editingBuiltin.target_path = def.target_path;
      editingBuiltin.description = def.description;
      editingBuiltin = { ...editingBuiltin };
      showToast("Destino e descrição restaurados para o padrão de fábrica!", "info");
    }
  }

  function restoreAllFieldsForEditingBuiltin() {
    if (!editingBuiltin) return;
    const def = factoryDefaultsMap[editingBuiltin.id];
    if (def) {
      editingBuiltin = JSON.parse(JSON.stringify(def));
      showToast(`Categoria '${def.display_name}' restaurada para os valores padrão de fábrica!`, "info");
    }
  }

  function openEditBuiltinModal(item: BuiltinCategoryConfig) {
    editingBuiltin = JSON.parse(JSON.stringify(item));
    newExtInput = "";
    newKwInput = "";
    newSubInput = "";
    showBuiltinEditModal = true;
  }

  function addExtension() {
    if (!editingBuiltin || !newExtInput.trim()) return;
    const clean = newExtInput.trim().replace(/^\./, "").toLowerCase();
    if (clean && !editingBuiltin.extensions.includes(clean)) {
      editingBuiltin.extensions = [...editingBuiltin.extensions, clean];
      newExtInput = "";
    }
  }

  function removeExtension(ext: string) {
    if (!editingBuiltin) return;
    editingBuiltin.extensions = editingBuiltin.extensions.filter((e) => e !== ext);
  }

  function addKeyword() {
    if (!editingBuiltin || !newKwInput.trim()) return;
    const clean = newKwInput.trim().toLowerCase();
    if (clean && !editingBuiltin.keywords.includes(clean)) {
      editingBuiltin.keywords = [...editingBuiltin.keywords, clean];
      newKwInput = "";
    }
  }

  function removeKeyword(kw: string) {
    if (!editingBuiltin) return;
    editingBuiltin.keywords = editingBuiltin.keywords.filter((k) => k !== kw);
  }

  function addSubfolder() {
    if (!editingBuiltin || !newSubInput.trim()) return;
    const clean = newSubInput.trim().replace(/\s+/g, "-");
    if (clean && !editingBuiltin.subfolders.includes(clean)) {
      editingBuiltin.subfolders = [...editingBuiltin.subfolders, clean];
      newSubInput = "";
    }
  }

  function removeSubfolder(sub: string) {
    if (!editingBuiltin) return;
    editingBuiltin.subfolders = editingBuiltin.subfolders.filter((s) => s !== sub);
  }

  async function handleSaveBuiltinConfig() {
    if (!editingBuiltin) return;
    try {
      await saveBuiltinRuleConfig(editingBuiltin);
      showToast(`Heurística '${editingBuiltin.display_name}' atualizada com sucesso!`, "success");
      showBuiltinEditModal = false;
      await reloadData();
    } catch (e: any) {
      showToast("Erro ao salvar heurística: " + e, "error");
    }
  }

  async function handleResetBuiltin(id: string) {
    try {
      await resetBuiltinRuleConfig(id);
      showToast("Heurística restaurada para o padrão do sistema.", "success");
      await reloadData();
    } catch (e: any) {
      showToast("Erro ao restaurar heurística: " + e, "error");
    }
  }

  async function handleResetAllBuiltins() {
    if (!confirm("Deseja restaurar todas as regras e heurísticas padrão para o estado de fábrica por completo? Todas as extensões, subpastas e destinos padrão serão resetados.")) {
      return;
    }

    try {
      await resetAllBuiltinRulesConfig();
      showToast("Todas as heurísticas e regras padrão foram restauradas para o estado de fábrica por completo!", "success");
      await reloadData();
    } catch (e: any) {
      showToast("Erro ao restaurar padrão: " + e, "error");
    }
  }

  async function handleDeleteLearnedRule(id: string) {
    try {
      await deleteLearnedRule(id);
      learnedRules = learnedRules.filter((r) => r.id !== id);
      showToast("Regra aprendida removida.", "success");
    } catch (e: any) {
      showToast("Erro ao remover regra aprendida: " + e, "error");
    }
  }

  // ==========================================
  // PASTAS GLOBAIS E SUGESTÕES DA IA
  // ==========================================

  function handleOpenCreateFolderModal() {
    folderForm = {
      parentGlobal: "Executaveis",
      name: "",
      desc: "",
    };
    showFolderModal = true;
  }

  function handleSaveCustomFolder() {
    if (!folderForm.name.trim()) {
      showToast("Digite o nome da subpasta.", "error");
      return;
    }
    const cleanName = folderForm.name.replace(/\s+/g, "-");
    const fullPath = `${folderForm.parentGlobal}/${cleanName}`;

    const existingGlobal = userCustomGlobalFolders.find((g) => g.id === folderForm.parentGlobal);
    if (existingGlobal) {
      if (!existingGlobal.subfolders.includes(cleanName)) {
        existingGlobal.subfolders.push(cleanName);
      }
    } else {
      userCustomGlobalFolders.push({
        id: folderForm.parentGlobal,
        name: folderForm.parentGlobal,
        desc: folderForm.desc || `Subpastas personalizadas em ${folderForm.parentGlobal}`,
        subfolders: [cleanName],
        is_default: false,
        color: "#d14d41",
      });
    }
    userCustomGlobalFolders = [...userCustomGlobalFolders];

    createCategory(fullPath, "#d14d41").then(() => {
      showToast(`Subpasta '${fullPath}' adicionada à estrutura ativa!`, "success");
      showFolderModal = false;
    }).catch((e) => {
      showToast("Erro ao registrar subpasta: " + e, "error");
    });
  }

  function handleRestoreDefaults() {
    userCustomGlobalFolders = [];
    showToast("Estrutura de Pastas Globais restaurada para o padrão do sistema!", "success");
  }

  function handleApproveSuggestion(sug: FolderSuggestion) {
    aiFolderSuggestions.update((list) => list.filter((s) => s.id !== sug.id));
    createCategory(sug.folder_path, "#8b7ec8").then(() => {
      showToast(`Sugestão '${sug.folder_path}' aprovada com sucesso!`, "success");
    }).catch((e) => {
      showToast("Erro ao aprovar sugestão: " + e, "error");
    });
  }

  function handleRejectSuggestion(sugId: string) {
    aiFolderSuggestions.update((list) => list.filter((s) => s.id !== sugId));
    showToast("Sugestão da IA recusada.", "info");
  }

  function getConditionFieldLabel(field: string): string {
    switch (field) {
      case "extension": return "Extensão";
      case "filename_contains": return "Nome Contém";
      case "parent_folder": return "Pasta Pai Contém";
      case "content_contains": return "Conteúdo Contém";
      case "size_greater": return "Tamanho > (MB)";
      case "size_smaller": return "Tamanho < (MB)";
      default: return field;
    }
  }

  function getConditionOperatorLabel(op: string): string {
    switch (op) {
      case "equals": return "é igual a";
      case "contains": return "contém";
      case "starts_with": return "começa com";
      case "ends_with": return "termina com";
      case "regex_match": return "combina com regex";
      case "greater_than": return "maior que";
      case "less_than": return "menor que";
      default: return op;
    }
  }

  function getSubfolderBehaviorLabel(beh?: string): string {
    switch (beh) {
      case "none": return "Sem subpastas (raiz)";
      case "by_year": return "Por Ano (/2026)";
      case "by_pattern": return "Extrair do Nome";
      case "auto":
      default: return "Auto (2+ arquivos)";
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
        <h1>Gerenciador de Regras e Heurísticas</h1>
      </div>
      <p class="subtitle">
        Edite regras personalizadas e heurísticas padrão do sistema (extensões, subpastas e destinos) com restauração completa de fábrica.
      </p>
    </div>

    <div class="header-actions">
      <button class="secondary-btn" on:click={() => currentView.set("settings")}>
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="15 18 9 12 15 6"></polyline>
        </svg>
        Configurações
      </button>

      {#if activeTab === "custom"}
        <button class="primary-btn" on:click={openCreateModal}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
          </svg>
          Nova Regra Personalizada
        </button>
      {:else if activeTab === "builtin"}
        <button class="warning-btn" on:click={handleResetAllBuiltins} title="Restaura todas as regras padrão, extensões e subpastas de fábrica por completo">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 7v6h6"></path>
            <path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"></path>
          </svg>
          Restaurar Padrão por Completo
        </button>
      {:else if activeTab === "global-folders"}
        <button class="secondary-btn" on:click={handleRestoreDefaults} title="Restaurar pastas globais para o padrão do sistema">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 7v6h6"></path>
            <path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"></path>
          </svg>
          Restaurar Padrão
        </button>
        <button class="primary-btn" on:click={handleOpenCreateFolderModal}>
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
          </svg>
          Adicionar Subpasta / Pasta
        </button>
      {/if}
    </div>
  </div>

  <!-- Segmented Tabs Bar -->
  <div class="tabs-bar">
    <!-- 1. Heurísticas Padrão (Editáveis) -->
    <button
      class="tab-btn"
      class:active={activeTab === "builtin"}
      on:click={() => (activeTab = "builtin")}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path>
        <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path>
      </svg>
      <span>Heurísticas Padrão (Editáveis)</span>
      <span class="tab-badge">{builtinRulesList.length}</span>
    </button>

    <!-- 2. Regras Personalizadas -->
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

    <!-- 3. Pastas Globais & Estrutura -->
    <button
      class="tab-btn"
      class:active={activeTab === "global-folders"}
      on:click={() => (activeTab = "global-folders")}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
      </svg>
      <span>Pastas Globais & Estrutura</span>
      <span class="tab-badge">{defaultGlobalFolders.length}</span>
    </button>

    <!-- 4. Sugestões da IA -->
    <button
      class="tab-btn"
      class:active={activeTab === "suggestions"}
      on:click={() => (activeTab = "suggestions")}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83"></path>
      </svg>
      <span>Sugestões da IA</span>
      {#if $aiFolderSuggestions.length > 0}
        <span class="tab-badge alert-badge">{$aiFolderSuggestions.length}</span>
      {:else}
        <span class="tab-badge">0</span>
      {/if}
    </button>

    <!-- 5. Regras Aprendidas -->
    <button
      class="tab-btn"
      class:active={activeTab === "learned"}
      on:click={() => (activeTab = "learned")}
    >
      <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="10"></circle>
        <polyline points="12 6 12 12 16 14"></polyline>
      </svg>
      <span>Regras Aprendidas</span>
      <span class="tab-badge">{learnedRules.length}</span>
    </button>
  </div>

  <!-- Tab Contents -->
  <div class="rules-content">
    {#if isLoading}
      <div class="loading-state">
        <div class="spinner"></div>
        <span>Carregando dados...</span>
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
          <p>Crie regras condicionais com controle completo de condições, destino e agrupamento automático de subpastas.</p>
          <div class="empty-actions-row">
            <button class="primary-btn" on:click={openCreateModal}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <line x1="12" y1="5" x2="12" y2="19"></line>
                <line x1="5" y1="12" x2="19" y2="12"></line>
              </svg>
              Criar Nova Regra
            </button>
          </div>
        </div>
      {:else}
        <div class="rules-grid">
          {#each customRules as rule (rule.id)}
            <div class="rule-card" class:is-disabled={!rule.is_enabled}>
              <div class="rule-card-header">
                <div class="rule-title-box">
                  <span class="rule-name">{rule.name}</span>
                  <span class="version-pill" title="Versão da regra">v{rule.version || 1}</span>
                  <span class="priority-pill">P{rule.priority}</span>
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
                </div>
              </div>

              <!-- Rule Pipeline Box -->
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
                    {rule.action_type === "move_category" ? "Mover para" : "Aplicar Tag"}
                  </span>
                  <span class="action-target">"{rule.action_value}"</span>
                </div>

                <!-- Subfolder Behavior Indicator -->
                <div class="behavior-pill-row">
                  <span class="behavior-label">Subpastas:</span>
                  <span class="behavior-tag">{getSubfolderBehaviorLabel(rule.subfolder_behavior)}</span>
                </div>
              </div>

              <!-- Card Action Bar -->
              <div class="rule-card-footer">
                <button class="rule-action-btn primary" on:click={() => openEditModal(rule)}>
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                  </svg>
                  Editar Regra
                </button>

                <button class="rule-action-btn secondary" on:click={() => handleOpenHistory(rule)}>
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <circle cx="12" cy="12" r="10"></circle>
                    <polyline points="12 6 12 12 16 14"></polyline>
                  </svg>
                  Histórico (v{rule.version || 1})
                </button>

                {#if rule.version > 1 || rule.original_config}
                  <button class="rule-action-btn warning" on:click={() => handleRestoreRuleOriginal(rule)}>
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M3 7v6h6"></path>
                      <path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"></path>
                    </svg>
                    Restaurar Original
                  </button>
                {/if}

                <button class="rule-action-btn danger" on:click={() => handleDeleteCustomRule(rule.id)} title="Excluir regra">
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="3 6 5 6 21 6"></polyline>
                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                  </svg>
                  Excluir
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}

    <!-- 2. TAB: REGRAS E HEURÍSTICAS PADRÃO (EDITÁVEIS) -->
    {:else if activeTab === "builtin"}
      <div class="builtin-section-container">
        <div class="builtin-intro-bar">
          <div class="intro-text">
            <span>As heurísticas padrão definem como o Indexo organiza arquivos com base em extensões, palavras-chave e subpastas.</span>
            <span class="subtext">Você pode personalizar livremente cada categoria ou restaurar tudo para o padrão de fábrica.</span>
          </div>
        </div>

        <div class="builtin-grid">
          {#each builtinRulesList as cat (cat.id)}
            <div class="builtin-card" class:customized-border={cat.is_customized}>
              <div class="builtin-header">
                <div class="builtin-title-row">
                  <span class="builtin-name">{cat.display_name}</span>
                  {#if cat.is_customized}
                    <span class="custom-badge-pill">Modificado</span>
                  {:else}
                    <span class="default-badge-pill">Padrão</span>
                  {/if}
                </div>
                <span class="target-path-pill">{cat.target_path}/</span>
              </div>

              <p class="builtin-desc">{cat.description}</p>

              <!-- Extensões -->
              <div class="builtin-tags-section">
                <div class="tags-header">
                  <span class="section-sub">Extensões ({cat.extensions.length}):</span>
                </div>
                <div class="tags-row">
                  {#each cat.extensions as ext}
                    <span class="ext-pill">.{ext}</span>
                  {/each}
                </div>
              </div>

              <!-- Subpastas Configuradas -->
              <div class="builtin-tags-section">
                <div class="tags-header">
                  <span class="section-sub">Subpastas Ativas ({cat.subfolders.length}):</span>
                  <span class="behavior-mini-tag">{getSubfolderBehaviorLabel(cat.subfolder_behavior)}</span>
                </div>
                <div class="tags-row">
                  {#each cat.subfolders as sub}
                    <span class="sub-pill">{sub}</span>
                  {/each}
                </div>
              </div>

              <!-- Palavras-Chave -->
              {#if cat.keywords.length > 0}
                <div class="builtin-tags-section">
                  <span class="section-sub">Palavras-Chave ({cat.keywords.length}):</span>
                  <div class="tags-row">
                    {#each cat.keywords as kw}
                      <span class="kw-pill">{kw}</span>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- Action Footer -->
              <div class="builtin-card-footer">
                <button class="rule-action-btn primary" on:click={() => openEditBuiltinModal(cat)}>
                  <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                    <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                  </svg>
                  Editar Heurística
                </button>

                {#if cat.is_customized}
                  <button class="rule-action-btn warning" on:click={() => handleResetBuiltin(cat.id)}>
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <path d="M3 7v6h6"></path>
                      <path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"></path>
                    </svg>
                    Restaurar Padrão
                  </button>
                {/if}
              </div>
            </div>
          {/each}
        </div>
      </div>

    <!-- 3. TAB: PASTAS GLOBAIS & ESTRUTURA -->
    {:else if activeTab === "global-folders"}
      <div class="global-folders-view">
        <div class="taxonomy-section">
          <div class="section-header-bar">
            <div class="section-title-group">
              <span class="section-badge default">Padrão do Sistema</span>
              <h2 class="section-h2">Pastas Globais Principais (Sem Espaços)</h2>
            </div>
            <span class="section-note">Criadas sob demanda e protegidas contra desestruturação.</span>
          </div>

          <div class="global-folders-grid">
            {#each defaultGlobalFolders as folder}
              <div class="global-card">
                <div class="global-card-header">
                  <div class="global-title-row">
                    <span class="color-dot" style="background: {folder.color}"></span>
                    <span class="global-folder-name">{folder.name}/</span>
                  </div>
                  <span class="built-in-pill">Padrão</span>
                </div>
                <p class="global-folder-desc">{folder.desc}</p>

                <div class="subfolders-list-section">
                  <span class="subfolders-title">Subpastas Dinâmicas Especializadas:</span>
                  <div class="subfolders-tags">
                    {#each folder.subfolders as sub}
                      <span class="subfolder-chip">{sub}</span>
                    {/each}
                  </div>
                </div>
              </div>
            {/each}
          </div>
        </div>

        {#if userCustomGlobalFolders.length > 0}
          <div class="taxonomy-section custom-section">
            <div class="section-header-bar">
              <div class="section-title-group">
                <span class="section-badge custom">Criado pelo Usuário</span>
                <h2 class="section-h2">Estruturas Personalizadas Adicionadas</h2>
              </div>
            </div>

            <div class="global-folders-grid">
              {#each userCustomGlobalFolders as folder}
                <div class="global-card custom-card">
                  <div class="global-card-header">
                    <div class="global-title-row">
                      <span class="color-dot" style="background: {folder.color}"></span>
                      <span class="global-folder-name">{folder.name}/</span>
                    </div>
                    <span class="custom-pill">Personalizado</span>
                  </div>
                  <p class="global-folder-desc">{folder.desc}</p>

                  <div class="subfolders-list-section">
                    <span class="subfolders-title">Subpastas Criadas:</span>
                    <div class="subfolders-tags">
                      {#each folder.subfolders as sub}
                        <span class="subfolder-chip custom-chip">{sub}</span>
                      {/each}
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>

    <!-- 4. TAB: SUGESTÕES DA IA -->
    {:else if activeTab === "suggestions"}
      {#if $aiFolderSuggestions.length === 0}
        <div class="empty-state">
          <div class="empty-icon">
            <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="#24837b" stroke-width="1.5">
              <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
              <polyline points="22 4 12 14.01 9 11.01"></polyline>
            </svg>
          </div>
          <h3>Nenhuma sugestão pendente</h3>
          <p>O motor de inteligência do Indexo monitora agrupamentos e padrões repetidos durante as varreduras. Novas sugestões de subpastas aparecerão aqui para sua aprovação.</p>
        </div>
      {:else}
        <div class="suggestions-container">
          <div class="suggestions-header-bar">
            <span class="suggestions-info">
              A IA identificou novos agrupamentos de arquivos e sugere as seguintes subpastas:
            </span>
          </div>

          <div class="suggestions-list">
            {#each $aiFolderSuggestions as sug (sug.id)}
              <div class="suggestion-card glass-panel">
                <div class="sug-content">
                  <div class="sug-path-row">
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#8b7ec8" stroke-width="2">
                      <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
                    </svg>
                    <span class="sug-path">{sug.folder_path}</span>
                  </div>
                  <p class="sug-reason">{sug.reason}</p>
                  <span class="sug-origin">Origem: {sug.suggested_at}</span>
                </div>

                <div class="sug-actions">
                  <button class="approve-btn" on:click={() => handleApproveSuggestion(sug)}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <polyline points="20 6 9 17 4 12"></polyline>
                    </svg>
                    Aprovar Sugestão
                  </button>
                  <button class="reject-btn" on:click={() => handleRejectSuggestion(sug.id)}>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      <line x1="18" y1="6" x2="6" y2="18"></line>
                      <line x1="6" y1="6" x2="18" y2="18"></line>
                    </svg>
                    Recusar
                  </button>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}

    <!-- 5. TAB: REGRAS APRENDIDAS PELA IA -->
    {:else if activeTab === "learned"}
      {#if learnedRules.length === 0}
        <div class="empty-state">
          <div class="empty-icon">
            <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
              <circle cx="12" cy="12" r="10"></circle>
              <polyline points="12 6 12 12 16 14"></polyline>
            </svg>
          </div>
          <h3>Nenhuma regra aprendida ainda</h3>
          <p>Quando você reatribuir arquivos no Preview ou no modal de Não-Identificados, o Indexo aprenderá seus padrões automaticamente.</p>
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
    <div class="modal-card wide">
      <div class="modal-header-row">
        <h2>{editingRuleId ? "Editar Regra Personalizada" : "Nova Regra Personalizada"}</h2>
        <button class="close-btn" on:click={() => (showRuleModal = false)}>✕</button>
      </div>

      <div class="modal-body-form">
        <!-- 1. Identificação -->
        <label class="form-field">
          <span class="field-label">Nome da Regra:</span>
          <input
            type="text"
            placeholder="Ex: ROMs Nintendo 3DS ou Relatórios Contábeis"
            bind:value={ruleForm.name}
            class="text-input"
          />
        </label>

        <!-- 2. Condição (SE) -->
        <div class="form-group-box">
          <span class="group-title">1. Condição de Ativação (SE):</span>

          <div class="form-row">
            <label class="form-field">
              <span class="field-label">Campo a Analisar:</span>
              <select bind:value={ruleForm.condition_field} class="select-input">
                <option value="extension">Extensão do Arquivo</option>
                <option value="filename_contains">Nome Contém</option>
                <option value="parent_folder">Pasta de Origem Contém</option>
                <option value="content_contains">Conteúdo / Texto Lido (OCR)</option>
                <option value="size_greater">Tamanho Maior que (MB)</option>
                <option value="size_smaller">Tamanho Menor que (MB)</option>
              </select>
            </label>

            <label class="form-field">
              <span class="field-label">Operador Lógico:</span>
              <select bind:value={ruleForm.condition_operator} class="select-input">
                <option value="equals">É exatamente igual a</option>
                <option value="contains">Contém o termo</option>
                <option value="starts_with">Começa com</option>
                <option value="ends_with">Termina com</option>
                <option value="regex_match">Expressão Regular (Regex)</option>
                <option value="greater_than">Maior que</option>
                <option value="less_than">Menor que</option>
              </select>
            </label>
          </div>

          <label class="form-field">
            <span class="field-label">Valor / Padrão da Condição:</span>
            <input
              type="text"
              placeholder="Ex: gba, fatura, 2026, etc."
              bind:value={ruleForm.condition_value}
              class="text-input"
            />
          </label>
        </div>

        <!-- 3. Ação (ENTÃO) -->
        <div class="form-group-box">
          <span class="group-title">2. Ação e Destino (ENTÃO):</span>

          <div class="form-row">
            <label class="form-field">
              <span class="field-label">Tipo de Ação:</span>
              <select bind:value={ruleForm.action_type} class="select-input">
                <option value="move_category">Mover para Categoria / Pasta</option>
                <option value="apply_tag">Aplicar Tag Semântica</option>
              </select>
            </label>

            <label class="form-field">
              <span class="field-label">Caminho de Destino (use hífens):</span>
              <input
                type="text"
                placeholder="Ex: Executaveis/Jogos-Emuladores-ROMs/Nintendo-3DS"
                bind:value={ruleForm.action_value}
                class="text-input"
              />
            </label>
          </div>
        </div>

        <!-- 4. Comportamento de Subpastas & Prioridade -->
        <div class="form-group-box">
          <span class="group-title">3. Agrupamento em Subpastas & Prioridade:</span>

          <div class="form-row">
            <label class="form-field">
              <span class="field-label">Comportamento de Subpastas:</span>
              <select bind:value={ruleForm.subfolder_behavior} class="select-input">
                <option value="auto">Automático (agrupar quando houver 2+ arquivos semelhantes)</option>
                <option value="by_pattern">Criar subpasta extraindo o padrão do nome do arquivo</option>
                <option value="by_year">Agrupar em subpastas por Ano (ex: /2026)</option>
                <option value="none">Sem subpastas (manter na raiz da categoria)</option>
              </select>
            </label>

            <label class="form-field">
              <span class="field-label">Prioridade da Regra (1 a 100):</span>
              <input
                type="number"
                min="1"
                max="100"
                bind:value={ruleForm.priority}
                class="text-input"
              />
            </label>
          </div>
        </div>
      </div>

      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showRuleModal = false)}>Cancelar</button>
        <button class="primary-btn" on:click={handleSaveRule}>
          {editingRuleId ? "Salvar Nova Versão" : "Criar Regra"}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Histórico de Versões da Regra Customizada -->
{#if showHistoryModal}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (showHistoryModal = false)}
    on:keydown={(e) => e.key === "Escape" && (showHistoryModal = false)}
  >
    <div class="modal-card wide">
      <div class="modal-header-row">
        <div>
          <h2>Histórico de Versões: {selectedRuleForHistory?.name}</h2>
          <span class="version-subtitle">Versão Atual: v{selectedRuleForHistory?.version || 1}</span>
        </div>
        <button class="close-btn" on:click={() => (showHistoryModal = false)}>✕</button>
      </div>

      <div class="history-modal-body">
        {#if isLoadingHistory}
          <div class="mini-loading">
            <div class="spinner"></div>
            <span>Carregando histórico de edições...</span>
          </div>
        {:else if ruleHistoryList.length === 0}
          <div class="empty-history-box">
            <p>Esta regra está em sua versão inicial original (v1) e ainda não possui modificações anteriores registradas.</p>
          </div>
        {:else}
          <div class="timeline-container">
            {#each ruleHistoryList as hist (hist.id)}
              <div class="timeline-item">
                <div class="timeline-dot"></div>
                <div class="timeline-card">
                  <div class="timeline-card-header">
                    <div class="version-row">
                      <span class="version-badge-hist">v{hist.version}</span>
                      <span class="hist-saved-at">{new Date(hist.saved_at).toLocaleString()}</span>
                    </div>
                    <span class="hist-note">{hist.note || "Snapshot de edição"}</span>
                  </div>

                  <div class="hist-rule-snapshot">
                    <div>
                      <strong>SE:</strong> [{getConditionFieldLabel(hist.condition_field)}] {getConditionOperatorLabel(hist.condition_operator)} "{hist.condition_value}"
                    </div>
                    <div>
                      <strong>ENTÃO:</strong> Mover para "{hist.action_value}"
                    </div>
                    <div>
                      <strong>Subpastas:</strong> {getSubfolderBehaviorLabel(hist.subfolder_behavior)} | <strong>Prioridade:</strong> {hist.priority}
                    </div>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <div class="modal-actions">
        {#if selectedRuleForHistory && (selectedRuleForHistory.version > 1 || selectedRuleForHistory.original_config)}
          <button class="warning-btn" on:click={() => selectedRuleForHistory && handleRestoreRuleOriginal(selectedRuleForHistory)}>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M3 7v6h6"></path>
              <path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"></path>
            </svg>
            Restaurar Configuração Original
          </button>
        {/if}
        <button class="secondary-btn" on:click={() => (showHistoryModal = false)}>Fechar</button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Editar Heurística / Regra Padrão do Sistema -->
{#if showBuiltinEditModal && editingBuiltin}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (showBuiltinEditModal = false)}
    on:keydown={(e) => e.key === "Escape" && (showBuiltinEditModal = false)}
  >
    <div class="modal-card wide">
      <div class="modal-header-row">
        <div>
          <h2>Editar Heurística Padrão: {editingBuiltin.display_name}</h2>
          <span class="version-subtitle">Grupo: {editingBuiltin.group_name}</span>
        </div>
        <button class="close-btn" on:click={() => (showBuiltinEditModal = false)}>✕</button>
      </div>

      <div class="modal-body-form">
        <!-- Seção: Nome e Destino Base -->
        <div class="form-group-box">
          <div class="group-header-row">
            <span class="group-title">Destino e Identificação da Categoria:</span>
            <button class="restore-field-btn" on:click={restoreFieldDestination} title="Restaura apenas o nome e caminho de destino para o padrão">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 7v6h6"></path>
                <path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"></path>
              </svg>
              Restaurar Destino Padrão
            </button>
          </div>

          <div class="form-row">
            <label class="form-field">
              <span class="field-label">Nome de Exibição:</span>
              <input
                type="text"
                bind:value={editingBuiltin.display_name}
                class="text-input"
              />
            </label>

            <label class="form-field">
              <span class="field-label">Caminho de Destino Base (use hífens):</span>
              <input
                type="text"
                bind:value={editingBuiltin.target_path}
                class="text-input"
              />
            </label>
          </div>

          <label class="form-field">
            <span class="field-label">Descrição:</span>
            <input
              type="text"
              bind:value={editingBuiltin.description}
              class="text-input"
            />
          </label>
        </div>

        <!-- 1. Gerenciar Extensões -->
        <div class="form-group-box">
          <div class="group-header-row">
            <span class="group-title">1. Extensões de Arquivo Reconhecidas ({editingBuiltin.extensions.length}):</span>
            <button class="restore-field-btn" on:click={restoreFieldExtensions} title="Restaura apenas as extensões originais desta categoria">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 7v6h6"></path>
                <path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"></path>
              </svg>
              Restaurar Extensões Padrão
            </button>
          </div>

          <div class="tag-input-row">
            <input
              type="text"
              placeholder="Digite uma extensão (ex: webp ou psd) e clique em Adicionar"
              bind:value={newExtInput}
              class="text-input"
              on:keydown={(e) => e.key === "Enter" && addExtension()}
            />
            <button class="secondary-btn" on:click={addExtension}>Adicionar Extensão</button>
          </div>

          <div class="chips-editor-container">
            {#each editingBuiltin.extensions as ext}
              <span class="chip-item">
                <span>.{ext}</span>
                <button class="chip-remove-btn" on:click={() => removeExtension(ext)} title="Remover extensão">✕</button>
              </span>
            {/each}
          </div>
        </div>

        <!-- 2. Gerenciar Subpastas -->
        <div class="form-group-box">
          <div class="group-header-row">
            <span class="group-title">2. Subpastas Ativas ({editingBuiltin.subfolders.length}):</span>
            <div class="group-right-actions">
              <div class="subfolder-behavior-picker">
                <span class="behavior-label">Agrupamento:</span>
                <select bind:value={editingBuiltin.subfolder_behavior} class="select-input mini">
                  <option value="auto">Automático (2+ arquivos)</option>
                  <option value="by_pattern">Extrair do Nome</option>
                  <option value="by_year">Por Ano (/2026)</option>
                  <option value="none">Sem subpastas (raiz)</option>
                </select>
              </div>

              <button class="restore-field-btn" on:click={restoreFieldSubfolders} title="Restaura apenas as subpastas originais desta categoria">
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M3 7v6h6"></path>
                  <path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"></path>
                </svg>
                Restaurar Subpastas Padrão
              </button>
            </div>
          </div>

          <div class="tag-input-row">
            <input
              type="text"
              placeholder="Digite o nome da subpasta (com hífens) e clique em Adicionar"
              bind:value={newSubInput}
              class="text-input"
              on:keydown={(e) => e.key === "Enter" && addSubfolder()}
            />
            <button class="secondary-btn" on:click={addSubfolder}>Adicionar Subpasta</button>
          </div>

          <div class="chips-editor-container">
            {#each editingBuiltin.subfolders as sub}
              <span class="chip-item subfolder-chip-edit">
                <span>{sub}</span>
                <button class="chip-remove-btn" on:click={() => removeSubfolder(sub)} title="Remover subpasta">✕</button>
              </span>
            {/each}
          </div>
        </div>

        <!-- 3. Gerenciar Palavras-Chave -->
        <div class="form-group-box">
          <div class="group-header-row">
            <span class="group-title">3. Palavras-Chave e Tokens ({editingBuiltin.keywords.length}):</span>
            <button class="restore-field-btn" on:click={restoreFieldKeywords} title="Restaura apenas os termos e palavras-chave padrão">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M3 7v6h6"></path>
                <path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"></path>
              </svg>
              Restaurar Termos Padrão
            </button>
          </div>

          <div class="tag-input-row">
            <input
              type="text"
              placeholder="Digite um termo/token e clique em Adicionar"
              bind:value={newKwInput}
              class="text-input"
              on:keydown={(e) => e.key === "Enter" && addKeyword()}
            />
            <button class="secondary-btn" on:click={addKeyword}>Adicionar Termo</button>
          </div>

          <div class="chips-editor-container">
            {#each editingBuiltin.keywords as kw}
              <span class="chip-item kw-chip-edit">
                <span>{kw}</span>
                <button class="chip-remove-btn" on:click={() => removeKeyword(kw)} title="Remover palavra-chave">✕</button>
              </span>
            {/each}
          </div>
        </div>
      </div>

      <div class="modal-actions">
        <button class="warning-btn" on:click={restoreAllFieldsForEditingBuiltin} title="Restaura todos os campos desta categoria para o padrão de fábrica">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M3 7v6h6"></path>
            <path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"></path>
          </svg>
          Restaurar Categoria por Completo
        </button>
        <button class="secondary-btn" on:click={() => (showBuiltinEditModal = false)}>Cancelar</button>
        <button class="primary-btn" on:click={handleSaveBuiltinConfig}>
          Salvar Heurística
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Adicionar Subpasta / Pasta Customizada -->
{#if showFolderModal}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (showFolderModal = false)}
    on:keydown={(e) => e.key === "Escape" && (showFolderModal = false)}
  >
    <div class="modal-card">
      <div class="modal-header-row">
        <h2>Adicionar Subpasta ou Pasta Personalizada</h2>
        <button class="close-btn" on:click={() => (showFolderModal = false)}>✕</button>
      </div>

      <div class="modal-body-form">
        <label class="form-field">
          <span class="field-label">Pasta Global Pai:</span>
          <select bind:value={folderForm.parentGlobal} class="select-input">
            {#each defaultGlobalFolders as g}
              <option value={g.name}>{g.name}/</option>
            {/each}
          </select>
        </label>

        <label class="form-field">
          <span class="field-label">Nome da Subpasta (sem espaços, com hífens):</span>
          <input
            type="text"
            placeholder="Ex: Jogos-GOG ou Meus-Trabalhos-2026"
            bind:value={folderForm.name}
            class="text-input"
            on:input={(e) => {
              folderForm.name = e.currentTarget.value.replace(/\s+/g, "-");
            }}
          />
        </label>

        <label class="form-field">
          <span class="field-label">Descrição (Opcional):</span>
          <input
            type="text"
            placeholder="Ex: Jogos adquiridos pela plataforma GOG"
            bind:value={folderForm.desc}
            class="text-input"
          />
        </label>

        <div class="preview-mini-path">
          <span>Caminho Resultante: </span>
          <strong>{folderForm.parentGlobal}/{folderForm.name || "[Nome]"}</strong>
        </div>
      </div>

      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showFolderModal = false)}>Cancelar</button>
        <button class="primary-btn" on:click={handleSaveCustomFolder}>
          Salvar na Estrutura
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
    overflow-x: auto;
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
    white-space: nowrap;
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

  .tab-badge.alert-badge {
    background: #d14d41;
    color: #fff;
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

  /* Custom Rules Grid */
  .rules-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(370px, 1fr));
    gap: 1rem;
  }

  .rule-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    padding: 1.1rem;
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
    gap: 0.45rem;
    flex-wrap: wrap;
  }

  .rule-name {
    font-weight: 700;
    font-size: 0.92rem;
    color: var(--text-primary);
  }

  .version-pill {
    font-size: 0.65rem;
    font-weight: 800;
    background: rgba(36, 131, 123, 0.15);
    color: #24837b;
    border: 1px solid rgba(36, 131, 123, 0.3);
    padding: 0.05rem 0.35rem;
    border-radius: var(--radius-sm);
  }

  .priority-pill {
    font-size: 0.65rem;
    font-weight: 700;
    background: var(--bg-tertiary);
    color: var(--text-muted);
    padding: 0.05rem 0.35rem;
    border-radius: var(--radius-full);
  }

  .rule-header-actions {
    display: flex;
    align-items: center;
    gap: 0.35rem;
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
    font-weight: 800;
    font-size: 0.68rem;
    background: var(--bg-tertiary);
    color: var(--text-muted);
    padding: 0.1rem 0.35rem;
    border-radius: var(--radius-sm);
  }

  .keyword-tag.action {
    background: rgba(36, 131, 123, 0.15);
    color: #24837b;
  }

  .field-txt, .operator-txt {
    color: var(--text-muted);
  }

  .value-highlight, .action-target {
    font-weight: 700;
    color: var(--text-primary);
  }

  .action-target {
    color: var(--accent-primary);
  }

  .arrow-down {
    color: var(--text-muted);
    font-size: 0.75rem;
    margin-left: 0.5rem;
  }

  .behavior-pill-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin-top: 0.25rem;
    padding-top: 0.35rem;
    border-top: 1px dashed var(--border-subtle);
    font-size: 0.72rem;
  }

  .behavior-label {
    color: var(--text-muted);
  }

  .behavior-tag {
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    padding: 0.05rem 0.4rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .rule-card-footer {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    flex-wrap: wrap;
    margin-top: 0.25rem;
    padding-top: 0.5rem;
    border-top: 1px solid var(--border-subtle);
  }

  .rule-action-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.35rem 0.65rem;
    font-size: 0.74rem;
    font-weight: 600;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all 120ms ease;
    border: 1px solid transparent;
  }

  .rule-action-btn.primary {
    background: rgba(36, 131, 123, 0.12);
    color: #24837b;
    border-color: rgba(36, 131, 123, 0.3);
  }

  .rule-action-btn.primary:hover {
    background: rgba(36, 131, 123, 0.22);
  }

  .rule-action-btn.secondary {
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border-color: var(--border-medium);
  }

  .rule-action-btn.secondary:hover {
    background: var(--bg-hover);
  }

  .rule-action-btn.warning {
    background: rgba(218, 112, 44, 0.12);
    color: #da702c;
    border-color: rgba(218, 112, 44, 0.35);
  }

  .rule-action-btn.warning:hover {
    background: rgba(218, 112, 44, 0.22);
  }

  .rule-action-btn.danger {
    background: transparent;
    color: var(--text-muted);
    border-color: var(--border-subtle);
    margin-left: auto;
  }

  .rule-action-btn.danger:hover {
    background: rgba(209, 77, 65, 0.12);
    color: #d14d41;
    border-color: rgba(209, 77, 65, 0.3);
  }

  .empty-actions-row {
    margin-top: 0.5rem;
  }

  /* Builtin Tab & Heuristics */
  .builtin-section-container {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
  }

  .builtin-intro-bar {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    padding: 0.85rem 1.25rem;
  }

  .intro-text {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    font-size: 0.82rem;
    color: var(--text-primary);
  }

  .intro-text .subtext {
    font-size: 0.76rem;
    color: var(--text-muted);
  }

  .builtin-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
    gap: 1.1rem;
  }

  .builtin-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    transition: all 150ms ease;
  }

  .builtin-card.customized-border {
    border-color: rgba(218, 112, 44, 0.5);
    background: rgba(218, 112, 44, 0.03);
  }

  .builtin-header {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .builtin-title-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .builtin-name {
    font-size: 0.98rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .target-path-pill {
    font-size: 0.75rem;
    font-weight: 700;
    color: var(--accent-primary);
    font-family: monospace;
  }

  .default-badge-pill {
    font-size: 0.65rem;
    font-weight: 700;
    background: var(--bg-tertiary);
    color: var(--text-muted);
    padding: 0.1rem 0.4rem;
    border-radius: var(--radius-sm);
  }

  .custom-badge-pill {
    font-size: 0.65rem;
    font-weight: 800;
    background: rgba(218, 112, 44, 0.15);
    color: #da702c;
    border: 1px solid rgba(218, 112, 44, 0.3);
    padding: 0.1rem 0.4rem;
    border-radius: var(--radius-sm);
  }

  .builtin-desc {
    font-size: 0.8rem;
    color: var(--text-muted);
    margin: 0;
  }

  .builtin-tags-section {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .tags-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .section-sub {
    font-size: 0.72rem;
    font-weight: 600;
    color: var(--text-muted);
  }

  .behavior-mini-tag {
    font-size: 0.68rem;
    color: var(--text-muted);
    background: var(--bg-tertiary);
    padding: 0.05rem 0.35rem;
    border-radius: var(--radius-sm);
  }

  .tags-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
  }

  .ext-pill {
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 0.1rem 0.4rem;
    font-size: 0.72rem;
    font-family: monospace;
    color: var(--text-primary);
  }

  .sub-pill {
    background: rgba(36, 131, 123, 0.1);
    color: #24837b;
    border: 1px solid rgba(36, 131, 123, 0.25);
    border-radius: var(--radius-sm);
    padding: 0.1rem 0.4rem;
    font-size: 0.72rem;
    font-family: monospace;
  }

  .kw-pill {
    background: rgba(208, 162, 21, 0.12);
    color: #bc5215;
    border-radius: var(--radius-sm);
    padding: 0.1rem 0.4rem;
    font-size: 0.72rem;
  }

  .builtin-card-footer {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: auto;
    padding-top: 0.6rem;
    border-top: 1px solid var(--border-subtle);
  }

  /* Global Folders View */
  .global-folders-view {
    display: flex;
    flex-direction: column;
    gap: 1.75rem;
  }

  .taxonomy-section {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .section-header-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-wrap: wrap;
    gap: 0.75rem;
  }

  .section-title-group {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .section-badge {
    font-size: 0.72rem;
    font-weight: 700;
    padding: 0.2rem 0.55rem;
    border-radius: var(--radius-sm);
  }

  .section-badge.default {
    background: rgba(36, 131, 123, 0.15);
    color: #24837b;
    border: 1px solid rgba(36, 131, 123, 0.3);
  }

  .section-badge.custom {
    background: rgba(209, 77, 65, 0.15);
    color: #d14d41;
    border: 1px solid rgba(209, 77, 65, 0.3);
  }

  .section-h2 {
    font-size: 1.1rem;
    font-weight: 700;
    color: var(--text-primary);
    margin: 0;
  }

  .section-note {
    font-size: 0.78rem;
    color: var(--text-muted);
  }

  .global-folders-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
    gap: 1rem;
  }

  .global-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    padding: 1.15rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .global-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .global-title-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .color-dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
  }

  .global-folder-name {
    font-size: 1rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .built-in-pill, .custom-pill {
    font-size: 0.68rem;
    font-weight: 700;
    padding: 0.1rem 0.45rem;
    border-radius: var(--radius-full);
  }

  .built-in-pill {
    background: var(--bg-tertiary);
    color: var(--text-muted);
  }

  .custom-pill {
    background: rgba(209, 77, 65, 0.15);
    color: #d14d41;
  }

  .global-folder-desc {
    font-size: 0.8rem;
    color: var(--text-muted);
    margin: 0;
  }

  .subfolders-list-section {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    margin-top: auto;
    padding-top: 0.5rem;
    border-top: 1px solid var(--border-subtle);
  }

  .subfolders-title {
    font-size: 0.72rem;
    font-weight: 600;
    color: var(--text-muted);
  }

  .subfolders-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }

  .subfolder-chip {
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 0.2rem 0.45rem;
    font-size: 0.72rem;
    color: var(--text-primary);
    font-family: monospace;
  }

  .subfolder-chip.custom-chip {
    border-color: rgba(209, 77, 65, 0.4);
    background: rgba(209, 77, 65, 0.06);
  }

  /* AI Suggestions Tab */
  .suggestions-container {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .suggestions-header-bar {
    font-size: 0.84rem;
    color: var(--text-primary);
    font-weight: 500;
  }

  .suggestions-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .suggestion-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    padding: 1rem 1.25rem;
    flex-wrap: wrap;
  }

  .sug-content {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .sug-path-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .sug-path {
    font-size: 0.95rem;
    font-weight: 700;
    color: var(--text-primary);
    font-family: monospace;
  }

  .sug-reason {
    font-size: 0.82rem;
    color: var(--text-muted);
    margin: 0;
  }

  .sug-origin {
    font-size: 0.72rem;
    color: var(--accent-primary);
    font-weight: 500;
  }

  .sug-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .approve-btn {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    background: #24837b;
    color: #fff;
    border: none;
    border-radius: var(--radius-md);
    padding: 0.5rem 0.85rem;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 120ms ease;
  }

  .approve-btn:hover {
    filter: brightness(1.1);
  }

  .reject-btn {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    background: transparent;
    color: var(--text-muted);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    padding: 0.5rem 0.85rem;
    font-size: 0.8rem;
    cursor: pointer;
    transition: all 120ms ease;
  }

  .reject-btn:hover {
    background: var(--bg-hover);
    color: #d14d41;
  }

  /* Learned Rules Table */
  .learned-table-container {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .learned-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.82rem;
  }

  .learned-table th {
    background: var(--bg-tertiary);
    padding: 0.75rem 1rem;
    text-align: left;
    font-weight: 600;
    color: var(--text-muted);
    border-bottom: 1px solid var(--border-subtle);
  }

  .learned-table td {
    padding: 0.65rem 1rem;
    border-bottom: 1px solid var(--border-subtle);
    color: var(--text-primary);
  }

  .pattern-mono {
    font-family: monospace;
    font-weight: 600;
  }

  .type-pill {
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    padding: 0.1rem 0.35rem;
    font-size: 0.72rem;
    color: var(--text-muted);
  }

  .cat-pill {
    padding-left: 0.5rem;
    font-weight: 600;
  }

  /* Modals */
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
    max-height: 90vh;
    overflow-y: auto;
  }

  .modal-card.wide {
    width: 680px;
  }

  .modal-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .modal-header-row h2 {
    font-size: 1.15rem;
    font-weight: 700;
    margin: 0;
    color: var(--text-primary);
  }

  .version-subtitle {
    font-size: 0.75rem;
    color: #24837b;
    font-weight: 700;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 1.1rem;
    cursor: pointer;
  }

  .modal-body-form {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .form-field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .field-label {
    font-size: 0.8rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .text-input, .select-input {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    padding: 0.5rem 0.75rem;
    color: var(--text-primary);
    font-size: 0.85rem;
    outline: none;
  }

  .select-input.mini {
    padding: 0.25rem 0.5rem;
    font-size: 0.78rem;
  }

  .text-input:focus, .select-input:focus {
    border-color: var(--accent-primary);
  }

  .form-group-box {
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 0.85rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .group-title {
    font-size: 0.75rem;
    font-weight: 700;
    color: var(--text-muted);
    text-transform: uppercase;
  }

  .group-header-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .group-right-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .restore-field-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    background: transparent;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 0.2rem 0.5rem;
    font-size: 0.72rem;
    font-weight: 600;
    color: var(--accent-primary);
    cursor: pointer;
    transition: all 120ms ease;
  }

  .restore-field-btn:hover {
    background: rgba(36, 131, 123, 0.12);
    border-color: rgba(36, 131, 123, 0.35);
  }

  .subfolder-behavior-picker {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .tag-input-row {
    display: flex;
    gap: 0.5rem;
  }

  .tag-input-row input {
    flex: 1;
  }

  .chips-editor-container {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    max-height: 120px;
    overflow-y: auto;
    padding: 0.25rem 0;
  }

  .chip-item {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 0.2rem 0.5rem;
    font-size: 0.76rem;
    font-family: monospace;
    color: var(--text-primary);
  }

  .chip-item.subfolder-chip-edit {
    background: rgba(36, 131, 123, 0.1);
    color: #24837b;
    border-color: rgba(36, 131, 123, 0.3);
  }

  .chip-item.kw-chip-edit {
    background: rgba(208, 162, 21, 0.12);
    color: #bc5215;
  }

  .chip-remove-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 0.75rem;
    cursor: pointer;
    padding: 0;
    display: flex;
    align-items: center;
  }

  .chip-remove-btn:hover {
    color: #d14d41;
  }

  .form-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }

  .preview-mini-path {
    background: var(--bg-tertiary);
    border-radius: var(--radius-sm);
    padding: 0.5rem 0.75rem;
    font-size: 0.8rem;
    color: var(--text-primary);
  }

  /* History Modal Timeline */
  .history-modal-body {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    max-height: 400px;
    overflow-y: auto;
    padding-right: 0.25rem;
  }

  .timeline-container {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    position: relative;
    padding-left: 1.25rem;
    border-left: 2px solid var(--border-medium);
    margin-left: 0.5rem;
  }

  .timeline-item {
    position: relative;
  }

  .timeline-dot {
    position: absolute;
    left: -1.65rem;
    top: 0.35rem;
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--accent-primary);
    border: 2px solid var(--bg-primary);
  }

  .timeline-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    padding: 0.75rem 0.95rem;
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .timeline-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .version-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .version-badge-hist {
    font-size: 0.7rem;
    font-weight: 800;
    background: rgba(36, 131, 123, 0.15);
    color: #24837b;
    padding: 0.1rem 0.4rem;
    border-radius: var(--radius-sm);
  }

  .hist-saved-at {
    font-size: 0.72rem;
    color: var(--text-muted);
  }

  .hist-note {
    font-size: 0.72rem;
    color: var(--text-muted);
    font-style: italic;
  }

  .hist-rule-snapshot {
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 0.5rem 0.75rem;
    font-size: 0.76rem;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    color: var(--text-primary);
  }

  .empty-history-box {
    text-align: center;
    padding: 2rem 1rem;
    color: var(--text-muted);
    font-size: 0.85rem;
  }

  .mini-loading {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    padding: 2rem;
    font-size: 0.84rem;
    color: var(--text-muted);
  }

  .modal-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: 0.5rem;
  }

  .primary-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    background: var(--accent-primary);
    color: #fff;
    border: none;
    border-radius: var(--radius-md);
    padding: 0.55rem 1.1rem;
    font-weight: 600;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .secondary-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    background: var(--bg-secondary);
    color: var(--text-primary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-md);
    padding: 0.55rem 1rem;
    font-weight: 500;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .warning-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    background: rgba(218, 112, 44, 0.12);
    color: #da702c;
    border: 1px solid rgba(218, 112, 44, 0.4);
    border-radius: var(--radius-md);
    padding: 0.55rem 1rem;
    font-weight: 600;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .warning-btn:hover {
    background: rgba(218, 112, 44, 0.22);
  }

  .icon-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 0.25rem;
    border-radius: var(--radius-sm);
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .icon-btn:hover {
    background: var(--bg-tertiary);
    color: var(--text-primary);
  }

  .icon-btn.text-danger:hover {
    color: #d14d41;
  }

  /* Toggle Switch */
  .toggle-switch {
    position: relative;
    display: inline-block;
    width: 32px;
    height: 18px;
    cursor: pointer;
  }

  .toggle-switch input {
    opacity: 0;
    width: 0;
    height: 0;
  }

  .slider {
    position: absolute;
    top: 0; left: 0; right: 0; bottom: 0;
    background-color: var(--bg-tertiary);
    border: 1px solid var(--border-medium);
    transition: 150ms;
    border-radius: 18px;
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
    background-color: #24837b;
    border-color: #24837b;
  }

  input:checked + .slider:before {
    transform: translateX(14px);
    background-color: #fff;
  }

  .spinner {
    width: 24px;
    height: 24px;
    border: 2px solid var(--border-medium);
    border-top-color: var(--accent-primary);
    border-radius: 50%;
    animation: spin 600ms linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
