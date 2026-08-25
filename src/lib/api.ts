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

export function onScanProgress(cb: (payload: ScanProgressPayload) => void): Promise<UnlistenFn> {
  return listen<ScanProgressPayload>("scan://progress", (e) => cb(e.payload));
}

export function onClassifyProgress(cb: (payload: ClassifyProgressPayload) => void): Promise<UnlistenFn> {
  return listen<ClassifyProgressPayload>("classify://progress", (e) => cb(e.payload));
}
