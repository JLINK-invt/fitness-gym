import type { Update } from "@tauri-apps/plugin-updater";

let pending: Update | null = null;

export interface UpdateSummary {
  version: string;
  currentVersion: string;
  date: string | null;
  notes: string | null;
}

/** Looks for a signed update. Resolves to null when the app is up to date. */
export async function checkForUpdate(): Promise<UpdateSummary | null> {
  const { check } = await import("@tauri-apps/plugin-updater");
  const update = await check();
  if (!update) {
    pending = null;
    return null;
  }
  pending = update;
  return {
    version: update.version,
    currentVersion: update.currentVersion,
    date: update.date ?? null,
    notes: update.body ?? null,
  };
}

/** Downloads and installs the update found by the last check. */
export async function downloadAndInstallUpdate(): Promise<void> {
  if (!pending) {
    throw new Error("No hay una actualización pendiente.");
  }
  await pending.downloadAndInstall();
  pending = null;
}
