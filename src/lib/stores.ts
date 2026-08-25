import { writable } from "svelte/store";
import type { ClassifiedFile, ScanSummary } from "./api";

export type View = "folder-select" | "scanning" | "preview" | "renamer" | "tags" | "categories" | "settings";

export const currentView = writable<View>("folder-select");
export const selectedFolder = writable<string | null>(null);
export const currentSessionId = writable<string | null>(null);
export const scanSummary = writable<ScanSummary | null>(null);
export const classifiedFiles = writable<ClassifiedFile[]>([]);
export const theme = writable<"light" | "dark" | "system">("system");
export const language = writable<"pt-BR" | "en-US">("pt-BR");
export const alsoRenameInOrganization = writable<boolean>(false);

export interface ToastMessage {
  id: string;
  message: string;
  type: "success" | "info" | "error";
}
export const toastMessages = writable<ToastMessage[]>([]);

export function showToast(message: string, type: "success" | "info" | "error" = "success") {
  const id = Math.random().toString(36).substring(2, 9);
  toastMessages.update((list) => [...list, { id, message, type }]);
  setTimeout(() => {
    toastMessages.update((list) => list.filter((t) => t.id !== id));
  }, 4000);
}
