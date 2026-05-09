import { create } from "zustand";
import { UserProfile } from "../lib/auth";

interface AuthStore {
  user: UserProfile | null;
  /** Whether we've checked the keyring at startup. Used to avoid login flash. */
  initialized: boolean;
  setUser: (user: UserProfile | null) => void;
  setInitialized: (v: boolean) => void;
}

export const useAuth = create<AuthStore>((set) => ({
  user: null,
  initialized: false,
  setUser: (user) => set({ user }),
  setInitialized: (v) => set({ initialized: v }),
}));