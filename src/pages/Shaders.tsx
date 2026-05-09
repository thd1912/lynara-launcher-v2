import { motion } from "framer-motion";
import { Palette, Download, Eye, Power } from "lucide-react";
import { useState } from "react";
import { cn } from "../lib/utils";

const mockShaders = [
  { id: "complementary", name: "Complementary Shaders", author: "EminGT", popular: true, installed: true },
  { id: "bsl", name: "BSL Shaders", author: "Capt Tatsu", popular: true, installed: false },
  { id: "sildur", name: "Sildur's Vibrant", author: "Sildur", popular: false, installed: false },
  { id: "seus", name: "SEUS Renewed", author: "Sonic Ether", popular: false, installed: false },
];

export default function Shaders() {
  const [enabled, setEnabled] = useState(true);
  const [active, setActive] = useState("complementary");

  return (
    <div className="h-full p-8 space-y-6">
      <motion.header
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
      >
        <h1 className="font-display text-4xl font-bold tracking-tight">Shaders</h1>
        <p className="text-text-secondary text-sm mt-2">
          Améliore ton expérience visuelle avec des shaders premium.
        </p>
      </motion.header>

      {/* Master toggle card */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.1 }}
        className="glass rounded-2xl p-5 flex items-center justify-between"
      >
        <div className="flex items-center gap-4">
          <div className={cn(
            "w-11 h-11 rounded-xl flex items-center justify-center transition-colors",
            enabled ? "bg-accent/15 ring-1 ring-accent/25" : "bg-white/[0.04]"
          )}>
            <Power size={18} className={enabled ? "text-accent" : "text-text-muted"} />
          </div>
          <div>
            <p className="text-sm font-semibold text-text-primary">
              Shaders {enabled ? "activés" : "désactivés"}
            </p>
            <p className="text-xs text-text-muted">
              {enabled ? "Iris + Sodium installés" : "Active pour utiliser des shaders"}
            </p>
          </div>
        </div>

        <button
          onClick={() => setEnabled(!enabled)}
          className={cn(
            "relative w-12 h-6 rounded-full transition-colors",
            enabled ? "bg-accent" : "bg-white/10"
          )}
        >
          <motion.div
            animate={{ x: enabled ? 24 : 2 }}
            transition={{ type: "spring", stiffness: 500, damping: 30 }}
            className="absolute top-0.5 w-5 h-5 rounded-full bg-white shadow-md"
          />
        </button>
      </motion.div>

      {/* Shader list */}
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.15 }}
      >
        <h2 className="text-xs font-bold text-text-muted uppercase tracking-widest mb-3">
          Packs disponibles
        </h2>
        <div className="grid grid-cols-2 gap-3">
          {mockShaders.map((s, i) => (
            <ShaderCard
              key={s.id}
              shader={s}
              active={active === s.id}
              onSelect={() => setActive(s.id)}
              delay={0.05 * i}
            />
          ))}
        </div>
      </motion.div>
    </div>
  );
}

function ShaderCard({ shader, active, onSelect, delay }: {
  shader: typeof mockShaders[0];
  active: boolean;
  onSelect: () => void;
  delay: number;
}) {
  return (
    <motion.button
      onClick={onSelect}
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3, delay }}
      whileHover={{ y: -2 }}
      className={cn(
        "glass rounded-xl p-4 text-left transition-all relative overflow-hidden",
        active && "ring-2 ring-accent shadow-accent"
      )}
    >
      {/* Preview gradient */}
      <div className="h-24 rounded-lg mb-3 overflow-hidden relative bg-gradient-to-br from-bg-card to-bg-elevated">
        <div className="absolute inset-0 bg-[radial-gradient(circle_at_30%_40%,rgba(244,185,66,0.15),transparent_60%),radial-gradient(circle_at_70%_60%,rgba(140,80,180,0.12),transparent_60%)]" />
        <div className="absolute inset-0 flex items-center justify-center">
          <Palette size={28} className="text-text-muted/40" />
        </div>
        {shader.popular && (
          <div className="absolute top-2 right-2 px-2 py-0.5 rounded text-[9px] font-bold bg-accent/20 text-accent border border-accent/30">
            POPULAIRE
          </div>
        )}
      </div>

      <div className="flex items-start justify-between gap-2">
        <div className="flex-1 min-w-0">
          <p className="text-sm font-semibold text-text-primary truncate">{shader.name}</p>
          <p className="text-[10px] text-text-muted">par {shader.author}</p>
        </div>

        {shader.installed ? (
          <div className="flex items-center gap-1 px-2 py-1 rounded-md bg-success/15 text-success text-[10px] font-bold">
            <Eye size={10} />
            <span>Installé</span>
          </div>
        ) : (
          <div className="flex items-center gap-1 px-2 py-1 rounded-md bg-white/5 text-text-secondary text-[10px] font-medium">
            <Download size={10} />
            <span>{(Math.random() * 8 + 2).toFixed(1)} MB</span>
          </div>
        )}
      </div>
    </motion.button>
  );
}
