import {
  Home as HomeIcon,
  Newspaper,
  Palette,
  Settings as SettingsIcon,
  LogOut,
} from "lucide-react";
import { motion } from "framer-motion";
import { Page } from "../../App";
import { cn } from "../../lib/utils";
import LogoMark from "../ui/LogoMark";
import { useAuth } from "../../stores/auth";
import { logout as logoutApi } from "../../lib/auth";

interface Props {
  currentPage: Page;
  onNavigate: (page: Page) => void;
}

const navItems: { id: Page; label: string; icon: typeof HomeIcon }[] = [
  { id: "home", label: "Accueil", icon: HomeIcon },
  { id: "news", label: "News", icon: Newspaper },
  { id: "shaders", label: "Shaders", icon: Palette },
  { id: "settings", label: "Paramètres", icon: SettingsIcon },
];

export default function Sidebar({ currentPage, onNavigate }: Props) {
  const user = useAuth((s) => s.user);
  const setUser = useAuth((s) => s.setUser);

  const handleLogout = async () => {
    try {
      await logoutApi();
    } catch (e) {
      console.warn("Logout error:", e);
    }
    setUser(null);
  };

  const username = user?.username ?? "Joueur";
  const initial = username[0]?.toUpperCase() ?? "?";

  // Crafatar URL for the player's head (uses real UUID when available)
  const headUrl = user?.uuid
    ? `https://crafatar.com/avatars/${user.uuid}?size=64&overlay`
    : null;

  return (
    <aside className="w-60 bg-bg-deep/60 backdrop-blur-xl border-r border-border flex flex-col flex-shrink-0 relative z-10">
      {/* === Brand mark (top) === */}
      <div className="px-5 pt-5 pb-4">
        <div className="flex items-center gap-3">
          <LogoMark size={36} glow />
          <div>
            <p className="font-display text-lg font-bold tracking-tight leading-none text-gradient-gold">
              LYNARA
            </p>
            <p className="text-[9px] text-text-muted uppercase tracking-[0.2em] font-bold mt-1">
              Launcher v2.0
            </p>
          </div>
        </div>
      </div>

      {/* === Profile zone === */}
      <div className="px-5 pb-5 border-b border-border">
        <div className="flex items-center gap-3 p-3 rounded-xl bg-white/[0.03] border border-white/[0.05]">
          <div className="relative">
            {headUrl ? (
              <img
                src={headUrl}
                alt={username}
                className="w-10 h-10 rounded-lg shadow-accent"
                style={{ imageRendering: "pixelated" }}
                onError={(e) => {
                  (e.currentTarget as HTMLImageElement).style.display = "none";
                }}
              />
            ) : (
              <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-accent to-accent-dark flex items-center justify-center shadow-accent">
                <span className="text-bg-deep text-base font-black leading-none">
                  {initial}
                </span>
              </div>
            )}
            <div className="absolute -bottom-0.5 -right-0.5 w-3 h-3 rounded-full bg-success border-2 border-bg-deep" />
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-sm font-semibold text-text-primary truncate">{username}</p>
            <p className="text-[9px] text-success uppercase tracking-widest font-bold">
              En ligne
            </p>
          </div>
        </div>
      </div>

      {/* === Nav === */}
      <nav className="flex-1 p-3 space-y-1">
        {navItems.map((item) => {
          const Icon = item.icon;
          const active = currentPage === item.id;
          return (
            <button
              key={item.id}
              onClick={() => onNavigate(item.id)}
              className={cn(
                "w-full flex items-center gap-3 px-3.5 py-2.5 rounded-lg text-sm font-medium transition-all relative group",
                active
                  ? "text-text-primary"
                  : "text-text-secondary hover:text-text-primary hover:bg-white/[0.04]"
              )}
            >
              {active && (
                <motion.div
                  layoutId="activeNav"
                  className="absolute inset-0 bg-gradient-to-r from-accent/15 via-accent/[0.03] to-transparent border-l-2 border-accent rounded-lg"
                  transition={{ type: "spring", stiffness: 400, damping: 30 }}
                />
              )}
              <Icon
                size={17}
                className={cn(
                  "relative z-10 transition-colors",
                  active && "text-accent"
                )}
              />
              <span className="relative z-10">{item.label}</span>
            </button>
          );
        })}
      </nav>

      {/* === Server status pill === */}
      <div className="p-3 border-t border-border space-y-2">
        <div className="flex items-center gap-2.5 px-3 py-2.5 rounded-lg bg-success/10 border border-success/20">
          <div className="relative flex-shrink-0">
            <div className="w-2 h-2 rounded-full bg-success" />
            <div className="absolute inset-0 w-2 h-2 rounded-full bg-success animate-ping opacity-75" />
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-[10px] font-bold text-success uppercase tracking-wider">
              Serveur en ligne
            </p>
            <p className="text-[10px] text-text-muted font-mono truncate">
              play.lynara.fr · 8ms
            </p>
          </div>
        </div>

        <button
          onClick={handleLogout}
          className="w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-text-muted hover:text-danger hover:bg-danger/10 transition-colors text-xs font-medium"
        >
          <LogOut size={14} />
          <span>Déconnexion</span>
        </button>
      </div>
    </aside>
  );
}
