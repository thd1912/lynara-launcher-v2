import { useEffect, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import TitleBar from "./components/layout/TitleBar";
import Sidebar from "./components/layout/Sidebar";
import AnimatedBackground from "./components/layout/AnimatedBackground";
import Home from "./pages/Home";
import News from "./pages/News";
import Shaders from "./pages/Shaders";
import Settings from "./pages/Settings";
import Login from "./pages/Login";
import { useAuth } from "./stores/auth";
import { getCurrentUser } from "./lib/auth";
import "./index.css";
import UpdateChecker from "./components/UpdateChecker";

export type Page = "home" | "news" | "shaders" | "settings";

function App() {
  const [currentPage, setCurrentPage] = useState<Page>("home");
  const user = useAuth((s) => s.user);
  const initialized = useAuth((s) => s.initialized);
  const setUser = useAuth((s) => s.setUser);
  const setInitialized = useAuth((s) => s.setInitialized);

  // Check OS keyring for stored session at startup (skip login if user is already authenticated)
  useEffect(() => {
    getCurrentUser()
      .then((u) => {
        if (u) setUser(u);
      })
      .catch((e) => console.warn("No stored session:", e))
      .finally(() => setInitialized(true));
  }, [setUser, setInitialized]);

  // Don't show anything until initialization completes (avoids login flash if user is logged in)
  if (!initialized) {
    return (
      <>
        <UpdateChecker />
        <div className="h-screen w-screen flex flex-col text-text-primary overflow-hidden relative">
          <AnimatedBackground />
          <TitleBar />
          <main className="flex-1 flex items-center justify-center">
            <div className="w-2 h-2 rounded-full bg-accent animate-pulse" />
          </main>
        </div>
      </>
    );
  }

  // === Not authenticated → show Login ===
  if (!user) {
    return (
      <>
        <UpdateChecker />
        <div className="h-screen w-screen flex flex-col text-text-primary overflow-hidden relative">
          <AnimatedBackground />
          <TitleBar />
          <main className="flex-1 relative overflow-hidden">
            <Login />
          </main>
        </div>
      </>
    );
  }

  // === Authenticated → show full app ===
  return (
    <>
      <UpdateChecker />
      <div className="h-screen w-screen flex flex-col text-text-primary overflow-hidden relative">
        <AnimatedBackground />
        <TitleBar />

        <div className="flex-1 flex overflow-hidden relative">
          <Sidebar currentPage={currentPage} onNavigate={setCurrentPage} />

          <main className="flex-1 relative overflow-hidden">
            <AnimatePresence mode="wait">
              <motion.div
                key={currentPage}
                initial={{ opacity: 0, y: 8 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -8 }}
                transition={{ duration: 0.3, ease: "easeOut" }}
                className="relative h-full overflow-y-auto"
              >
                {currentPage === "home" && <Home />}
                {currentPage === "news" && <News />}
                {currentPage === "shaders" && <Shaders />}
                {currentPage === "settings" && <Settings />}
              </motion.div>
            </AnimatePresence>
          </main>
        </div>
      </div>
    </>
  );
}

export default App;

