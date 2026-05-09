import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

// =====================================================================
// TYPES
// =====================================================================

export interface JavaInstall {
  path: string;
  version: number;
}

export interface PrepareResult {
  version_id: string;
  java_required: number;
  java_found: JavaInstall | null;
  main_class: string;
  estimated_download_bytes: number;
  manifest: unknown;
}

export type InstallPhase =
  | "initializing"
  | "downloading_client"
  | "downloading_libraries"
  | "extracting_natives"
  | "downloading_asset_index"
  | "downloading_assets"
  | "done";

export interface InstallProgress {
  phase: InstallPhase;
  message: string;
  bytes_done: number;
  bytes_total: number;
  speed_bps: number;
  files_done: number;
  files_total: number;
}

// === Game runtime events ===
export type GameEvent =
  | { kind: "started" }
  | { kind: "log"; line: string }
  | { kind: "error"; line: string }
  | { kind: "closed"; code: number };

// =====================================================================
// COMMANDS
// =====================================================================

export async function prepareMinecraft(versionId: string): Promise<PrepareResult> {
  return invoke("prepare_minecraft", { versionId });
}

export async function installMinecraft(versionId: string): Promise<void> {
  return invoke("install_minecraft", { versionId });
}

/**
 * Launch Minecraft with optional auto-connect to a server.
 * Promise resolves when the JVM is spawned (not when the game closes).
 * Subscribe to `onGameEvent` to track game lifecycle.
 */
export async function launchMinecraft(
  versionId: string,
  serverAddress?: string
): Promise<void> {
  return invoke("launch_minecraft", { versionId, serverAddress: serverAddress ?? null });
}

// =====================================================================
// EVENT SUBSCRIPTIONS
// =====================================================================

export async function onInstallProgress(
  callback: (progress: InstallProgress) => void
): Promise<UnlistenFn> {
  return listen<InstallProgress>("install:progress", (event) => {
    callback(event.payload);
  });
}

export async function onGameEvent(
  callback: (event: GameEvent) => void
): Promise<UnlistenFn> {
  return listen<GameEvent>("game:event", (event) => {
    callback(event.payload);
  });
}

// =====================================================================
// FORMATTERS
// =====================================================================

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

export function formatSpeed(bps: number): string {
  if (bps === 0) return "—";
  return `${formatBytes(bps)}/s`;
}

export function formatETA(bytes_remaining: number, speed_bps: number): string {
  if (speed_bps <= 0) return "—";
  const seconds = bytes_remaining / speed_bps;
  if (seconds < 60) return `${Math.round(seconds)}s`;
  if (seconds < 3600) return `${Math.round(seconds / 60)}m`;
  return `${(seconds / 3600).toFixed(1)}h`;
}

export function phaseLabel(phase: InstallPhase): string {
  switch (phase) {
    case "initializing":
      return "Initialisation";
    case "downloading_client":
      return "Client Minecraft";
    case "downloading_libraries":
      return "Bibliothèques";
    case "extracting_natives":
      return "Extraction natives";
    case "downloading_asset_index":
      return "Index des ressources";
    case "downloading_assets":
      return "Ressources";
    case "done":
      return "Terminé";
  }
}
