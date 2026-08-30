<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from "svelte-i18n";
  import {
    classifiedFiles,
    currentSessionId,
    selectedFolder,
    alsoRenameInOrganization,
    showToast,
  } from "../lib/stores";
  import {
    applyOrganization,
    undoLastApply,
    recordUserCorrection,
    listCategories,
    createCategory,
    openInExplorer,
    openWithDefaultApp,
    getFilePreview,
    suggestSemanticNames,
    type Category,
    type ClassifiedFile,
    type FileMove,
    type FilePreviewData,
    type RenameConfig,
    type FileRenameCandidate,
  } from "../lib/api";
  import FileTreeNode, { type TreeNodeData } from "../lib/FileTreeNode.svelte";
  import FilePreviewModal from "../lib/FilePreviewModal.svelte";

  let searchQuery = "";
  let isApplying = false;
  let isUndoing = false;
  let showConfirmModal = false;

  // Tag & Category Modals State
  let showTagModal = false;
  let showCategoryModal = false;
  let modalSearchQuery = "";
  let modalActiveTab: "all" | "manual" | "auto" = "all";
  let modalNewNameInput = "";
  let modalNewColorInput = "#3b82f6";

  // Rename Modal State
  let showRenameModal = false;
  let renamingTargetFile: ClassifiedFile | null = null;
  let renameModalInput = "";
  let proposedNamesMap = new Map<string, string>(); // file_id -> suggested_name
  let userCustomNamesMap = new Map<string, string>(); // file_id -> custom_name

  // Highlighted destination node state
  let highlightedAfterNodeId: string | null = null;

  // File Preview Modal State
  let showFilePreviewModal = false;
  let previewLoading = false;
  let filePreviewData: FilePreviewData | null = null;
  let activePreviewFile: ClassifiedFile | null = null;

  let allCategories: Category[] = [];
  let ignoredFileIds = new Set<string>();

  // Tree expansion state (Set of expanded folder IDs)
  let expandedBeforeIds = new Set<string>();
  let expandedAfterIds = new Set<string>();

  // Context Menu State
  let contextMenu = {
    visible: false,
    x: 0,
    y: 0,
    file: null as ClassifiedFile | null,
    folder: null as TreeNodeData | null,
  };

  function handleShowInOrganizedPreview(file: ClassifiedFile) {
    closeContextMenu();
    const catPath = file.suggested_category || "Outros";
    const segments = catPath.replace(/\\/g, "/").split("/").map((s) => s.trim()).filter(Boolean);

    expandedAfterIds.add("after-root");
    let currentRel = "";
    for (const seg of segments) {
      currentRel = currentRel ? `${currentRel}/${seg}` : seg;
      expandedAfterIds.add(`after-dir-${currentRel}`);
    }
    expandedAfterIds = new Set(expandedAfterIds);

    const targetId = `after-file-${file.file_id}`;
    highlightedAfterNodeId = targetId;

    setTimeout(() => {
      const el = document.getElementById(targetId);
      if (el) {
        el.scrollIntoView({ behavior: "smooth", block: "center" });
      }
    }, 60);

    showToast(`Arquivo localizado na pasta '${catPath}'`, "info");

    setTimeout(() => {
      if (highlightedAfterNodeId === targetId) {
        highlightedAfterNodeId = null;
      }
    }, 4000);
  }

  function handleShowFolderInOrganizedPreview(folder: TreeNodeData) {
    closeContextMenu();
    const files = getFolderFiles(folder);
    if (files.length === 0) return;

    expandedAfterIds.add("after-root");
    for (const f of files) {
      const catPath = f.suggested_category || "Outros";
      const segments = catPath.replace(/\\/g, "/").split("/").map((s) => s.trim()).filter(Boolean);
      let currentRel = "";
      for (const seg of segments) {
        currentRel = currentRel ? `${currentRel}/${seg}` : seg;
        expandedAfterIds.add(`after-dir-${currentRel}`);
      }
    }
    expandedAfterIds = new Set(expandedAfterIds);

    const firstCat = files[0].suggested_category || "Outros";
    const firstSegments = firstCat.replace(/\\/g, "/").split("/").map((s) => s.trim()).filter(Boolean);
    const targetId = `after-dir-${firstSegments.join("/")}`;
    highlightedAfterNodeId = targetId;

    setTimeout(() => {
      const el = document.getElementById(targetId);
      if (el) {
        el.scrollIntoView({ behavior: "smooth", block: "center" });
      }
    }, 60);

    showToast(`${files.length} arquivos localizados no preview organizado`, "info");

    setTimeout(() => {
      if (highlightedAfterNodeId === targetId) {
        highlightedAfterNodeId = null;
      }
    }, 4000);
  }

  // Target file(s) for reassignment
  let targetFilesForReassign: ClassifiedFile[] = [];
  let targetReassignLabel = "";

  const presetColors = [
    "#3b82f6", "#06b6d4", "#10b981", "#84cc16",
    "#f59e0b", "#f97316", "#ef4444", "#ec4899",
    "#8b5cf6", "#6366f1", "#64748b"
  ];

  onMount(async () => {
    await reloadCategories();
    if ($alsoRenameInOrganization) {
      await loadProposedNames();
    }
  });

  async function reloadCategories() {
    try {
      allCategories = await listCategories();
    } catch (_) {}
  }

  // Quando o toggle de renomear estiver ativo ou mudar, calcula nomes semânticos
  $: if ($alsoRenameInOrganization && $classifiedFiles.length > 0 && proposedNamesMap.size === 0) {
    loadProposedNames();
  }

  async function loadProposedNames() {
    if ($classifiedFiles.length === 0) return;
    try {
      const candidates: FileRenameCandidate[] = $classifiedFiles.map((f) => ({
        file_id: f.file_id || "",
        path: f.path || "",
        filename: f.filename || "",
        category: f.suggested_category || "Outros",
        category_color: f.category_color || null,
        size_bytes: f.size_bytes || 0,
        modified_at: (f as any).modified_at || null,
        text_sample: (f as any).text_sample || null,
      }));

      const config: RenameConfig = {
        preset: "semantic",
        separator: "_",
        case_style: "title",
        date_format: "YYYY-MM",
        include_category: true,
        remove_noise: true,
        custom_template: null,
      };

      const res = await suggestSemanticNames(candidates, config);
      const map = new Map<string, string>();
      for (const s of res) {
        map.set(s.file_id, s.proposed_filename);
      }
      proposedNamesMap = map;
    } catch (_) {}
  }

  function getProposedFilename(file: ClassifiedFile): string {
    if (userCustomNamesMap.has(file.file_id)) {
      return userCustomNamesMap.get(file.file_id)!;
    }
    if ($alsoRenameInOrganization && proposedNamesMap.has(file.file_id)) {
      return proposedNamesMap.get(file.file_id)!;
    }
    return file.filename;
  }

  // Filtragem de arquivos por busca
  $: filteredFiles = $classifiedFiles.filter((f) => {
    if (ignoredFileIds.has(f.file_id)) return false;
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase();
    return (
      f.filename.toLowerCase().includes(q) ||
      f.suggested_category.toLowerCase().includes(q) ||
      f.path.toLowerCase().includes(q)
    );
  });

  // Construção da Árvore "Antes" (Estrutura Atual de Pastas do Disco)
  $: beforeTree = (() => {
    const rootPath = $selectedFolder || "";
    return buildBeforeTree(filteredFiles, rootPath);
  })();

  // Construção da Árvore "Depois" (Estrutura Proposta Organizada por Tags/Categorias + Nomes Novos se ativo)
  $: afterTree = (() => {
    const rootPath = $selectedFolder || "";
    // Trigger reatividade quando nomes propostos ou customizados mudam
    const _ = [$alsoRenameInOrganization, proposedNamesMap, userCustomNamesMap];
    return buildAfterTree(filteredFiles, rootPath, allCategories);
  })();

  let initializedSessionId: string | null = null;

  // Expandir todas as pastas inicialmente na primeira carga
  $: if ($currentSessionId !== initializedSessionId && beforeTree.length > 0 && afterTree.length > 0) {
    initializedSessionId = $currentSessionId;
    const allBefore = new Set<string>();
    collectFolderIds(beforeTree, allBefore);
    expandedBeforeIds = allBefore;

    const allAfter = new Set<string>();
    collectFolderIds(afterTree, allAfter);
    expandedAfterIds = allAfter;
  }

  function collectFolderIds(nodes: TreeNodeData[], acc: Set<string>) {
    for (const n of nodes) {
      if (n.isFolder) {
        acc.add(n.id);
        if (n.children) collectFolderIds(n.children, acc);
      }
    }
  }

  function buildBeforeTree(files: ClassifiedFile[], rootPath: string): TreeNodeData[] {
    const normalizedRoot = rootPath.replace(/\\/g, "/").replace(/\/+$/, "");
    const rootName = normalizedRoot ? normalizedRoot.split("/").pop() || "Pasta Raiz" : "Pasta Raiz";

    const rootNode: TreeNodeData = {
      id: "before-root",
      name: rootName,
      isFolder: true,
      fullPath: rootPath,
      children: [],
      fileCount: 0,
    };

    const folderMap = new Map<string, TreeNodeData>();
    folderMap.set("", rootNode);

    for (const file of files) {
      const normalizedFilePath = file.path.replace(/\\/g, "/");
      let relPath = "";
      if (normalizedFilePath.startsWith(normalizedRoot)) {
        relPath = normalizedFilePath.substring(normalizedRoot.length).replace(/^\/+/, "");
      } else {
        relPath = file.filename;
      }

      const parts = relPath.split("/");
      parts.pop(); // Remove o nome do arquivo, sobrando as pastas intermediárias

      let currentRel = "";
      let parentNode = rootNode;

      for (const segment of parts) {
        currentRel = currentRel ? `${currentRel}/${segment}` : segment;
        if (!folderMap.has(currentRel)) {
          const newFolder: TreeNodeData = {
            id: `before-dir-${currentRel}`,
            name: segment,
            isFolder: true,
            fullPath: `${normalizedRoot}/${currentRel}`,
            isPreservedFolder: file.is_already_organized,
            children: [],
            fileCount: 0,
          };
          parentNode.children = parentNode.children || [];
          parentNode.children.push(newFolder);
          folderMap.set(currentRel, newFolder);
        } else if (!file.is_already_organized) {
          const existingFolder = folderMap.get(currentRel)!;
          existingFolder.isPreservedFolder = existingFolder.isPreservedFolder && file.is_already_organized;
        }
        parentNode = folderMap.get(currentRel)!;
      }

      const fileNode: TreeNodeData = {
        id: `before-file-${file.file_id}`,
        name: file.filename,
        isFolder: false,
        fullPath: file.path,
        file,
        fileCount: 1,
        isPreservedFile: file.is_already_organized,
      };
      parentNode.children = parentNode.children || [];
      parentNode.children.push(fileNode);
    }

    function finalizeNode(node: TreeNodeData): number {
      if (!node.isFolder) return 1;
      let count = 0;
      if (node.children) {
        for (const child of node.children) {
          count += finalizeNode(child);
        }
        node.children.sort((a, b) => {
          if (a.isFolder && !b.isFolder) return -1;
          if (!a.isFolder && b.isFolder) return 1;
          return a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: "base" });
        });
      }
      node.fileCount = count;
      return count;
    }

    finalizeNode(rootNode);

    return [rootNode];
  }

  function buildAfterTree(files: ClassifiedFile[], rootPath: string, categories: Category[]): TreeNodeData[] {
    const normalizedRoot = rootPath.replace(/\\/g, "/").replace(/\/+$/, "");
    const rootName = normalizedRoot ? normalizedRoot.split("/").pop() || "Pasta Raiz" : "Pasta Raiz";

    const catColorMap = new Map<string, string>();
    for (const c of categories) {
      if (c.color) catColorMap.set(c.name.toLowerCase(), c.color);
    }

    const rootNode: TreeNodeData = {
      id: "after-root",
      name: `${rootName} (Organizada)`,
      isFolder: true,
      fullPath: rootPath,
      children: [],
      fileCount: 0,
    };

    const folderMap = new Map<string, TreeNodeData>();
    folderMap.set("", rootNode);

    for (const file of files) {
      const catPath = file.suggested_category || "Outros";
      const catColor = file.category_color || catColorMap.get(catPath.toLowerCase()) || "#3b82f6";
      const segments = catPath.replace(/\\/g, "/").split("/").map((s) => s.trim()).filter(Boolean);

      let currentRel = "";
      let parentNode = rootNode;

      for (let i = 0; i < segments.length; i++) {
        const segment = segments[i];
        currentRel = currentRel ? `${currentRel}/${segment}` : segment;
        const folderId = `after-dir-${currentRel}`;

        if (!folderMap.has(currentRel)) {
          const newFolder: TreeNodeData = {
            id: folderId,
            name: segment,
            isFolder: true,
            fullPath: `${normalizedRoot}/${currentRel}`,
            categoryColor: i === 0 ? catColor : undefined,
            categoryName: currentRel,
            isPreservedFolder: file.is_already_organized,
            children: [],
            fileCount: 0,
          };
          parentNode.children = parentNode.children || [];
          parentNode.children.push(newFolder);
          folderMap.set(currentRel, newFolder);
        } else if (!file.is_already_organized) {
          const existingFolder = folderMap.get(currentRel)!;
          existingFolder.isPreservedFolder = existingFolder.isPreservedFolder && file.is_already_organized;
        }

        parentNode = folderMap.get(currentRel)!;
      }

      const finalFilename = getProposedFilename(file);
      const fileRelPath = `${currentRel}/${finalFilename}`;

      const fileNode: TreeNodeData = {
        id: `after-file-${file.file_id}`,
        name: finalFilename,
        isFolder: false,
        fullPath: `${normalizedRoot}/${fileRelPath}`,
        file,
        fileCount: 1,
        isPreservedFile: file.is_already_organized,
      };
      parentNode.children = parentNode.children || [];
      parentNode.children.push(fileNode);
    }

    function finalizeNode(node: TreeNodeData): number {
      if (!node.isFolder) return 1;
      let count = 0;
      if (node.children) {
        for (const child of node.children) {
          count += finalizeNode(child);
        }
        node.children.sort((a, b) => {
          if (a.isFolder && !b.isFolder) return -1;
          if (!a.isFolder && b.isFolder) return 1;
          return a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: "base" });
        });
      }
      node.fileCount = count;
      return count;
    }

    finalizeNode(rootNode);

    return [rootNode];
  }

  $: modalFilteredCategories = allCategories.filter((c) => {
    if (modalActiveTab === "manual" && c.created_by !== "user") return false;
    if (modalActiveTab === "auto" && c.created_by !== "auto") return false;
    if (!modalSearchQuery.trim()) return true;
    return c.name.toLowerCase().includes(modalSearchQuery.toLowerCase());
  });

  function toggleBeforeFolder(id: string) {
    if (expandedBeforeIds.has(id)) {
      expandedBeforeIds.delete(id);
    } else {
      expandedBeforeIds.add(id);
    }
    expandedBeforeIds = new Set(expandedBeforeIds);
  }

  function toggleAfterFolder(id: string) {
    if (expandedAfterIds.has(id)) {
      expandedAfterIds.delete(id);
    } else {
      expandedAfterIds.add(id);
    }
    expandedAfterIds = new Set(expandedAfterIds);
  }

  function expandAllBefore() {
    const all = new Set<string>();
    collectFolderIds(beforeTree, all);
    expandedBeforeIds = all;
  }

  function collapseAllBefore() {
    expandedBeforeIds = new Set();
  }

  function expandAllAfter() {
    const all = new Set<string>();
    collectFolderIds(afterTree, all);
    expandedAfterIds = all;
  }

  function collapseAllAfter() {
    expandedAfterIds = new Set();
  }

  // Extrair todos os arquivos de um nó ou pasta
  function getFolderFiles(folder: TreeNodeData): ClassifiedFile[] {
    const files: ClassifiedFile[] = [];
    function traverse(node: TreeNodeData) {
      if (!node.isFolder && node.file) {
        files.push(node.file);
      } else if (node.children) {
        for (const child of node.children) {
          traverse(child);
        }
      }
    }
    traverse(folder);
    return files;
  }

  // Handlers do menu de clique direito e cálculo de posicionamento (Viewport Clamping)
  function clampContextMenu(node: HTMLElement, coords: { x: number; y: number }) {
    function position(pos: { x: number; y: number }) {
      requestAnimationFrame(() => {
        const winWidth = window.innerWidth;
        const winHeight = window.innerHeight;
        const rect = node.getBoundingClientRect();

        let newTop = pos.y;
        let newLeft = pos.x;

        // Se o menu ultrapassar a borda inferior da janela, ajusta para cima
        if (newTop + rect.height > winHeight - 12) {
          newTop = Math.max(12, winHeight - rect.height - 12);
        }

        // Se o menu ultrapassar a borda direita da janela, ajusta para a esquerda
        if (newLeft + rect.width > winWidth - 12) {
          newLeft = Math.max(12, winWidth - rect.width - 12);
        }

        // Limites mínimos de segurança
        if (newTop < 12) newTop = 12;
        if (newLeft < 12) newLeft = 12;

        node.style.top = `${newTop}px`;
        node.style.left = `${newLeft}px`;
        node.style.maxHeight = `${winHeight - 24}px`;
      });
    }

    position(coords);

    return {
      update(newCoords: { x: number; y: number }) {
        position(newCoords);
      },
    };
  }

  function openFileContextMenu(e: MouseEvent, file: ClassifiedFile) {
    contextMenu = {
      visible: true,
      x: e.clientX,
      y: e.clientY,
      file,
      folder: null,
    };
  }

  function openFolderContextMenu(e: MouseEvent, folder: TreeNodeData) {
    contextMenu = {
      visible: true,
      x: e.clientX,
      y: e.clientY,
      file: null,
      folder,
    };
  }

  function closeContextMenu() {
    contextMenu.visible = false;
  }

  // Ações do menu de contexto
  async function handleOpenFilePreview(file: ClassifiedFile) {
    closeContextMenu();
    activePreviewFile = file;
    previewLoading = true;
    showFilePreviewModal = true;
    filePreviewData = null;
    try {
      filePreviewData = await getFilePreview(file.path);
    } catch (err: any) {
      showToast("Erro ao carregar pré-visualização: " + err, "error");
    } finally {
      previewLoading = false;
    }
  }

  async function handleOpenWithDefaultApp(path: string) {
    closeContextMenu();
    try {
      await openWithDefaultApp(path);
    } catch (err: any) {
      showToast("Erro ao abrir com aplicativo padrão: " + err, "error");
    }
  }

  async function handleOpenInExplorer(path: string) {
    closeContextMenu();
    try {
      await openInExplorer(path);
    } catch (err: any) {
      showToast("Erro ao abrir no Explorador: " + err, "error");
    }
  }

  function handleOpenRenameModal(file: ClassifiedFile) {
    renamingTargetFile = file;
    renameModalInput = getProposedFilename(file);
    closeContextMenu();
    showRenameModal = true;
  }

  function handleSaveRenameModal() {
    if (!renamingTargetFile || !renameModalInput.trim()) return;
    const clean = renameModalInput.trim();
    userCustomNamesMap.set(renamingTargetFile.file_id, clean);
    userCustomNamesMap = new Map(userCustomNamesMap);
    showRenameModal = false;
    showToast(`Nome atualizado para '${clean}'`, "success");
  }

  function handleOpenChangeTag(file: ClassifiedFile) {
    targetFilesForReassign = [file];
    targetReassignLabel = file.filename;
    modalSearchQuery = "";
    modalActiveTab = "all";
    modalNewNameInput = "";
    modalNewColorInput = presetColors[Math.floor(Math.random() * presetColors.length)];
    closeContextMenu();
    showTagModal = true;
  }

  function handleOpenChangeFolderTag(folder: TreeNodeData) {
    const files = getFolderFiles(folder);
    if (files.length === 0) {
      showToast("Nenhum arquivo encontrado nesta pasta.", "info");
      closeContextMenu();
      return;
    }
    targetFilesForReassign = files;
    targetReassignLabel = `${folder.name} (${files.length} ${files.length === 1 ? "arquivo" : "arquivos"})`;
    modalSearchQuery = "";
    modalActiveTab = "all";
    modalNewNameInput = "";
    modalNewColorInput = presetColors[Math.floor(Math.random() * presetColors.length)];
    closeContextMenu();
    showTagModal = true;
  }

  function handleOpenChangeCategory(file: ClassifiedFile) {
    targetFilesForReassign = [file];
    targetReassignLabel = file.filename;
    modalSearchQuery = "";
    modalActiveTab = "all";
    modalNewNameInput = "";
    modalNewColorInput = presetColors[Math.floor(Math.random() * presetColors.length)];
    closeContextMenu();
    showCategoryModal = true;
  }

  function handleOpenChangeFolderCategory(folder: TreeNodeData) {
    const files = getFolderFiles(folder);
    if (files.length === 0) {
      showToast("Nenhum arquivo encontrado nesta pasta.", "info");
      closeContextMenu();
      return;
    }
    targetFilesForReassign = files;
    targetReassignLabel = `${folder.name} (${files.length} ${files.length === 1 ? "arquivo" : "arquivos"})`;
    modalSearchQuery = "";
    modalActiveTab = "all";
    modalNewNameInput = "";
    modalNewColorInput = presetColors[Math.floor(Math.random() * presetColors.length)];
    closeContextMenu();
    showCategoryModal = true;
  }

  function handleIgnoreFile(file: ClassifiedFile) {
    ignoredFileIds.add(file.file_id);
    ignoredFileIds = new Set(ignoredFileIds);
    closeContextMenu();
    showToast(`Arquivo '${file.filename}' ignorado da organização.`, "info");
  }

  function handleIgnoreFolder(folder: TreeNodeData) {
    const files = getFolderFiles(folder);
    if (files.length === 0) return;
    for (const f of files) {
      ignoredFileIds.add(f.file_id);
    }
    ignoredFileIds = new Set(ignoredFileIds);
    closeContextMenu();
    showToast(`${files.length} arquivos da pasta '${folder.name}' ignorados da organização.`, "info");
  }

  function handleKeepPreserved(files: ClassifiedFile[]) {
    closeContextMenu();
    showToast(`Estrutura original de ${files.length} arquivo(s) mantida.`, "info");
  }

  function handleReorganizeWithAI(files: ClassifiedFile[]) {
    const targetFileIds = new Set(files.map((f) => f.file_id));
    classifiedFiles.update((list) =>
      list.map((item) =>
        targetFileIds.has(item.file_id)
          ? {
              ...item,
              is_already_organized: false,
            }
          : item
      )
    );
    closeContextMenu();
    showToast(`${files.length} arquivo(s) marcados para reorganização.`, "success");
  }

  async function handleAlwaysRule(file: ClassifiedFile) {
    closeContextMenu();
    try {
      await recordUserCorrection(file.file_id, file.category_id, file.category_id);
      showToast($_("preview.toast.rule_saved"), "success");
    } catch (err: any) {
      showToast("Erro ao gravar regra permanente: " + err, "error");
    }
  }

  async function assignCategoryToTarget(category: Category) {
    if (targetFilesForReassign.length === 0) return;
    const files = [...targetFilesForReassign];
    const targetFileIds = new Set(files.map((f) => f.file_id));

    try {
      for (const file of files) {
        await recordUserCorrection(file.file_id, file.category_id, category.id);
      }

      classifiedFiles.update((list) =>
        list.map((item) =>
          targetFileIds.has(item.file_id)
            ? {
                ...item,
                suggested_category: category.name,
                category_id: category.id,
                category_color: category.color ?? "#3b82f6",
                confidence: 1.0,
                tier_used: 1,
              }
            : item
        )
      );

      showTagModal = false;
      showCategoryModal = false;
      const count = files.length;
      if (count === 1) {
        showToast(`Arquivo '${files[0].filename}' reatribuído para '${category.name}'!`, "success");
      } else {
        showToast(`${count} arquivos reatribuídos para '${category.name}'!`, "success");
      }
      targetFilesForReassign = [];
      targetReassignLabel = "";
      await reloadCategories();
      if ($alsoRenameInOrganization) {
        await loadProposedNames();
      }
    } catch (err: any) {
      showToast("Erro ao reatribuir: " + err, "error");
    }
  }

  async function handleCreateAndAssignNew(isCategory: boolean = false) {
    if (!modalNewNameInput.trim() || targetFilesForReassign.length === 0) return;
    try {
      const newCat = await createCategory(modalNewNameInput.trim(), modalNewColorInput);
      await assignCategoryToTarget(newCat);
    } catch (err: any) {
      showToast(`Erro ao criar ${isCategory ? "categoria" : "tag"}: ` + err, "error");
    }
  }

  async function handleApplyChanges() {
    if (!$selectedFolder || !$currentSessionId) return;
    showConfirmModal = false;
    isApplying = true;

    try {
      const root = $selectedFolder;
      const moves: FileMove[] = [];

      for (const file of filteredFiles) {
        const segments = file.suggested_category
          .replace(/\\/g, "/")
          .split("/")
          .map((s) => s.replace(/[<>:"/\\|?*]/g, "_").trim())
          .filter(Boolean);
        const sep = root.includes("\\") ? "\\" : "/";
        const sanitizedRelPath = segments.join(sep);
        const finalFilename = getProposedFilename(file);
        const destPath = `${root}${sep}${sanitizedRelPath}${sep}${finalFilename}`;

        moves.push({
          file_id: file.file_id,
          from_path: file.path,
          to_path: destPath,
        });
      }

      const summary = await applyOrganization($currentSessionId, moves);
      if (summary.failed.length === 0) {
        showToast($_("preview.toast.applied", { values: { count: summary.moved } }), "success");
      } else {
        showToast(
          `${summary.moved} movidos, ${summary.failed.length} falharam: ${summary.failed[0]}`,
          "error"
        );
      }
    } catch (err: any) {
      showToast("Erro ao aplicar organização: " + err, "error");
    } finally {
      isApplying = false;
    }
  }

  async function handleUndo() {
    isUndoing = true;
    try {
      const count = await undoLastApply($currentSessionId);
      if (count > 0) {
        showToast($_("preview.toast.undone", { values: { count } }), "success");
      } else {
        showToast("Nenhuma operação anterior para desfazer.", "info");
      }
    } catch (err: any) {
      showToast("Erro ao desfazer: " + err, "error");
    } finally {
      isUndoing = false;
    }
  }
</script>

<svelte:window on:click={closeContextMenu} />

<div class="preview-layout">
  <!-- Top Action Bar -->
  <div class="preview-action-bar">
    <div class="search-box">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"></circle>
        <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
      </svg>
      <input
        type="text"
        placeholder={$_("preview.search_placeholder")}
        bind:value={searchQuery}
      />
      {#if searchQuery}
        <button class="clear-search" on:click={() => (searchQuery = "")}>✕</button>
      {/if}
    </div>

    <!-- Toggle de Renomear junto na Organização -->
    <label class="preview-rename-toggle" title="Se ativado, sugere novos nomes inteligentes para os arquivos na árvore de destino">
      <input type="checkbox" bind:checked={$alsoRenameInOrganization} />
      <span>Sugerir novos nomes na organização</span>
    </label>

    <div class="action-buttons">
      <button
        class="secondary-btn"
        disabled={isUndoing || isApplying}
        on:click={handleUndo}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M3 7v6h6"></path>
          <path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13"></path>
        </svg>
        {isUndoing ? $_("preview.undoing") : $_("preview.undo")}
      </button>

      <button
        class="primary-btn"
        disabled={isApplying || filteredFiles.length === 0}
        on:click={() => (showConfirmModal = true)}
      >
        {#if isApplying}
          <div class="mini-spinner"></div>
          {$_("preview.applying")}
        {:else}
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <polyline points="20 6 9 17 4 12"></polyline>
          </svg>
          {$_("preview.apply")} ({filteredFiles.length})
        {/if}
      </button>
    </div>
  </div>

  <!-- Side-by-Side Dual Tree View -->
  <div class="trees-container">
    <!-- Left Column: Tree Antes (Estrutura Atual) -->
    <section class="preview-column glass-panel current-column">
      <div class="column-header">
        <div class="column-title">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
          </svg>
          <h2>{$_("preview.column.current")}</h2>
        </div>
        <div class="column-meta-actions">
          <span class="column-meta">{filteredFiles.length} {$_("preview.total_files")}</span>
          <div class="tree-controls">
            <button class="mini-btn" on:click={expandAllBefore} title="Expandir todas as pastas">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="7 13 12 18 17 13"></polyline>
                <polyline points="7 6 12 11 17 6"></polyline>
              </svg>
            </button>
            <button class="mini-btn" on:click={collapseAllBefore} title="Recolher todas">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="17 11 12 6 7 11"></polyline>
                <polyline points="17 18 12 13 7 18"></polyline>
              </svg>
            </button>
          </div>
        </div>
      </div>

      <div class="tree-scroll-container">
        {#if beforeTree.length === 0 || filteredFiles.length === 0}
          <div class="empty-state">Nenhum arquivo ou pasta encontrado com os filtros atuais.</div>
        {:else}
          <div class="tree-root">
            {#each beforeTree as node (node.id)}
              <FileTreeNode
                {node}
                expandedIds={expandedBeforeIds}
                onToggleFolder={toggleBeforeFolder}
                onFileContextMenu={openFileContextMenu}
                onFolderContextMenu={openFolderContextMenu}
              />
            {/each}
          </div>
        {/if}
      </div>
    </section>

    <!-- Right Column: Tree Depois (Estrutura Proposta com nomes sugeridos) -->
    <section class="preview-column glass-panel proposed-column">
      <div class="column-header">
        <div class="column-title">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"></path>
            <line x1="7" y1="7" x2="7.01" y2="7"></line>
          </svg>
          <h2>{$_("preview.column.proposed")}</h2>
        </div>
        <div class="column-meta-actions">
          <span class="column-meta">{filteredFiles.length} {$_("preview.total_files")}</span>
          <div class="tree-controls">
            <button class="mini-btn" on:click={expandAllAfter} title="Expandir todas as pastas">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="7 13 12 18 17 13"></polyline>
                <polyline points="7 6 12 11 17 6"></polyline>
              </svg>
            </button>
            <button class="mini-btn" on:click={collapseAllAfter} title="Recolher todas">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                <polyline points="17 11 12 6 7 11"></polyline>
                <polyline points="17 18 12 13 7 18"></polyline>
              </svg>
            </button>
          </div>
        </div>
      </div>

      <div class="tree-scroll-container">
        {#if afterTree.length === 0 || filteredFiles.length === 0}
          <div class="empty-state">Nenhuma categoria agrupada com os filtros atuais.</div>
        {:else}
          <div class="tree-root">
            {#each afterTree as node (node.id)}
              <FileTreeNode
                {node}
                expandedIds={expandedAfterIds}
                highlightedNodeId={highlightedAfterNodeId}
                onToggleFolder={toggleAfterFolder}
                onFileContextMenu={openFileContextMenu}
                onFolderContextMenu={openFolderContextMenu}
              />
            {/each}
          </div>
        {/if}
      </div>
    </section>
  </div>
</div>

<!-- Context Menu on Right Click (Com opção Renomear e Mostrar no Preview) -->
{#if contextMenu.visible}
  <div
    class="custom-context-menu"
    use:clampContextMenu={{ x: contextMenu.x, y: contextMenu.y }}
    role="menu"
    tabindex="-1"
    on:click|stopPropagation
    on:keydown|stopPropagation
  >
    {#if contextMenu.file}
      <!-- Header do Arquivo com Tag em Destaque -->
      <div class="context-card-header">
        <div class="context-file-title truncate" title={contextMenu.file.filename}>
          {contextMenu.file.filename}
        </div>
        <div class="context-file-path truncate" title={contextMenu.file.path}>
          {contextMenu.file.path}
        </div>

        <!-- Tag / Categoria Atual do Arquivo -->
        <div
          class="context-tag-pill"
          style="background: {contextMenu.file.category_color || '#3b82f6'}18; border-color: {contextMenu.file.category_color || '#3b82f6'}40;"
        >
          <span class="cat-dot" style="background: {contextMenu.file.category_color || '#3b82f6'};"></span>
          <span class="context-tag-name truncate" style="color: {contextMenu.file.category_color || '#3b82f6'};">
            {contextMenu.file.suggested_category}
          </span>
          <span class="context-confidence">
            {Math.round(contextMenu.file.confidence * 100)}%
          </span>
        </div>
      </div>

      <div class="context-divider"></div>

      <!-- Ações Especiais para Itens Preservados -->
      {#if contextMenu.file.is_already_organized}
        <button
          class="context-item"
          role="menuitem"
          on:click={() => handleKeepPreserved([contextMenu.file!])}
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="#10b981" stroke-width="2">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
          </svg>
          {$_("preview.context.keep_preserved")}
        </button>
        <button
          class="context-item"
          role="menuitem"
          on:click={() => handleReorganizeWithAI([contextMenu.file!])}
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="#3b82f6" stroke-width="2">
            <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
          </svg>
          {$_("preview.context.reorganize_ai")}
        </button>
        <div class="context-divider"></div>
      {/if}

      <!-- 1. Visualizar Conteúdo (Nativo) -->
      <button
        class="context-item highlight-action"
        role="menuitem"
        on:click={() => handleOpenFilePreview(contextMenu.file!)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"></path>
          <circle cx="12" cy="12" r="3"></circle>
        </svg>
        {$_("preview.context.preview")}
      </button>

      <!-- 2. Abrir no Aplicativo Padrão do Windows -->
      <button
        class="context-item"
        role="menuitem"
        on:click={() => handleOpenWithDefaultApp(contextMenu.file!.path)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
          <polyline points="15 3 21 3 21 9"></polyline>
          <line x1="10" y1="14" x2="21" y2="3"></line>
        </svg>
        Abrir no App Padrão
      </button>

      <!-- 3. Mostrar no Preview Organizado -->
      <button
        class="context-item"
        role="menuitem"
        on:click={() => handleShowInOrganizedPreview(contextMenu.file!)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="13 17 18 12 13 7"></polyline>
          <polyline points="6 17 11 12 6 7"></polyline>
        </svg>
        Mostrar no preview organizado
      </button>

      <!-- 4. Renomear Arquivo -->
      <button
        class="context-item"
        role="menuitem"
        on:click={() => handleOpenRenameModal(contextMenu.file!)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
          <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
        </svg>
        Renomear
      </button>

      <!-- 5. Abrir no Explorador -->
      <button
        class="context-item"
        role="menuitem"
        on:click={() => handleOpenInExplorer(contextMenu.file!.path)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
        </svg>
        {$_("preview.context.open_explorer")}
      </button>

      <div class="context-divider"></div>

      <!-- 5. Trocar / Criar Tag -->
      <button
        class="context-item"
        role="menuitem"
        on:click={() => handleOpenChangeTag(contextMenu.file!)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"></path>
          <line x1="7" y1="7" x2="7.01" y2="7"></line>
        </svg>
        {$_("preview.context.change_tag")}
      </button>

      <!-- 6. Trocar / Criar Categoria -->
      <button
        class="context-item"
        role="menuitem"
        on:click={() => handleOpenChangeCategory(contextMenu.file!)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
          <polyline points="2 10 22 10"></polyline>
        </svg>
        {$_("preview.context.change_category")}
      </button>

      <!-- 7. Regra Permanente -->
      <button
        class="context-item"
        role="menuitem"
        on:click={() => handleAlwaysRule(contextMenu.file!)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"></path>
        </svg>
        {$_("preview.context.always_rule")}
      </button>

      <div class="context-divider"></div>

      <!-- 8. Ignorar arquivo -->
      <button
        class="context-item text-danger"
        role="menuitem"
        on:click={() => handleIgnoreFile(contextMenu.file!)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"></circle>
          <line x1="4.93" y1="4.93" x2="19.07" y2="19.07"></line>
        </svg>
        {$_("preview.context.ignore")}
      </button>

    {:else if contextMenu.folder}
      <!-- Header da Pasta -->
      <div class="context-card-header">
        <div class="context-file-title truncate" title={contextMenu.folder.name}>
          {contextMenu.folder.name}
        </div>
        <div class="context-file-path truncate">
          {contextMenu.folder.fileCount} {contextMenu.folder.fileCount === 1 ? 'arquivo' : 'arquivos'} nesta pasta
        </div>
        {#if contextMenu.folder.categoryName}
          <div
            class="context-tag-pill"
            style="background: {contextMenu.folder.categoryColor || '#3b82f6'}18; border-color: {contextMenu.folder.categoryColor || '#3b82f6'}40;"
          >
            <span class="cat-dot" style="background: {contextMenu.folder.categoryColor || '#3b82f6'};"></span>
            <span class="context-tag-name truncate" style="color: {contextMenu.folder.categoryColor || '#3b82f6'};">
              {contextMenu.folder.categoryName}
            </span>
          </div>
        {/if}
      </div>

      <div class="context-divider"></div>

      <!-- Ações Especiais para Pastas Preservadas -->
      {#if contextMenu.folder.isPreservedFolder}
        <button
          class="context-item"
          role="menuitem"
          on:click={() => handleKeepPreserved(getFolderFiles(contextMenu.folder!))}
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="#10b981" stroke-width="2">
            <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
          </svg>
          {$_("preview.context.keep_preserved")}
        </button>
        <button
          class="context-item"
          role="menuitem"
          on:click={() => handleReorganizeWithAI(getFolderFiles(contextMenu.folder!))}
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="#3b82f6" stroke-width="2">
            <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon>
          </svg>
          {$_("preview.context.reorganize_ai")}
        </button>
        <div class="context-divider"></div>
      {/if}

      <!-- 1. Mostrar no Preview Organizado -->
      <button
        class="context-item highlight-action"
        role="menuitem"
        on:click={() => handleShowFolderInOrganizedPreview(contextMenu.folder!)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="13 17 18 12 13 7"></polyline>
          <polyline points="6 17 11 12 6 7"></polyline>
        </svg>
        Mostrar no preview organizado
      </button>

      <!-- 2. Abrir no Explorador -->
      <button
        class="context-item"
        role="menuitem"
        on:click={() => handleOpenInExplorer(contextMenu.folder!.fullPath)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
        </svg>
        {$_("preview.context.open_explorer")}
      </button>

      <div class="context-divider"></div>

      <!-- 2. Trocar / Criar Tag da Pasta -->
      <button
        class="context-item"
        role="menuitem"
        on:click={() => handleOpenChangeFolderTag(contextMenu.folder!)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z"></path>
          <line x1="7" y1="7" x2="7.01" y2="7"></line>
        </svg>
        {$_("preview.context.change_folder_tag")}
      </button>

      <!-- 3. Trocar / Criar Categoria da Pasta -->
      <button
        class="context-item"
        role="menuitem"
        on:click={() => handleOpenChangeFolderCategory(contextMenu.folder!)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
          <polyline points="2 10 22 10"></polyline>
        </svg>
        {$_("preview.context.change_folder_category")}
      </button>

      <!-- 4. Abrir Pasta no Explorador -->
      <button
        class="context-item"
        role="menuitem"
        on:click={() => handleOpenInExplorer(contextMenu.folder!.fullPath)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
        </svg>
        {$_("preview.context.open_explorer")}
      </button>

      <!-- 5. Alternar expansão -->
      <button
        class="context-item"
        role="menuitem"
        on:click={() => {
          if (contextMenu.folder) {
            if (contextMenu.folder.id.startsWith("before-")) {
              toggleBeforeFolder(contextMenu.folder.id);
            } else {
              toggleAfterFolder(contextMenu.folder.id);
            }
          }
          closeContextMenu();
        }}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="9 18 15 12 9 6"></polyline>
        </svg>
        {$_("preview.context.toggle_expand")}
      </button>

      <div class="context-divider"></div>

      <!-- 5. Ignorar pasta -->
      <button
        class="context-item text-danger"
        role="menuitem"
        on:click={() => handleIgnoreFolder(contextMenu.folder!)}
      >
        <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"></circle>
          <line x1="4.93" y1="4.93" x2="19.07" y2="19.07"></line>
        </svg>
        {$_("preview.context.ignore_folder")}
      </button>
    {/if}
  </div>
{/if}

<!-- Modal: Renomear Arquivo no Preview -->
{#if showRenameModal && renamingTargetFile}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (showRenameModal = false)}
    on:keydown={(e) => e.key === "Escape" && (showRenameModal = false)}
  >
    <div class="modal-card">
      <h2>Renomear Arquivo</h2>
      <p class="modal-subtitle">Nome original: <strong>{renamingTargetFile.filename}</strong></p>

      <input
        type="text"
        bind:value={renameModalInput}
        class="text-input"
        placeholder="Digite o novo nome para o arquivo..."
        on:keydown={(e) => e.key === "Enter" && handleSaveRenameModal()}
      />

      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showRenameModal = false)}>
          Cancelar
        </button>
        <button class="primary-btn" on:click={handleSaveRenameModal}>
          Salvar Nome
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal: Confirmar Aplicação -->
{#if showConfirmModal}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (showConfirmModal = false)}
    on:keydown={(e) => e.key === "Escape" && (showConfirmModal = false)}
  >
    <div class="modal-card">
      <h2>{$_("preview.modal.apply_title")}</h2>
      <p class="modal-subtitle">{$_("preview.modal.apply_msg", { values: { count: filteredFiles.length } })}</p>

      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showConfirmModal = false)}>
          {$_("preview.modal.cancel")}
        </button>
        <button class="primary-btn" on:click={handleApplyChanges}>
          {$_("preview.modal.confirm")}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal Amplo: Trocar / Criar Tag -->
{#if showTagModal}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (showTagModal = false)}
    on:keydown={(e) => e.key === "Escape" && (showTagModal = false)}
  >
    <div class="modal-card modal-card-large">
      <div class="modal-header-row">
        <div>
          <h2>{$_("preview.modal.tag_title")}</h2>
          <p class="modal-subtitle">Selecione ou crie uma tag para <strong>{targetReassignLabel}</strong>:</p>
        </div>
        <button class="close-btn" on:click={() => (showTagModal = false)}>✕</button>
      </div>

      <!-- Barra de Criação Rápida de Nova Tag -->
      <div class="create-quick-box glass-panel">
        <div class="create-input-group">
          <input
            type="text"
            placeholder="Nome da nova tag..."
            bind:value={modalNewNameInput}
            class="text-input no-margin"
            on:keydown={(e) => e.key === "Enter" && handleCreateAndAssignNew(false)}
          />
          <div class="palette-mini-dots">
            {#each presetColors as col}
              <button
                class="palette-dot"
                class:selected={modalNewColorInput === col}
                style="background: {col}"
                title="Cor"
                on:click={() => (modalNewColorInput = col)}
              ></button>
            {/each}
          </div>
        </div>
        <button
          class="primary-btn"
          disabled={!modalNewNameInput.trim()}
          on:click={() => handleCreateAndAssignNew(false)}
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
          </svg>
          Criar & Atribuir
        </button>
      </div>

      <!-- Abas e Busca de Tags Existentes -->
      <div class="modal-toolbar">
        <div class="modal-tabs">
          <button
            class="modal-tab"
            class:active={modalActiveTab === "all"}
            on:click={() => (modalActiveTab = "all")}
          >
            {$_("tags.tab.all")}
            <span class="tab-badge">{allCategories.length}</span>
          </button>
          <button
            class="modal-tab"
            class:active={modalActiveTab === "manual"}
            on:click={() => (modalActiveTab = "manual")}
          >
            {$_("tags.tab.manual")}
            <span class="tab-badge">{allCategories.filter(c => c.created_by === "user").length}</span>
          </button>
          <button
            class="modal-tab"
            class:active={modalActiveTab === "auto"}
            on:click={() => (modalActiveTab = "auto")}
          >
            {$_("tags.tab.auto")}
            <span class="tab-badge">{allCategories.filter(c => c.created_by === "auto").length}</span>
          </button>
        </div>

        <div class="search-box modal-search">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8"></circle>
            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
          </svg>
          <input
            type="text"
            placeholder={$_("preview.search_categories")}
            bind:value={modalSearchQuery}
          />
          {#if modalSearchQuery}
            <button class="clear-search" on:click={() => (modalSearchQuery = "")}>✕</button>
          {/if}
        </div>
      </div>

      <!-- Grid de Tags Existentes -->
      <div class="large-tags-grid">
        {#if modalFilteredCategories.length === 0}
          <div class="empty-picker">Nenhuma tag encontrada com os filtros atuais.</div>
        {:else}
          {#each modalFilteredCategories as tag (tag.id)}
            <button
              class="large-tag-btn glass-panel"
              style="border-left: 4px solid {tag.color ?? '#3b82f6'}"
              title="Atribuir tag '{tag.name}'"
              on:click={() => assignCategoryToTarget(tag)}
            >
              <div class="tag-card-content">
                <span class="large-tag-name">{tag.name}</span>
                <div class="large-tag-meta">
                  <span class="mini-origin-badge {tag.created_by}">
                    {tag.created_by === "auto" ? $_("tags.created_by_auto") : $_("tags.created_by_user")}
                  </span>
                  <span class="tag-file-counter">
                    {$_("tags.files_count", { values: { count: tag.file_count } })}
                  </span>
                </div>
              </div>
            </button>
          {/each}
        {/if}
      </div>

      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showTagModal = false)}>
          {$_("preview.modal.cancel")}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal Amplo: Trocar / Criar Categoria (Espelhado) -->
{#if showCategoryModal}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={() => (showCategoryModal = false)}
    on:keydown={(e) => e.key === "Escape" && (showCategoryModal = false)}
  >
    <div class="modal-card modal-card-large">
      <div class="modal-header-row">
        <div>
          <h2>{$_("preview.modal.category_title")}</h2>
          <p class="modal-subtitle">Selecione ou crie uma categoria para <strong>{targetReassignLabel}</strong>:</p>
        </div>
        <button class="close-btn" on:click={() => (showCategoryModal = false)}>✕</button>
      </div>

      <!-- Barra de Criação Rápida de Nova Categoria -->
      <div class="create-quick-box glass-panel">
        <div class="create-input-group">
          <input
            type="text"
            placeholder="Nome da nova categoria..."
            bind:value={modalNewNameInput}
            class="text-input no-margin"
            on:keydown={(e) => e.key === "Enter" && handleCreateAndAssignNew(true)}
          />
          <div class="palette-mini-dots">
            {#each presetColors as col}
              <button
                class="palette-dot"
                class:selected={modalNewColorInput === col}
                style="background: {col}"
                title="Cor"
                on:click={() => (modalNewColorInput = col)}
              ></button>
            {/each}
          </div>
        </div>
        <button
          class="primary-btn"
          disabled={!modalNewNameInput.trim()}
          on:click={() => handleCreateAndAssignNew(true)}
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="12" y1="5" x2="12" y2="19"></line>
            <line x1="5" y1="12" x2="19" y2="12"></line>
          </svg>
          Criar & Atribuir
        </button>
      </div>

      <!-- Abas e Busca de Categorias Existentes -->
      <div class="modal-toolbar">
        <div class="modal-tabs">
          <button
            class="modal-tab"
            class:active={modalActiveTab === "all"}
            on:click={() => (modalActiveTab = "all")}
          >
            {$_("categories.tab.all")}
            <span class="tab-badge">{allCategories.length}</span>
          </button>
          <button
            class="modal-tab"
            class:active={modalActiveTab === "manual"}
            on:click={() => (modalActiveTab = "manual")}
          >
            {$_("categories.tab.manual")}
            <span class="tab-badge">{allCategories.filter(c => c.created_by === "user").length}</span>
          </button>
          <button
            class="modal-tab"
            class:active={modalActiveTab === "auto"}
            on:click={() => (modalActiveTab = "auto")}
          >
            {$_("categories.tab.auto")}
            <span class="tab-badge">{allCategories.filter(c => c.created_by === "auto").length}</span>
          </button>
        </div>

        <div class="search-box modal-search">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="11" cy="11" r="8"></circle>
            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
          </svg>
          <input
            type="text"
            placeholder={$_("preview.search_categories")}
            bind:value={modalSearchQuery}
          />
          {#if modalSearchQuery}
            <button class="clear-search" on:click={() => (modalSearchQuery = "")}>✕</button>
          {/if}
        </div>
      </div>

      <!-- Grid de Categorias Existentes -->
      <div class="large-tags-grid">
        {#if modalFilteredCategories.length === 0}
          <div class="empty-picker">Nenhuma categoria encontrada com os filtros atuais.</div>
        {:else}
          {#each modalFilteredCategories as cat (cat.id)}
            <button
              class="large-tag-btn glass-panel"
              style="border-left: 4px solid {cat.color ?? '#3b82f6'}"
              title="Atribuir categoria '{cat.name}'"
              on:click={() => assignCategoryToTarget(cat)}
            >
              <div class="tag-card-content">
                <span class="large-tag-name">{cat.name}</span>
                <div class="large-tag-meta">
                  <span class="mini-origin-badge {cat.created_by}">
                    {cat.created_by === "auto" ? $_("categories.created_by_auto") : $_("categories.created_by_user")}
                  </span>
                  <span class="tag-file-counter">
                    {$_("categories.files_count", { values: { count: cat.file_count } })}
                  </span>
                </div>
              </div>
            </button>
          {/each}
        {/if}
      </div>

      <div class="modal-actions">
        <button class="secondary-btn" on:click={() => (showCategoryModal = false)}>
          {$_("preview.modal.cancel")}
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Modal Amplo: Visualização de Conteúdo do Arquivo (Inspector / Quick Look) -->
<FilePreviewModal
  show={showFilePreviewModal}
  loading={previewLoading}
  data={filePreviewData}
  categoryName={activePreviewFile?.suggested_category}
  categoryColor={activePreviewFile?.category_color ?? undefined}
  confidence={activePreviewFile?.confidence}
  onClose={() => (showFilePreviewModal = false)}
  onOpenWithDefaultApp={handleOpenWithDefaultApp}
  onOpenInExplorer={handleOpenInExplorer}
/>

<style>
  .preview-layout {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 1.25rem 1.5rem;
    gap: 1rem;
    overflow: hidden;
    min-height: 0;
    min-width: 0;
    animation: fadeIn 250ms ease-out;
  }

  .preview-action-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1.25rem;
    flex-wrap: wrap;
    flex-shrink: 0;
  }

  .search-box {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 0.5rem 0.85rem;
    min-width: 260px;
  }

  .search-box input {
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-primary);
    font-size: 0.85rem;
    width: 100%;
  }

  .clear-search {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    font-size: 0.85rem;
  }

  .preview-rename-toggle {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.84rem;
    color: var(--text-primary);
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    padding: 0.45rem 0.85rem;
    border-radius: var(--radius-md);
    cursor: pointer;
    user-select: none;
  }

  .preview-rename-toggle input {
    cursor: pointer;
    accent-color: var(--accent-primary);
  }

  .action-buttons {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .primary-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: var(--accent-primary);
    color: white;
    border: none;
    border-radius: var(--radius-md);
    padding: 0.55rem 1.1rem;
    font-weight: 600;
    cursor: pointer;
    font-size: 0.88rem;
    transition: all 150ms ease;
  }

  .primary-btn:hover:not(:disabled) {
    filter: brightness(1.1);
  }

  .primary-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .secondary-btn {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    background: var(--bg-tertiary);
    color: var(--text-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 0.55rem 1rem;
    font-weight: 500;
    cursor: pointer;
    font-size: 0.88rem;
    transition: all 150ms ease;
  }

  .secondary-btn:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .secondary-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .trees-container {
    flex: 1;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.25rem;
    min-height: 0;
    overflow: hidden;
  }

  .preview-column {
    display: flex;
    flex-direction: column;
    border-radius: var(--radius-lg);
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    overflow: hidden;
    min-height: 0;
  }

  .column-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.85rem 1.15rem;
    border-bottom: 1px solid var(--border-subtle);
    background: var(--bg-tertiary);
    flex-shrink: 0;
  }

  .column-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: var(--text-primary);
  }

  .column-title h2 {
    font-size: 0.95rem;
    font-weight: 700;
    margin: 0;
  }

  .column-meta-actions {
    display: flex;
    align-items: center;
    gap: 0.75rem;
  }

  .column-meta {
    font-size: 0.75rem;
    color: var(--text-muted);
    font-weight: 500;
  }

  .tree-controls {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .mini-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: var(--radius-sm);
    background: var(--bg-secondary);
    border: 1px solid var(--border-subtle);
    color: var(--text-muted);
    cursor: pointer;
    transition: all 120ms ease;
  }

  .mini-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }

  .tree-scroll-container {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 0.5rem;
    min-height: 0;
  }

  .tree-root {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .empty-state {
    padding: 3.5rem 1rem;
    text-align: center;
    color: var(--text-muted);
    font-size: 0.88rem;
  }

  /* Context Menu on Right Click */
  .custom-context-menu {
    position: fixed;
    z-index: 2000;
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-xl);
    padding: 0.5rem;
    width: 290px;
    max-height: calc(100vh - 24px);
    overflow-y: auto;
    overflow-x: hidden;
    overscroll-behavior: contain;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    animation: fadeIn 120ms ease-out;
    /* Oculta a barra de rolagem visualmente mantendo o scroll 100% funcional */
    scrollbar-width: none; /* Firefox */
    -ms-overflow-style: none; /* IE e Edge */
  }

  .custom-context-menu::-webkit-scrollbar {
    display: none; /* Chrome, Safari e Webview2 */
    width: 0px;
    height: 0px;
  }

  .context-card-header {
    padding: 0.5rem 0.6rem 0.65rem 0.6rem;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .context-file-title {
    font-size: 0.88rem;
    font-weight: 700;
    color: var(--text-primary);
  }

  .context-file-path {
    font-size: 0.72rem;
    font-family: var(--font-mono);
    color: var(--text-muted);
  }

  .context-tag-pill {
    display: inline-flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.25rem 0.6rem;
    border-radius: var(--radius-full);
    border: 1px solid transparent;
    font-size: 0.78rem;
    font-weight: 700;
    margin-top: 0.25rem;
    width: fit-content;
    max-width: 100%;
  }

  .context-tag-name {
    flex: 1;
    min-width: 0;
  }

  .context-confidence {
    font-size: 0.72rem;
    opacity: 0.85;
    background: rgba(0, 0, 0, 0.08);
    padding: 0.05rem 0.35rem;
    border-radius: var(--radius-sm);
  }

  .cat-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .context-item {
    display: flex;
    align-items: center;
    gap: 0.55rem;
    padding: 0.55rem 0.7rem;
    font-size: 0.84rem;
    font-weight: 500;
    color: var(--text-primary);
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    text-align: left;
    width: 100%;
    flex-shrink: 0;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .context-item:hover {
    background: var(--bg-hover);
    color: var(--accent-primary);
  }

  .context-item.highlight-action {
    color: var(--accent-primary);
    font-weight: 600;
  }

  .context-item.highlight-action:hover {
    background: var(--accent-light);
  }

  .context-item.text-danger:hover {
    background: rgba(244, 63, 94, 0.12);
    color: var(--accent-rose);
  }

  .context-divider {
    height: 1px;
    background: var(--border-subtle);
    margin: 0.2rem 0;
  }

  /* Modals Gerais */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.7);
    backdrop-filter: blur(6px);
    z-index: 3000;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
    overflow-y: auto;
  }

  .modal-card {
    background: var(--bg-secondary);
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-xl);
    padding: 1.75rem;
    max-width: 480px;
    width: 100%;
    animation: fadeIn 200ms ease-out;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .modal-card-large {
    max-width: 860px;
    max-height: 85vh;
    overflow: hidden;
  }

  .modal-header-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 1rem;
  }

  .modal-header-row h2 {
    font-size: 1.25rem;
    font-weight: 700;
    margin: 0 0 0.25rem 0;
    color: var(--text-primary);
  }

  .modal-subtitle {
    font-size: 0.85rem;
    color: var(--text-muted);
    margin: 0;
    line-height: 1.4;
  }

  .close-btn {
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 1.1rem;
    cursor: pointer;
    padding: 0.2rem 0.4rem;
    border-radius: var(--radius-sm);
  }

  .close-btn:hover {
    color: var(--text-primary);
    background: var(--bg-tertiary);
  }

  /* Barra de criação rápida */
  .create-quick-box {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.85rem 1rem;
    border-radius: var(--radius-md);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    flex-wrap: wrap;
  }

  .create-input-group {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    min-width: 260px;
  }

  .text-input {
    width: 100%;
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 0.55rem 0.85rem;
    color: var(--text-primary);
    font-size: 0.88rem;
    outline: none;
  }

  .text-input.no-margin {
    margin-bottom: 0;
  }

  .palette-mini-dots {
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }

  .palette-dot {
    width: 18px;
    height: 18px;
    border-radius: var(--radius-full);
    border: 2px solid transparent;
    cursor: pointer;
    transition: transform 120ms ease;
  }

  .palette-dot.selected {
    border-color: white;
    transform: scale(1.25);
  }

  /* Toolbar de abas e busca dentro dos modais */
  .modal-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    border-bottom: 1px solid var(--border-subtle);
    padding-bottom: 0.6rem;
    flex-wrap: wrap;
  }

  .modal-tabs {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .modal-tab {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.4rem 0.75rem;
    border-radius: var(--radius-md);
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 0.82rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 120ms ease;
  }

  .modal-tab:hover {
    color: var(--text-primary);
    background: var(--bg-tertiary);
  }

  .modal-tab.active {
    background: var(--accent-primary);
    color: white;
    font-weight: 600;
  }

  .tab-badge {
    background: rgba(0, 0, 0, 0.2);
    font-size: 0.7rem;
    padding: 0.05rem 0.4rem;
    border-radius: var(--radius-full);
  }

  .modal-search {
    min-width: 220px;
    padding: 0.4rem 0.75rem;
  }

  /* Grid amplo de tags e categorias */
  .large-tags-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 0.75rem;
    max-height: 380px;
    overflow-y: auto;
    padding: 0.25rem;
    min-height: 120px;
  }

  .large-tag-btn {
    display: flex;
    flex-direction: column;
    padding: 0.75rem 0.9rem;
    border-radius: var(--radius-md);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-subtle);
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
    transition: all 150ms ease;
  }

  .large-tag-btn:hover {
    background: var(--bg-hover);
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .tag-card-content {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .large-tag-name {
    font-weight: 600;
    font-size: 0.92rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .large-tag-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.74rem;
  }

  .mini-origin-badge {
    padding: 0.1rem 0.4rem;
    border-radius: var(--radius-sm);
    font-weight: 500;
  }

  .mini-origin-badge.auto {
    background: rgba(139, 92, 246, 0.15);
    color: #a78bfa;
  }

  .mini-origin-badge.user {
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
  }

  .tag-file-counter {
    color: var(--text-muted);
  }

  .empty-picker {
    grid-column: 1 / -1;
    padding: 3rem 1rem;
    text-align: center;
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  /* File Viewer Modal */
  .preview-viewer-modal {
    max-width: 940px;
    height: 82vh;
    display: flex;
    flex-direction: column;
  }

  .preview-title-info {
    flex: 1;
    min-width: 0;
  }

  .file-path-sub {
    font-family: var(--font-mono);
    font-size: 0.76rem;
    margin-top: 0.2rem;
  }

  .preview-header-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .mini-action {
    padding: 0.4rem 0.75rem;
    font-size: 0.8rem;
  }

  .preview-viewer-body {
    flex: 1;
    overflow-y: auto;
    background: var(--bg-primary);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 250px;
    padding: 1rem;
  }

  .viewer-loader {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    color: var(--text-muted);
    font-size: 0.9rem;
  }

  .image-viewer-container {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: auto;
  }

  .preview-rendered-img {
    max-width: 100%;
    max-height: 520px;
    object-fit: contain;
    border-radius: var(--radius-sm);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.2);
  }

  .media-viewer-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.25rem;
    padding: 2rem;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.6rem;
    margin-top: 0.5rem;
  }

  .mini-spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 800ms linear infinite;
  }

  .mini-spinner.large {
    width: 28px;
    height: 28px;
    border-width: 3px;
    border-top-color: var(--accent-primary);
    border-color: rgba(59, 130, 246, 0.2);
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: translateY(6px); }
    to { opacity: 1; transform: translateY(0); }
  }
</style>
