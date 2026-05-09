import { motion } from "framer-motion";
import { Newspaper } from "lucide-react";

export default function News() {
  return (
    <div className="h-full p-8">
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
        className="space-y-6"
      >
        <header>
          <h1 className="font-display text-4xl font-bold tracking-tight">News & Annonces</h1>
          <p className="text-text-secondary text-sm mt-2">Tout ce qui se passe sur Lynara, en temps réel.</p>
        </header>

        <div className="glass rounded-2xl p-12 text-center">
          <div className="w-16 h-16 rounded-2xl bg-accent/15 flex items-center justify-center mx-auto mb-4">
            <Newspaper size={26} className="text-accent" />
          </div>
          <p className="text-text-secondary text-sm mb-1">Bientôt disponible</p>
          <p className="text-text-muted text-xs">
            Le système de news sera connecté à <span className="font-mono text-accent">lynara.fr/launcher/news.json</span>
          </p>
        </div>
      </motion.div>
    </div>
  );
}
