import { motion } from "framer-motion";
import { Cpu, FolderOpen, RefreshCw, Volume2 } from "lucide-react";
import { useState } from "react";
import { cn } from "../lib/utils";

export default function Settings() {
  const [ram, setRam] = useState(4);
  const [sound, setSound] = useState(true);
  const [autoUpdate, setAutoUpdate] = useState(true);

  return (
    <div className="h-full p-8 space-y-6 max-w-3xl">
      <motion.header
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
      >
        <h1 className="font-display text-4xl font-bold tracking-tight">Paramètres</h1>
        <p className="text-text-secondary text-sm mt-2">
          Configure le launcher selon tes préférences.
        </p>
      </motion.header>

      {/* === RAM ALLOCATION === */}
      <motion.section
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.1 }}
        className="glass rounded-2xl p-6"
      >
        <div className="flex items-center gap-3 mb-5">
          <div className="w-9 h-9 rounded-lg bg-accent/15 ring-1 ring-accent/25 flex items-center justify-center">
            <Cpu size={16} className="text-accent" />
          </div>
          <div>
            <h2 className="text-sm font-bold text-text-primary">Mémoire allouée (RAM)</h2>
            <p className="text-xs text-text-muted">Quantité de RAM dédiée à Minecraft</p>
          </div>
        </div>

        <div className="flex items-center gap-4">
          <input
            type="range"
            min={2}
            max={12}
            value={ram}
            onChange={(e) => setRam(Number(e.target.value))}
            className="flex-1 h-1.5 bg-white/10 rounded-full appearance-none cursor-pointer slider-accent"
            style={{
              background: `linear-gradient(to right, #f4b942 0%, #f4b942 ${
                ((ram - 2) / 10) * 100
              }%, rgba(255,255,255,0.1) ${((ram - 2) / 10) * 100}%, rgba(255,255,255,0.1) 100%)`,
            }}
          />
          <div className="px-4 py-2 rounded-lg bg-bg-card font-mono font-bold text-accent text-sm tabular-nums min-w-[80px] text-center">
            {ram} GB
          </div>
        </div>

        <p className="text-[10px] text-text-muted mt-3">
          Recommandé : 4 GB pour vanilla, 6-8 GB avec shaders.
        </p>
      </motion.section>

      {/* === TOGGLES === */}
      <motion.section
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.15 }}
        className="glass rounded-2xl divide-y divide-border"
      >
        <ToggleRow
          icon={Volume2}
          title="Sons UI"
          desc="Active les bruits subtils sur clic et survol"
          checked={sound}
          onChange={setSound}
        />
        <ToggleRow
          icon={RefreshCw}
          title="Mises à jour automatiques"
          desc="Met à jour le launcher dès qu'une nouvelle version est disponible"
          checked={autoUpdate}
          onChange={setAutoUpdate}
        />
      </motion.section>

      {/* === FILES & PATHS === */}
      <motion.section
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.2 }}
        className="glass rounded-2xl p-6"
      >
        <h2 className="text-sm font-bold text-text-primary mb-4">Fichiers et dossiers</h2>
        <div className="space-y-2">
          <ActionRow icon={FolderOpen} label="Dossier .minecraft" path="%APPDATA%\.minecraft" />
          <ActionRow icon={FolderOpen} label="Dossier Lynara" path="%APPDATA%\.lynara" />
        </div>
      </motion.section>

      <p className="text-center text-[10px] text-text-muted pt-4">
        Lynara Launcher v2.0 · Made with ❤️ for the Lynara community
      </p>
    </div>
  );
}

function ToggleRow({
  icon: Icon,
  title,
  desc,
  checked,
  onChange,
}: {
  icon: typeof Cpu;
  title: string;
  desc: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <div className="flex items-center justify-between p-5">
      <div className="flex items-center gap-3 flex-1">
        <div className="w-9 h-9 rounded-lg bg-white/[0.04] ring-1 ring-white/[0.06] flex items-center justify-center flex-shrink-0">
          <Icon size={15} className="text-text-secondary" />
        </div>
        <div>
          <p className="text-sm font-semibold text-text-primary">{title}</p>
          <p className="text-xs text-text-muted">{desc}</p>
        </div>
      </div>
      <button
        onClick={() => onChange(!checked)}
        className={cn(
          "relative w-12 h-6 rounded-full transition-colors flex-shrink-0",
          checked ? "bg-accent" : "bg-white/10"
        )}
      >
        <motion.div
          animate={{ x: checked ? 24 : 2 }}
          transition={{ type: "spring", stiffness: 500, damping: 30 }}
          className="absolute top-0.5 w-5 h-5 rounded-full bg-white shadow-md"
        />
      </button>
    </div>
  );
}

function ActionRow({
  icon: Icon,
  label,
  path,
}: {
  icon: typeof FolderOpen;
  label: string;
  path: string;
}) {
  return (
    <button className="w-full flex items-center justify-between p-3 rounded-lg hover:bg-white/[0.03] transition-colors group">
      <div className="flex items-center gap-3">
        <Icon size={15} className="text-text-secondary" />
        <div className="text-left">
          <p className="text-sm font-medium text-text-primary">{label}</p>
          <p className="text-[10px] text-text-muted font-mono">{path}</p>
        </div>
      </div>
      <span className="text-[10px] text-text-muted group-hover:text-accent transition-colors">
        Ouvrir
      </span>
    </button>
  );
}
