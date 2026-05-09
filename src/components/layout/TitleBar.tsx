import { Minus, Square, X } from "lucide-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import LogoMark from "../ui/LogoMark";

export default function TitleBar() {
  const w = getCurrentWindow();

  return (
    <div
      data-tauri-drag-region
      className="h-9 flex items-center justify-between bg-bg-deep/80 backdrop-blur-xl border-b border-border select-none flex-shrink-0 relative z-50"
    >
      {/* Left: Logo + name (drag area) */}
      <div className="flex items-center gap-2.5 px-4 pointer-events-none" data-tauri-drag-region>
        <LogoMark size={20} glow />
        <span className="text-text-secondary text-[10px] font-bold tracking-[0.22em]">
          LYNARA <span className="text-accent">LAUNCHER</span>
        </span>
      </div>

      {/* Right: Window controls */}
      <div className="flex h-full">
        <button
          onClick={() => w.minimize()}
          className="w-11 h-full flex items-center justify-center hover:bg-white/5 transition-colors"
        >
          <Minus size={13} className="text-text-secondary" />
        </button>
        <button
          onClick={() => w.toggleMaximize()}
          className="w-11 h-full flex items-center justify-center hover:bg-white/5 transition-colors"
        >
          <Square size={11} className="text-text-secondary" />
        </button>
        <button
          onClick={() => w.close()}
          className="w-11 h-full flex items-center justify-center hover:bg-danger/90 transition-colors group"
        >
          <X size={13} className="text-text-secondary group-hover:text-white" />
        </button>
      </div>
    </div>
  );
}
