import { useState, useEffect, useRef } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  Copy,
  Check,
  Loader2,
  ArrowRight,
  ShieldCheck,
  AlertCircle,
  ExternalLink,
  PartyPopper,
} from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import LogoMark from "../components/ui/LogoMark";
import {
  startDeviceCodeLogin,
  pollDeviceCode,
  DeviceCodeInfo,
} from "../lib/auth";
import { useAuth } from "../stores/auth";

type LoginState =
  | "idle"
  | "loading"
  | "code-display"
  | "success"
  | "error";

export default function Login() {
  const [state, setState] = useState<LoginState>("idle");
  const [deviceInfo, setDeviceInfo] = useState<DeviceCodeInfo | null>(null);
  const [error, setError] = useState<string>("");
  const [copied, setCopied] = useState(false);
  const [countdown, setCountdown] = useState(0);
  const pollIntervalRef = useRef<number | null>(null);
  const countdownIntervalRef = useRef<number | null>(null);
  const setUser = useAuth((s) => s.setUser);

  // Cleanup intervals on unmount
  useEffect(() => {
    return () => {
      if (pollIntervalRef.current) clearInterval(pollIntervalRef.current);
      if (countdownIntervalRef.current) clearInterval(countdownIntervalRef.current);
    };
  }, []);

  const handleStartLogin = async () => {
    setError("");
    setState("loading");
    try {
      const info = await startDeviceCodeLogin();
      setDeviceInfo(info);
      setState("code-display");
      setCountdown(info.expires_in);

      // Auto-open browser to verification URL
      try {
        await openUrl(info.verification_uri);
      } catch (e) {
        console.warn("Couldn't auto-open browser:", e);
      }

      // Countdown timer (display only)
      countdownIntervalRef.current = window.setInterval(() => {
        setCountdown((c) => Math.max(0, c - 1));
      }, 1000);

      // Poll Microsoft for token
      pollIntervalRef.current = window.setInterval(async () => {
        try {
          const result = await pollDeviceCode(info.device_code);
          if (result.status === "complete") {
            cleanup();
            setState("success");
            // Show success briefly, then transition to Home via setUser
            setTimeout(() => setUser(result.profile), 1100);
          } else if (result.status === "error") {
            cleanup();
            setError(result.message);
            setState("error");
          }
          // authorization_pending / slow_down → keep polling
        } catch (e) {
          console.error("Poll error:", e);
          // Don't crash on transient errors, just keep polling
        }
      }, info.interval * 1000);
    } catch (e) {
      setError(typeof e === "string" ? e : "Erreur inattendue");
      setState("error");
    }
  };

  const cleanup = () => {
    if (pollIntervalRef.current) {
      clearInterval(pollIntervalRef.current);
      pollIntervalRef.current = null;
    }
    if (countdownIntervalRef.current) {
      clearInterval(countdownIntervalRef.current);
      countdownIntervalRef.current = null;
    }
  };

  const handleCopyCode = async () => {
    if (!deviceInfo) return;
    try {
      await navigator.clipboard.writeText(deviceInfo.user_code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (e) {
      console.error(e);
    }
  };

  const handleReset = () => {
    cleanup();
    setDeviceInfo(null);
    setError("");
    setState("idle");
  };

  const handleReopenBrowser = () => {
    if (deviceInfo) openUrl(deviceInfo.verification_uri).catch(() => {});
  };

  const formatTime = (s: number) => {
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return `${m}:${sec.toString().padStart(2, "0")}`;
  };

  return (
    <div className="h-full flex items-center justify-center p-8">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.6 }}
        className="w-full max-w-md"
      >
        {/* === LOGO + BRAND === */}
        <div className="text-center mb-8">
          <motion.div
            initial={{ scale: 0.7, rotate: -10 }}
            animate={{ scale: 1, rotate: 0 }}
            transition={{ delay: 0.2, type: "spring", stiffness: 180 }}
            className="inline-block mb-4"
          >
            <LogoMark size={84} glow="strong" floating />
          </motion.div>
          <h1 className="font-display text-4xl font-bold tracking-tight">
            <span className="text-text-primary">LYNA</span>
            <span className="text-gradient-gold">RA</span>
          </h1>
          <p className="text-text-muted text-[10px] mt-1.5 tracking-[0.25em] font-bold">
            LAUNCHER OFFICIEL
          </p>
        </div>

        {/* === STATE CARD === */}
        <div className="glass-strong rounded-2xl p-7 min-h-[280px] flex flex-col">
          <AnimatePresence mode="wait">
            {state === "idle" && <IdleState key="idle" onStart={handleStartLogin} />}
            {state === "loading" && <LoadingState key="loading" />}
            {state === "code-display" && deviceInfo && (
              <CodeState
                key="code"
                info={deviceInfo}
                copied={copied}
                onCopy={handleCopyCode}
                countdown={countdown}
                formatTime={formatTime}
                onReopen={handleReopenBrowser}
                onCancel={handleReset}
              />
            )}
            {state === "success" && <SuccessState key="success" />}
            {state === "error" && (
              <ErrorState key="error" message={error} onRetry={handleReset} />
            )}
          </AnimatePresence>
        </div>

        {/* === FOOTER === */}
        <div className="flex items-center justify-center gap-1.5 mt-5 text-text-muted">
          <ShieldCheck size={11} />
          <span className="text-[10px]">Connexion sécurisée via Microsoft</span>
        </div>
      </motion.div>
    </div>
  );
}

// =====================================================================
// SUB-COMPONENTS (one per state)
// =====================================================================

function IdleState({ onStart }: { onStart: () => void }) {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.3 }}
      className="flex flex-col justify-center flex-1"
    >
      <h2 className="text-xl font-semibold text-text-primary mb-2 text-center">
        Bienvenue, aventurier
      </h2>
      <p className="text-sm text-text-secondary mb-7 text-center leading-relaxed">
        Connecte-toi avec ton compte Microsoft pour rejoindre Lynara.
      </p>

      <motion.button
        onClick={onStart}
        whileHover={{ y: -2 }}
        whileTap={{ y: 0 }}
        className="w-full flex items-center justify-center gap-3 px-6 py-4 rounded-xl bg-gradient-to-br from-accent via-accent to-accent-dark text-bg-deep font-bold shadow-accent-lg overflow-hidden relative group"
      >
        <MicrosoftLogo />
        <span>Se connecter avec Microsoft</span>
        <ArrowRight size={16} className="group-hover:translate-x-1 transition-transform" />
        {/* Inner highlight */}
        <div className="absolute inset-x-0 top-0 h-1/2 bg-gradient-to-b from-white/25 to-transparent rounded-t-xl pointer-events-none" />
        {/* Shine */}
        <div className="absolute inset-0 -translate-x-full group-hover:translate-x-full transition-transform duration-700 bg-gradient-to-r from-transparent via-white/30 to-transparent" />
      </motion.button>

      <p className="text-[10px] text-text-muted text-center mt-4 leading-relaxed">
        Le launcher utilise le système officiel Microsoft Device Code.
        <br />
        Aucun mot de passe n'est jamais demandé ni stocké.
      </p>
    </motion.div>
  );
}

function LoadingState() {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="flex flex-col items-center justify-center flex-1 gap-4"
    >
      <Loader2 size={32} className="text-accent animate-spin" />
      <p className="text-sm text-text-secondary">Préparation de la connexion…</p>
    </motion.div>
  );
}

function CodeState({
  info,
  copied,
  onCopy,
  countdown,
  formatTime,
  onReopen,
  onCancel,
}: {
  info: DeviceCodeInfo;
  copied: boolean;
  onCopy: () => void;
  countdown: number;
  formatTime: (s: number) => string;
  onReopen: () => void;
  onCancel: () => void;
}) {
  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -10 }}
      transition={{ duration: 0.3 }}
      className="flex flex-col"
    >
      <p className="text-[10px] uppercase tracking-[0.22em] font-bold text-text-muted text-center mb-3">
        Ton code de connexion
      </p>

      {/* The big code */}
      <button
        onClick={onCopy}
        className="group relative w-full px-6 py-5 rounded-xl bg-bg-card border-2 border-accent/40 hover:border-accent transition-colors mb-4"
      >
        <div className="font-display text-5xl font-bold tracking-[0.18em] text-gradient-gold text-center select-all">
          {info.user_code}
        </div>
        <div className="absolute top-2.5 right-2.5 flex items-center gap-1 px-2 py-1 rounded-md bg-bg-deep/60 text-[10px] text-text-secondary opacity-0 group-hover:opacity-100 transition-opacity">
          {copied ? (
            <>
              <Check size={10} className="text-success" />
              <span className="text-success">Copié</span>
            </>
          ) : (
            <>
              <Copy size={10} />
              <span>Copier</span>
            </>
          )}
        </div>
      </button>

      {/* Instructions */}
      <div className="space-y-2 mb-5 text-xs text-text-secondary">
        <Step n={1}>
          Une page Microsoft s'est ouverte dans ton navigateur
          <button
            onClick={onReopen}
            className="ml-1 text-accent hover:underline inline-flex items-center gap-0.5"
          >
            (rouvrir <ExternalLink size={9} />)
          </button>
        </Step>
        <Step n={2}>Entre le code ci-dessus</Step>
        <Step n={3}>Connecte-toi avec ton compte Microsoft</Step>
      </div>

      {/* Polling indicator */}
      <div className="flex items-center justify-between p-3 rounded-lg bg-bg-card/40">
        <div className="flex items-center gap-2">
          <Loader2 size={12} className="text-accent animate-spin" />
          <span className="text-[11px] text-text-secondary">En attente de ta connexion…</span>
        </div>
        <span className="text-[10px] font-mono text-text-muted">
          {formatTime(countdown)}
        </span>
      </div>

      <button
        onClick={onCancel}
        className="text-[10px] text-text-muted hover:text-text-secondary transition-colors mt-3 self-center"
      >
        Annuler
      </button>
    </motion.div>
  );
}

function SuccessState() {
  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.8 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: 0.4, type: "spring", stiffness: 180 }}
      className="flex flex-col items-center justify-center flex-1 gap-4"
    >
      <motion.div
        initial={{ scale: 0 }}
        animate={{ scale: 1 }}
        transition={{ delay: 0.1, type: "spring", stiffness: 200 }}
        className="w-16 h-16 rounded-full bg-success/15 ring-2 ring-success flex items-center justify-center"
      >
        <PartyPopper size={28} className="text-success" />
      </motion.div>
      <div className="text-center">
        <p className="text-lg font-semibold text-text-primary">Connecté !</p>
        <p className="text-sm text-text-muted">Bienvenue sur Lynara</p>
      </div>
    </motion.div>
  );
}

function ErrorState({ message, onRetry }: { message: string; onRetry: () => void }) {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="flex flex-col items-center justify-center flex-1 gap-4"
    >
      <div className="w-14 h-14 rounded-full bg-danger/15 ring-2 ring-danger flex items-center justify-center">
        <AlertCircle size={24} className="text-danger" />
      </div>
      <div className="text-center max-w-xs">
        <p className="text-base font-semibold text-text-primary mb-1">Connexion échouée</p>
        <p className="text-xs text-text-secondary leading-relaxed">{message}</p>
      </div>
      <button
        onClick={onRetry}
        className="px-5 py-2.5 rounded-lg bg-white/5 hover:bg-white/10 text-sm font-medium text-text-primary transition-colors"
      >
        Réessayer
      </button>
    </motion.div>
  );
}

// =====================================================================
// HELPERS
// =====================================================================

function Step({ n, children }: { n: number; children: React.ReactNode }) {
  return (
    <div className="flex items-start gap-3">
      <div className="w-5 h-5 rounded-full bg-accent/15 ring-1 ring-accent/30 flex items-center justify-center flex-shrink-0 mt-0.5">
        <span className="text-[10px] font-bold text-accent">{n}</span>
      </div>
      <span className="leading-relaxed">{children}</span>
    </div>
  );
}

function MicrosoftLogo() {
  return (
    <svg width="16" height="16" viewBox="0 0 21 21" fill="none">
      <rect x="1" y="1" width="9" height="9" fill="#F25022" />
      <rect x="11" y="1" width="9" height="9" fill="#7FBA00" />
      <rect x="1" y="11" width="9" height="9" fill="#00A4EF" />
      <rect x="11" y="11" width="9" height="9" fill="#FFB900" />
    </svg>
  );
}
