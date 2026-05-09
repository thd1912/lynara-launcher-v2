import { create } from "zustand";
import { JavaInstall, InstallProgress } from "../lib/minecraft";

export type GameStatus =
  | "idle"
  | "preparing"
  | "ready"
  | "downloading"
  | "launching"
  | "running"
  | "error";

interface PreflightInfo {
  version_id: string;
  java_required: number;
  java_found: JavaInstall | null;
  estimated_download_bytes: number;
  main_class: string;
}

interface GameStore {
  status: GameStatus;
  message: string;
  error: string | null;
  preflight: PreflightInfo | null;
  progress: InstallProgress | null;
  /** Last ~40 lines of Minecraft stdout/stderr (Phase 3.3) */
  logs: string[];
  /** Exit code if the game has closed */
  exitCode: number | null;

  setStatus: (s: GameStatus) => void;
  setMessage: (m: string) => void;
  setError: (e: string | null) => void;
  setPreflight: (p: PreflightInfo | null) => void;
  setProgress: (p: InstallProgress | null) => void;
  appendLog: (line: string) => void;
  clearLogs: () => void;
  setExitCode: (code: number | null) => void;
  reset: () => void;
}

const MAX_LOG_LINES = 40;

export const useGame = create<GameStore>((set) => ({
  status: "idle",
  message: "",
  error: null,
  preflight: null,
  progress: null,
  logs: [],
  exitCode: null,

  setStatus: (status) => set({ status }),
  setMessage: (message) => set({ message }),
  setError: (error) => set({ error }),
  setPreflight: (preflight) => set({ preflight }),
  setProgress: (progress) => set({ progress }),
  appendLog: (line) =>
    set((state) => {
      const next = [...state.logs, line];
      if (next.length > MAX_LOG_LINES) {
        next.splice(0, next.length - MAX_LOG_LINES);
      }
      return { logs: next };
    }),
  clearLogs: () => set({ logs: [] }),
  setExitCode: (exitCode) => set({ exitCode }),
  reset: () =>
    set({
      status: "idle",
      message: "",
      error: null,
      preflight: null,
      progress: null,
      logs: [],
      exitCode: null,
    }),
}));
