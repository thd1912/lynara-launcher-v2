// src/components/UpdateChecker.tsx
//
// Vérifie au démarrage si une mise à jour est disponible.
// Si oui, affiche une boîte de dialogue permettant de télécharger + redémarrer.

import { useEffect, useState } from "react";
import { check, Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

interface UpdateState {
  status: "idle" | "checking" | "available" | "downloading" | "ready" | "none" | "error";
  update?: Update;
  progress?: number;
  totalBytes?: number;
  downloadedBytes?: number;
  error?: string;
}

export default function UpdateChecker() {
  const [state, setState] = useState<UpdateState>({ status: "idle" });
  const [showModal, setShowModal] = useState(false);

  // Check for updates on mount
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        setState({ status: "checking" });
        const update = await check();
        if (cancelled) return;

        if (update) {
          console.log(`[Lynara Updater] Update available: v${update.version} (current v${update.currentVersion})`);
          setState({ status: "available", update });
          setShowModal(true);
        } else {
          console.log("[Lynara Updater] No update available, on latest version");
          setState({ status: "none" });
        }
      } catch (err) {
        console.error("[Lynara Updater] Check failed:", err);
        if (!cancelled) {
          setState({ status: "error", error: String(err) });
        }
      }
    })();
    return () => { cancelled = true; };
  }, []);

  const handleInstall = async () => {
    if (!state.update) return;
    try {
      setState(s => ({ ...s, status: "downloading", progress: 0, downloadedBytes: 0 }));

      let totalBytes = 0;
      let downloadedBytes = 0;

      await state.update.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            totalBytes = event.data.contentLength ?? 0;
            setState(s => ({ ...s, totalBytes }));
            break;
          case "Progress":
            downloadedBytes += event.data.chunkLength;
            setState(s => ({
              ...s,
              downloadedBytes,
              progress: totalBytes > 0 ? (downloadedBytes / totalBytes) * 100 : 0,
            }));
            break;
          case "Finished":
            setState(s => ({ ...s, status: "ready", progress: 100 }));
            break;
        }
      });

      // Relaunch the app to apply the update
      await relaunch();
    } catch (err) {
      console.error("[Lynara Updater] Install failed:", err);
      setState(s => ({ ...s, status: "error", error: String(err) }));
    }
  };

  const handleSkip = () => {
    setShowModal(false);
  };

  if (!showModal) return null;
  if (state.status === "none" || state.status === "idle") return null;

  return (
    <div className="fixed inset-0 z-[9999] flex items-center justify-center bg-black/70 backdrop-blur-sm">
      <div className="w-[480px] max-w-[90vw] rounded-2xl border border-purple-900/40 bg-gradient-to-b from-[#2E2447] to-[#231A38] p-6 shadow-2xl">

        {/* Header */}
        <div className="mb-4 flex items-center gap-3">
          <span className="text-3xl text-amber-400">⚜</span>
          <div>
            <h2 className="text-xl font-bold text-amber-300">Mise à jour disponible</h2>
            <p className="text-sm text-purple-200">
              {state.update && (
                <>v{state.update.currentVersion} → <span className="font-semibold text-amber-200">v{state.update.version}</span></>
              )}
            </p>
          </div>
        </div>

        {/* Body */}
        {state.status === "available" && state.update && (
          <>
            {state.update.body && (
              <div className="mb-4 max-h-32 overflow-y-auto rounded-lg bg-black/30 p-3 text-sm text-purple-100">
                <pre className="whitespace-pre-wrap font-sans">{state.update.body}</pre>
              </div>
            )}
            <p className="mb-5 text-sm text-purple-200">
              Voulez-vous installer cette mise à jour maintenant ? Le launcher sera redémarré.
            </p>
            <div className="flex gap-3">
              <button
                onClick={handleSkip}
                className="flex-1 rounded-lg border border-purple-900/60 bg-purple-950/40 px-4 py-2 text-purple-200 transition hover:bg-purple-950/70"
              >
                Plus tard
              </button>
              <button
                onClick={handleInstall}
                className="flex-1 rounded-lg bg-gradient-to-b from-amber-500 to-amber-600 px-4 py-2 font-semibold text-amber-950 transition hover:from-amber-400 hover:to-amber-500"
              >
                Installer maintenant
              </button>
            </div>
          </>
        )}

        {state.status === "downloading" && (
          <>
            <p className="mb-3 text-sm text-purple-200">Téléchargement en cours...</p>
            <div className="mb-2 h-2 overflow-hidden rounded-full bg-black/40">
              <div
                className="h-full bg-gradient-to-r from-amber-500 to-amber-300 transition-all"
                style={{ width: `${state.progress ?? 0}%` }}
              />
            </div>
            <p className="text-xs text-purple-300">
              {state.progress?.toFixed(0)}%
              {state.totalBytes && state.downloadedBytes && (
                <> · {(state.downloadedBytes / 1024 / 1024).toFixed(1)} / {(state.totalBytes / 1024 / 1024).toFixed(1)} MB</>
              )}
            </p>
          </>
        )}

        {state.status === "ready" && (
          <>
            <p className="mb-3 text-sm text-amber-300">✓ Mise à jour prête, redémarrage en cours...</p>
          </>
        )}

        {state.status === "error" && (
          <>
            <p className="mb-3 text-sm text-red-400">Erreur de mise à jour : {state.error}</p>
            <button
              onClick={handleSkip}
              className="w-full rounded-lg border border-purple-900/60 bg-purple-950/40 px-4 py-2 text-purple-200 transition hover:bg-purple-950/70"
            >
              Fermer
            </button>
          </>
        )}

      </div>
    </div>
  );
}
