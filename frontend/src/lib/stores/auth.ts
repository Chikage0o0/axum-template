import { writable } from "svelte/store";
import type { AuthUser } from "$lib/types/auth-user";

export type Flash = { title: string; message: string };

export type AuthState = {
  isAuthenticated: boolean;
  token: string | null;
  user: AuthUser | null;
  flash: Flash | null;
};

function readTokenFromStorage(): string | null {
  try {
    return localStorage.getItem("token");
  } catch {
    return null;
  }
}

const initialToken = typeof window === "undefined" ? null : readTokenFromStorage();

const store = writable<AuthState>({
  isAuthenticated: Boolean(initialToken),
  token: initialToken,
  user: null,
  flash: null,
});

export const auth = {
  subscribe: store.subscribe,
  readTokenFromStorage,
  login(token: string) {
    try {
      localStorage.setItem("token", token);
    } catch {
      // ignore
    }
    store.set({
      isAuthenticated: true,
      token,
      user: null,
      flash: null,
    });
  },
  logout(opts?: { reason?: "expired" | "manual" }) {
    try {
      localStorage.removeItem("token");
    } catch {
      // ignore
    }

    const flash =
      opts?.reason === "expired"
        ? { title: "登录失效", message: "令牌已过期或无效，请重新登录" }
        : null;

    store.set({ isAuthenticated: false, token: null, user: null, flash });
  },
  clearFlash() {
    store.update((s) => ({ ...s, flash: null }));
  },
  syncUser(user: AuthUser | null) {
    store.update((s) => {
      if (!s.isAuthenticated || !s.token) return s;
      return { ...s, user };
    });
  },
};
