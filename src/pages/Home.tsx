import {
  Play,
  Users,
  Activity,
  Zap,
  Sparkles,
  Crown,
  Loader2,
  CheckCircle2,
  AlertCircle,
  Download,
  Gamepad2,
  Terminal,
} from "lucide-react";
import { motion, AnimatePresence } from "framer-motion";
import { useEffect, useRef } from "react";
import LogoMark from "../components/ui/LogoMark";
import { useGame } from "../stores/game";
import {
  prepareMinecraft,
  installMinecraft,
  launchMinecraft,
  onInstallProgress,
  onGameEvent,
  formatBytes,
  formatSpeed,
  formatETA,
  phaseLabel,
} from "../lib/minecraft";

const TARGET_VERSION = "1.21.11";
const SERVER_ADDRESS = "play.lynara.fr";

export default function Home() {
  const status = useGame((s) => s.status);
  const error = useGame((s) => s.error);
  const progress = useGame((s) => s.progress);
  const logs = useGame((s) => s.logs);
  const exitCode = useGame((s) => s.exitCode);
  const setStatus = useGame((s) => s.setStatus);
  const setError = useGame((s) => s.setError);
  const setPreflight = useGame((s) => s.setPreflight);
  const setProgress = useGame((s) => s.setProgress);
  const appendLog = useGame((s) => s.appendLog);
  const clearLogs = useGame((s) => s.clearLogs);
  const setExitCode = useGame((s) => s.setExitCode);

  const isPreparing = status === "preparing";
  const isDownloading = status === "downloading";
  const isLaunching = status === "launching";
  const isRunning = status === "running";
  const hasError = status === "error";

  const installUnlistenRef = useRef<(() => void) | null>(null);
  const gameUnlistenRef = useRef<(() => void) | null>(null);

  // Subscribe to install progress + game events on mount
  useEffect(() => {
    let installUnsub: (() => void) | null = null;
    let gameUnsub: (() => void) | null = null;

    onInstallProgress((p) => setProgress(p)).then((fn) => {
      installUnsub = fn;
      installUnlistenRef.current = fn;
    });

    onGameEvent((evt) => {
      switch (evt.kind) {
        case "started":
          setStatus("running");
          break;
        case "log":
          appendLog(evt.line);
          break;
        case "error":
          appendLog(`[ERR] ${evt.line}`);
          break;
        case "closed":
          setStatus("idle");
          setExitCode(evt.code);
          break;
      }
    }).then((fn) => {
      gameUnsub = fn;
      gameUnlistenRef.current = fn;
    });

    return () => {
      if (installUnsub) installUnsub();
      if (gameUnsub) gameUnsub();
    };
  }, [setProgress, setStatus, appendLog, setExitCode]);

  const handlePlay = async () => {
    if (isPreparing || isDownloading || isLaunching || isRunning) return;
    setError(null);
    clearLogs();
    setExitCode(null);

    try {
      // Step 1 — Pre-flight
      setStatus("preparing");
      const preflight = await prepareMinecraft(TARGET_VERSION);
      setPreflight({
        version_id: preflight.version_id,
        java_required: preflight.java_required,
        java_found: preflight.java_found,
        estimated_download_bytes: preflight.estimated_download_bytes,
        main_class: preflight.main_class,
      });

      // Step 2 — Install (idempotent: skips files already present)
      setStatus("downloading");
      setProgress({
        phase: "initializing",
        message: "Démarrage...",
        bytes_done: 0,
        bytes_total: 0,
        speed_bps: 0,
        files_done: 0,
        files_total: 0,
      });
      await installMinecraft(TARGET_VERSION);

      // Step 3 — Launch (status will go to "running" on game:started event)
      setStatus("launching");
      await launchMinecraft(TARGET_VERSION, SERVER_ADDRESS);
      // Stay in "launching" until we receive game:started event from backend
    } catch (e) {
      const msg = typeof e === "string" ? e : (e as Error).message ?? "Erreur inconnue";
      setError(msg);
      setStatus("error");
    }
  };

  const pct =
    progress && progress.bytes_total > 0
      ? Math.min(100, (progress.bytes_done / progress.bytes_total) * 100)
      : 0;

  return (
    <div className="h-full p-8 space-y-6">
      {/* ================== HERO CARD ================== */}
      <motion.div
        initial={{ opacity: 0, y: 30 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.7, ease: [0.34, 1.56, 0.64, 1] }}
        className="relative rounded-3xl overflow-hidden glass-strong min-h-[420px] flex items-stretch"
      >
        <div className="absolute inset-0 bg-gradient-to-br from-[#3d2540]/50 via-[#2a1830]/30 to-[#1f1325]/50 pointer-events-none" />
        <div className="absolute -top-32 -right-32 w-[500px] h-[500px] bg-accent/[0.12] blur-[120px] rounded-full pointer-events-none" />
        <div className="absolute -bottom-20 -left-20 w-[400px] h-[400px] bg-[#8c50b4]/[0.10] blur-[100px] rounded-full pointer-events-none" />

        <motion.div
          initial={{ opacity: 0, scale: 0.8 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ delay: 0.4, duration: 0.4 }}
          className="absolute top-6 right-6 flex items-center gap-2 px-3.5 py-1.5 rounded-full glass z-20"
        >
          <Sparkles size={12} className="text-accent" />
          <span className="text-[10px] font-bold text-text-secondary tracking-wider">
            v2.0.0-alpha
          </span>
        </motion.div>

        <div className="relative z-10 flex-1 p-10 flex flex-col justify-end">
          <motion.div
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.2, duration: 0.5 }}
            className="inline-flex items-center gap-2 px-3.5 py-1.5 rounded-full bg-gradient-to-r from-accent/20 to-accent/5 border border-accent/30 mb-6 self-start"
          >
            <Crown size={12} className="text-accent" />
            <span className="text-[10px] font-bold text-accent uppercase tracking-[0.22em]">
              Le plus populaire
            </span>
          </motion.div>

          <motion.h1
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3, duration: 0.6 }}
            className="font-display font-bold tracking-tight leading-[0.92] mb-5"
            style={{ fontSize: "clamp(56px, 8vw, 96px)" }}
          >
            <span className="text-text-primary text-shadow-glow">LYNA</span>
            <span className="text-gradient-gold">RA</span>
          </motion.h1>

          <motion.p
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.4, duration: 0.6 }}
            className="text-text-secondary max-w-md mb-8 leading-relaxed"
            style={{ fontSize: "15px" }}
          >
            Un serveur OneBlock immersif où chaque île raconte une histoire.
            <br />
            <span className="text-text-primary font-semibold">
              Construis. Forge. Conquiers.
            </span>
          </motion.p>

          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.5, duration: 0.6 }}
            className="flex items-center gap-3 flex-wrap"
          >
            <PlayButton
              onClick={handlePlay}
              status={status}
              progress={progress}
              pct={pct}
            />

            <div className="px-5 py-4 rounded-2xl glass">
              <div className="flex items-center gap-2.5">
                <div className="relative">
                  <div className="w-2 h-2 rounded-full bg-success" />
                  <div className="absolute inset-0 w-2 h-2 rounded-full bg-success animate-ping opacity-75" />
                </div>
                <span className="text-sm font-mono font-semibold text-text-primary">
                  Lynara
                </span>
                <span className="text-xs text-text-muted">
                  Java {TARGET_VERSION}
                </span>
              </div>
            </div>
          </motion.div>
        </div>

        <div className="relative z-10 hidden md:flex items-center justify-center w-[40%] min-w-[300px] pr-8">
          <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
            <div className="w-72 h-72 rounded-full bg-accent/20 blur-3xl" />
          </div>
          <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
            <div
              className="w-80 h-80 rounded-full opacity-30"
              style={{
                background:
                  "conic-gradient(from 0deg, transparent 0deg, rgba(244,185,66,0.6) 90deg, transparent 180deg, rgba(244,185,66,0.4) 270deg, transparent 360deg)",
                animation: "spin-slow 20s linear infinite",
                maskImage:
                  "radial-gradient(circle, transparent 50%, black 50.5%, black 60%, transparent 60.5%)",
                WebkitMaskImage:
                  "radial-gradient(circle, transparent 50%, black 50.5%, black 60%, transparent 60.5%)",
              }}
            />
          </div>
          <motion.div
            initial={{ opacity: 0, scale: 0.7, rotate: -10 }}
            animate={{ opacity: 1, scale: 1, rotate: 0 }}
            transition={{ delay: 0.5, duration: 0.8, ease: [0.34, 1.56, 0.64, 1] }}
            className="relative"
          >
            <LogoMark size={240} glow="strong" floating />
          </motion.div>
        </div>
      </motion.div>

      {/* ================== STATE PANELS ================== */}
      <AnimatePresence mode="wait">
        {isDownloading && progress && (
          <motion.div
            key="dl"
            initial={{ opacity: 0, height: 0, y: -10 }}
            animate={{ opacity: 1, height: "auto", y: 0 }}
            exit={{ opacity: 0, height: 0, y: -10 }}
            transition={{ duration: 0.3 }}
            className="overflow-hidden"
          >
            <DownloadPanel progress={progress} pct={pct} />
          </motion.div>
        )}

        {isLaunching && (
          <motion.div
            key="launch"
            initial={{ opacity: 0, height: 0, y: -10 }}
            animate={{ opacity: 1, height: "auto", y: 0 }}
            exit={{ opacity: 0, height: 0, y: -10 }}
            transition={{ duration: 0.3 }}
            className="overflow-hidden"
          >
            <LaunchingPanel />
          </motion.div>
        )}

        {isRunning && (
          <motion.div
            key="run"
            initial={{ opacity: 0, height: 0, y: -10 }}
            animate={{ opacity: 1, height: "auto", y: 0 }}
            exit={{ opacity: 0, height: 0, y: -10 }}
            transition={{ duration: 0.3 }}
            className="overflow-hidden"
          >
            <RunningPanel logs={logs} />
          </motion.div>
        )}

        {hasError && error && (
          <motion.div
            key="err"
            initial={{ opacity: 0, height: 0, y: -10 }}
            animate={{ opacity: 1, height: "auto", y: 0 }}
            exit={{ opacity: 0, height: 0, y: -10 }}
            transition={{ duration: 0.3 }}
            className="overflow-hidden"
          >
            <ErrorPanel error={error} />
          </motion.div>
        )}

        {!isDownloading && !isLaunching && !isRunning && !hasError && exitCode !== null && (
          <motion.div
            key="closed"
            initial={{ opacity: 0, height: 0, y: -10 }}
            animate={{ opacity: 1, height: "auto", y: 0 }}
            exit={{ opacity: 0, height: 0, y: -10 }}
            transition={{ duration: 0.3 }}
            className="overflow-hidden"
          >
            <ClosedPanel code={exitCode} />
          </motion.div>
        )}
      </AnimatePresence>

      {/* ================== STATS GRID ================== */}
      <div className="grid grid-cols-3 gap-4">
        <StatCard icon={Users} label="Joueurs en ligne" value="0" sub="/ 20 max" delay={0.6} />
        <StatCard icon={Activity} label="Latence" value="8" sub="ms" delay={0.65} accent />
        <StatCard icon={Zap} label="Uptime" value="99.9" sub="%" delay={0.7} />
      </div>
    </div>
  );
}

/* ================ Sub-components ================ */

function PlayButton({
  onClick,
  status,
  progress,
  pct,
}: {
  onClick: () => void;
  status: string;
  progress: ReturnType<typeof useGame.getState>["progress"];
  pct: number;
}) {
  const isPreparing = status === "preparing";
  const isDownloading = status === "downloading";
  const isLaunching = status === "launching";
  const isRunning = status === "running";
  const disabled = isPreparing || isDownloading || isLaunching || isRunning;

  // === Downloading state — progress bar inside button ===
  if (isDownloading && progress) {
    const remaining = Math.max(0, progress.bytes_total - progress.bytes_done);
    return (
      <div className="relative inline-flex items-center gap-3.5 px-7 py-4 rounded-2xl bg-bg-card border border-accent/40 min-w-[420px] overflow-hidden">
        <div
          className="absolute inset-y-0 left-0 bg-gradient-to-r from-accent/40 to-accent/15 transition-all duration-200 ease-out"
          style={{ width: `${pct}%` }}
        />
        <div className="absolute inset-0 -translate-x-full animate-[shimmer_2s_infinite] bg-gradient-to-r from-transparent via-white/10 to-transparent pointer-events-none" />

        <div className="relative flex items-center gap-3 flex-1">
          <Download size={18} className="text-accent animate-pulse flex-shrink-0" />
          <div className="flex-1 min-w-0">
            <div className="flex items-center justify-between gap-3 mb-0.5">
              <span className="text-[11px] font-bold text-accent uppercase tracking-wider">
                {phaseLabel(progress.phase)}
              </span>
              <span className="font-mono text-sm font-bold text-text-primary tabular-nums">
                {pct.toFixed(0)}%
              </span>
            </div>
            <div className="text-[10px] text-text-secondary font-mono flex items-center gap-2">
              <span>
                {formatBytes(progress.bytes_done)} / {formatBytes(progress.bytes_total)}
              </span>
              <span className="text-text-muted">·</span>
              <span>{formatSpeed(progress.speed_bps)}</span>
              <span className="text-text-muted">·</span>
              <span>ETA {formatETA(remaining, progress.speed_bps)}</span>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // === Launching state ===
  if (isLaunching) {
    return (
      <div className="inline-flex items-center gap-3.5 px-10 py-5 rounded-2xl bg-gradient-to-br from-accent/20 to-accent/5 border border-accent/40 text-accent font-bold text-lg">
        <Loader2 size={22} className="animate-spin" />
        <span>Démarrage de Minecraft...</span>
      </div>
    );
  }

  // === Running state ===
  if (isRunning) {
    return (
      <div className="relative inline-flex items-center gap-3.5 px-10 py-5 rounded-2xl bg-gradient-to-br from-success/20 to-success/5 border border-success/40 text-success font-bold text-lg">
        <Gamepad2 size={22} />
        <span>Minecraft en cours</span>
        <div className="w-2 h-2 rounded-full bg-success animate-pulse ml-1" />
      </div>
    );
  }

  // === Idle / Preparing state ===
  return (
    <motion.button
      onClick={onClick}
      disabled={disabled}
      whileHover={!disabled ? { y: -3, scale: 1.02 } : {}}
      whileTap={!disabled ? { y: 0, scale: 1 } : {}}
      transition={{ type: "spring", stiffness: 400, damping: 17 }}
      className="group relative inline-flex items-center gap-3.5 px-10 py-5 rounded-2xl bg-gradient-to-br from-accent via-accent to-accent-dark text-bg-deep font-bold text-lg shadow-accent-lg overflow-hidden disabled:opacity-80 disabled:cursor-wait"
    >
      {isPreparing ? (
        <Loader2 size={22} className="animate-spin" />
      ) : (
        <Play size={22} fill="currentColor" />
      )}
      <span>{isPreparing ? "Préparation..." : "Jouer maintenant"}</span>
      {!disabled && (
        <div className="absolute inset-0 -translate-x-full group-hover:translate-x-full transition-transform duration-700 bg-gradient-to-r from-transparent via-white/40 to-transparent" />
      )}
      <div className="absolute inset-x-0 top-0 h-1/2 bg-gradient-to-b from-white/25 to-transparent rounded-t-2xl pointer-events-none" />
    </motion.button>
  );
}

function DownloadPanel({
  progress,
  pct,
}: {
  progress: NonNullable<ReturnType<typeof useGame.getState>["progress"]>;
  pct: number;
}) {
  return (
    <div className="glass rounded-2xl p-6">
      <div className="flex items-center gap-2 mb-4">
        <Download size={18} className="text-accent animate-pulse" />
        <h2 className="font-display text-base font-bold text-text-primary">
          Installation en cours
        </h2>
        <span className="ml-auto text-xs text-text-muted font-mono">
          {progress.files_done.toLocaleString()} /{" "}
          {progress.files_total.toLocaleString()} fichiers
        </span>
      </div>

      <p className="text-sm text-text-secondary mb-4">{progress.message}</p>

      <div className="relative h-2 rounded-full bg-bg-card overflow-hidden mb-4">
        <motion.div
          className="absolute inset-y-0 left-0 bg-gradient-to-r from-accent to-accent-light rounded-full"
          style={{ width: `${pct}%` }}
          transition={{ duration: 0.2 }}
        />
        <div
          className="absolute inset-y-0 left-0 bg-accent/40 blur-sm rounded-full"
          style={{ width: `${pct}%` }}
        />
      </div>

      <div className="grid grid-cols-4 gap-3">
        <SmallStat label="Phase" value={phaseLabel(progress.phase)} />
        <SmallStat
          label="Téléchargé"
          value={`${formatBytes(progress.bytes_done)} / ${formatBytes(progress.bytes_total)}`}
        />
        <SmallStat label="Vitesse" value={formatSpeed(progress.speed_bps)} />
        <SmallStat
          label="ETA"
          value={formatETA(
            Math.max(0, progress.bytes_total - progress.bytes_done),
            progress.speed_bps
          )}
        />
      </div>
    </div>
  );
}

function LaunchingPanel() {
  return (
    <div className="glass rounded-2xl p-6">
      <div className="flex items-center gap-3">
        <Loader2 size={20} className="text-accent animate-spin" />
        <div>
          <p className="text-sm font-semibold text-text-primary">
            Démarrage de Minecraft
          </p>
          <p className="text-xs text-text-secondary">
            Extraction des bibliothèques natives, lancement de la JVM...
          </p>
        </div>
      </div>
    </div>
  );
}

function RunningPanel({ logs }: { logs: string[] }) {
  return (
    <div className="glass rounded-2xl p-6">
      <div className="flex items-center gap-2 mb-4">
        <Gamepad2 size={18} className="text-success" />
        <h2 className="font-display text-base font-bold text-text-primary">
          Minecraft est en cours d'exécution
        </h2>
        <span className="ml-auto inline-flex items-center gap-1.5 text-xs text-success font-mono">
          <span className="w-1.5 h-1.5 rounded-full bg-success animate-pulse" />
          Connecté à {SERVER_ADDRESS}
        </span>
      </div>

      <div className="rounded-lg bg-bg-deep/60 border border-white/[0.04] p-3 max-h-[200px] overflow-y-auto">
        <div className="flex items-center gap-1.5 mb-2 text-text-muted">
          <Terminal size={11} />
          <span className="text-[10px] uppercase tracking-widest font-bold">
            Logs Minecraft
          </span>
        </div>
        <pre className="text-[11px] font-mono text-text-secondary whitespace-pre-wrap break-all leading-relaxed">
          {logs.length === 0
            ? "En attente des premiers logs..."
            : logs.slice(-15).join("\n")}
        </pre>
      </div>
    </div>
  );
}

function ClosedPanel({ code }: { code: number }) {
  const ok = code === 0;
  return (
    <div className="glass rounded-2xl p-6">
      <div className="flex items-center gap-3">
        {ok ? (
          <CheckCircle2 size={20} className="text-success" />
        ) : (
          <AlertCircle size={20} className="text-danger" />
        )}
        <div>
          <p className="text-sm font-semibold text-text-primary">
            {ok ? "Minecraft fermé proprement" : "Minecraft a planté"}
          </p>
          <p className="text-xs text-text-secondary font-mono">
            Code de sortie: {code}
          </p>
        </div>
      </div>
    </div>
  );
}

function ErrorPanel({ error }: { error: string }) {
  return (
    <div className="glass rounded-2xl p-6 border-danger/30">
      <div className="flex items-start gap-3">
        <AlertCircle size={20} className="text-danger flex-shrink-0 mt-0.5" />
        <div>
          <p className="text-sm font-semibold text-text-primary mb-1">Erreur</p>
          <p className="text-xs text-text-secondary font-mono break-all">{error}</p>
        </div>
      </div>
    </div>
  );
}

function SmallStat({ label, value }: { label: string; value: string }) {
  return (
    <div className="p-3 rounded-lg bg-white/[0.03] border border-white/[0.05]">
      <p className="text-[9px] uppercase tracking-widest text-text-muted font-bold mb-1">
        {label}
      </p>
      <p className="text-xs font-semibold text-text-primary truncate font-mono tabular-nums">
        {value}
      </p>
    </div>
  );
}

function StatCard({
  icon: Icon,
  label,
  value,
  sub,
  delay = 0,
  accent = false,
}: {
  icon: typeof Users;
  label: string;
  value: string;
  sub: string;
  delay?: number;
  accent?: boolean;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.5, delay, ease: "easeOut" }}
      whileHover={{ y: -3 }}
      className="glass rounded-2xl p-5 group cursor-default relative overflow-hidden"
    >
      <div className="absolute inset-0 opacity-0 group-hover:opacity-100 transition-opacity bg-gradient-to-br from-accent/[0.05] to-transparent pointer-events-none" />
      <div
        className={`relative w-10 h-10 rounded-lg flex items-center justify-center mb-3 transition-all group-hover:scale-110 ${
          accent
            ? "bg-accent/15 ring-1 ring-accent/30 shadow-[0_4px_16px_-4px_rgba(244,185,66,0.4)]"
            : "bg-white/[0.04] ring-1 ring-white/[0.06]"
        }`}
      >
        <Icon size={17} className={accent ? "text-accent" : "text-text-secondary"} />
      </div>
      <p className="relative text-[10px] uppercase tracking-[0.18em] text-text-muted font-bold mb-1.5">
        {label}
      </p>
      <div className="relative flex items-baseline gap-1.5">
        <span className="text-3xl font-bold font-display tracking-tight text-text-primary leading-none">
          {value}
        </span>
        <span className="text-xs text-text-muted">{sub}</span>
      </div>
    </motion.div>
  );
}
