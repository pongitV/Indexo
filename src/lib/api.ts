// Wrappers tipados em torno de `invoke()` -- toda comunicacao com o backend
// Rust passa por aqui, o resto do frontend nunca chama `invoke` direto.
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface ScanSummary {
  session_id: string;
  total_files: number;
  total_size_bytes: number;
}

export interface ScanProgressPayload {
  files_scanned: number;
  total_size_bytes: number;
  current_file: string;
}

export interface ClassifiedFile {
  file_id: string;
  path: string;
  filename: string;
  suggested_category: string;
  category_id: string;
  category_color?: string;
  confidence: number;
  tier_used: number; // 1 = heuristica, 2 = embedding/cluster, 3 = LLM local
  size_bytes?: number;
  is_already_organized?: boolean;
  original_relative_folder?: string | null;
}

export interface ClassifyProgressPayload {
  processed: number;
  total: number;
  current_phase: string;
  item?: ClassifiedFile;
}

export interface Category {
  id: string;
  name: string;
  color: string | null;
  created_by: "auto" | "user";
  file_count: number;
}

export interface FileMove {
  file_id: string;
  from_path: string;
  to_path: string;
}

export interface ApplySummary {
  moved: number;
  failed: string[];
}

export async function scanFolder(path: string): Promise<ScanSummary> {
  return invoke<ScanSummary>("scan_folder", { path });
}

export async function scanSpecificFiles(filePaths: string[]): Promise<ScanSummary> {
  return invoke<ScanSummary>("scan_specific_files", { filePaths });
}

export async function classifyScannedFiles(sessionId: string): Promise<ClassifiedFile[]> {
  return invoke<ClassifiedFile[]>("classify_scanned_files", { sessionId });
}

export async function applyOrganization(sessionId: string, moves: FileMove[]): Promise<ApplySummary> {
  return invoke<ApplySummary>("apply_organization", { sessionId, moves });
}

export async function undoLastApply(sessionId?: string | null): Promise<number> {
  return invoke<number>("undo_last_apply", { sessionId: sessionId ?? null });
}

export async function listCategories(): Promise<Category[]> {
  return invoke<Category[]>("list_categories");
}

export async function createCategory(name: string, color?: string): Promise<Category> {
  return invoke<Category>("create_category", { name, color: color ?? null });
}

export async function renameCategory(id: string, newName: string): Promise<void> {
  return invoke<void>("rename_category", { id, newName });
}

export async function mergeCategories(sourceId: string, targetId: string): Promise<void> {
  return invoke<void>("merge_categories", { sourceId, targetId });
}

export async function deleteCategory(id: string): Promise<void> {
  return invoke<void>("delete_category", { id });
}

export interface CategoryHistoryRecord {
  id: string;
  category_id: string;
  old_name: string;
  new_name: string;
  changed_by: string; // 'user' | 'ai_refinement' | 'merge' | 'auto'
  reason?: string | null;
  changed_at: string;
}

export interface MoveLogRecord {
  id: string;
  session_id: string;
  file_id: string;
  from_path: string;
  to_path: string;
  moved_at: string;
  undone: number;
}

export interface OrganizationSessionSummary {
  session_id: string;
  root_path: string;
  started_at: string;
  finished_at?: string | null;
  status: string;
  files_scanned: number;
  files_moved_count: number;
  undone_count: number;
  categories_assigned: string[];
  moves: MoveLogRecord[];
}

export async function getCategoryHistory(categoryId: string): Promise<CategoryHistoryRecord[]> {
  return invoke<CategoryHistoryRecord[]>("get_category_history", { categoryId });
}

export async function getOrganizationHistory(): Promise<OrganizationSessionSummary[]> {
  return invoke<OrganizationSessionSummary[]>("get_organization_history");
}

export async function undoSession(sessionId: string): Promise<number> {
  return invoke<number>("undo_session", { sessionId });
}

export async function cleanUnusedCategories(): Promise<number> {
  return invoke<number>("clean_unused_categories");
}

export async function purgeAutoCategories(): Promise<number> {
  return invoke<number>("purge_auto_categories");
}

export async function recordUserCorrection(
  fileId: string,
  oldCategoryId: string | null,
  newCategoryId: string,
): Promise<void> {
  return invoke<void>("record_user_correction", {
    fileId,
    oldCategoryId,
    newCategoryId,
  });
}

export async function exportProfile(destinationZipPath: string): Promise<void> {
  return invoke<void>("export_profile", { destinationZipPath });
}

export async function importProfile(sourceZipPath: string): Promise<void> {
  return invoke<void>("import_profile", { sourceZipPath });
}

export async function getSetting(key: string): Promise<string | null> {
  return invoke<string | null>("get_setting", { key });
}

export async function saveSetting(key: string, value: string): Promise<void> {
  return invoke<void>("save_setting", { key, value });
}

export interface FilePreviewData {
  filename: string;
  path: string;
  file_type: "image" | "text" | "code" | "pdf" | "spreadsheet" | "audio" | "video" | "binary";
  mime_type: string;
  size_bytes: number;
  text_content?: string | null;
  data_url?: string | null;
  exif_date?: string | null;
  dimensions?: string | null;
  line_count?: number | null;
  error?: string | null;
}

export async function openInExplorer(path: string): Promise<void> {
  return invoke<void>("open_in_explorer", { path });
}

export async function openWithDefaultApp(path: string): Promise<void> {
  return invoke<void>("open_with_default_app", { path });
}

export async function getFilePreview(path: string): Promise<FilePreviewData> {
  return invoke<FilePreviewData>("get_file_preview", { path });
}

export interface RenameConfig {
  preset: string; // "semantic" | "date_first" | "clean_only" | "custom"
  separator: string; // "_" | "-" | " " | "."
  case_style: string; // "title" | "lower" | "upper" | "camel" | "snake" | "kebab"
  date_format: string; // "YYYY-MM-DD" | "YYYY-MM" | "DD-MM-YYYY" | "none"
  include_category: boolean;
  remove_noise: boolean;
  custom_template?: string | null;
  structure_order?: string[]; // e.g. ["date", "subject", "clean_name"]
}

export interface RenameSuggestion {
  file_id: string;
  current_path: string;
  current_filename: string;
  proposed_filename: string;
  proposed_path: string;
  category: string;
  category_color?: string | null;
  size_bytes: number;
  is_modified_by_user: boolean;
  is_ignored: boolean;
  has_collision: boolean;
}

export interface FileRenameCandidate {
  file_id: string;
  path: string;
  filename: string;
  category: string;
  category_color?: string | null;
  size_bytes: number;
  modified_at?: string | null;
  text_sample?: string | null;
}

export interface RenameOperation {
  file_id: string;
  from_path: string;
  to_path: string;
}

export async function suggestSemanticNames(
  files: FileRenameCandidate[],
  config: RenameConfig
): Promise<RenameSuggestion[]> {
  return invoke<RenameSuggestion[]>("suggest_semantic_names", { files, config });
}

export async function applyRenames(
  sessionId: string,
  renames: RenameOperation[]
): Promise<ApplySummary> {
  return invoke<ApplySummary>("apply_renames", { sessionId, renames });
}

export function onScanProgress(cb: (payload: ScanProgressPayload) => void): Promise<UnlistenFn> {
  return listen<ScanProgressPayload>("scan://progress", (e) => cb(e.payload));
}

export function onClassifyProgress(cb: (payload: ClassifyProgressPayload) => void): Promise<UnlistenFn> {
  return listen<ClassifyProgressPayload>("classify://progress", (e) => cb(e.payload));
}

