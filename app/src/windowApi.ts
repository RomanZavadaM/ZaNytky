import { emit, listen, type UnlistenFn } from "@tauri-apps/api/event";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";

export type Section = "setup" | "prepare" | "result" | "database" | "entities" | "sources" | "channels" | "overlaps" | "overview" | "graph" | "article" | "report" | "selection";

const SECTIONS: Section[] = ["setup", "prepare", "result", "database", "entities", "sources", "channels", "overlaps", "overview", "graph", "article", "report", "selection"];
const LABEL_PREFIX = "section-";
const DATA_CHANGED = "zn-data-changed";
const SELECTION_CHANGED = "zn-selection-changed";
const SELECTION_KEY = "zn-selection";

export function sectionLabel(section: Section): string {
  return `${LABEL_PREFIX}${section}`;
}

/** The section this window was opened for, or null in the main window. */
export function windowSection(): Section | null {
  const params = new URLSearchParams(window.location.search);
  if (params.get("view") !== "section") return null;
  const tab = params.get("tab");
  return SECTIONS.find((s) => s === tab) ?? null;
}

/**
 * Opens a section in its own window. One window per section: an existing
 * one is brought to the front instead of creating a duplicate. Nothing is
 * passed in the URL except the section name (no paths, keys or text).
 */
export async function openSectionWindow(section: Section, title: string): Promise<void> {
  const label = sectionLabel(section);
  const existing = await WebviewWindow.getByLabel(label);
  if (existing) {
    await existing.unminimize();
    await existing.show();
    await existing.setFocus();
    return;
  }

  const query = new URLSearchParams({ view: "section", tab: section });
  const created = new WebviewWindow(label, {
    url: `index.html?${query.toString()}`,
    title,
    width: 980,
    height: 760,
    minWidth: 520,
    minHeight: 420,
    center: true,
  });

  await new Promise<void>((resolve, reject) => {
    void created.once("tauri://created", () => resolve());
    void created.once("tauri://error", (event) => reject(new Error(String(event.payload))));
  });
}

/** Tells every window that the database changed (import or status change). */
export async function notifyDataChanged(): Promise<void> {
  try {
    await emit(DATA_CHANGED);
  } catch {
    // not running inside the desktop shell; nothing to notify
  }
}

export async function onDataChanged(handler: () => void): Promise<UnlistenFn> {
  try {
    return await listen(DATA_CHANGED, handler);
  } catch {
    return () => undefined;
  }
}

/** The ids of the subjects ticked in the list (kept for the selection window; ids only). */
export function saveSelection(ids: number[]): void {
  try {
    localStorage.setItem(SELECTION_KEY, JSON.stringify(ids));
  } catch {
    // storage unavailable: the window then shows an empty selection
  }
}

export function loadSelection(): number[] {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(SELECTION_KEY) ?? "[]");
    return Array.isArray(parsed) ? parsed.filter((x): x is number => typeof x === "number") : [];
  } catch {
    return [];
  }
}

/** Opens (or refreshes) the window with the graph of the selection. */
export async function openSelectionWindow(title: string, ids: number[]): Promise<void> {
  saveSelection(ids);
  await openSectionWindow("selection", title);
  try {
    await emit(SELECTION_CHANGED);
  } catch {
    // not running inside the desktop shell
  }
}

export async function onSelectionChanged(handler: () => void): Promise<UnlistenFn> {
  try {
    return await listen(SELECTION_CHANGED, handler);
  } catch {
    return () => undefined;
  }
}
