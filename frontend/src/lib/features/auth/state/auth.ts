import { writable } from "svelte/store";
import type { AuthUser } from "$lib/features/auth/model/auth-user";
import { createPermissionSet } from "$lib/features/auth/model/permission-set";

export type Flash = { title: string; message: string };

export type AuthState = {
  isAuthenticated: boolean;
  token: string | null;
  user: AuthUser | null;
  permissions: string[];
  flash: Flash | null;
};

const initialState: AuthState = {
  isAuthenticated: false,
  token: null,
  user: null,
  permissions: [],
  flash: null,
};

let currentState = initialState;

const store = writable<AuthState>(initialState);
store.subscribe((next) => {
  currentState = next;
});

export const auth = {
  subscribe: store.subscribe,
  readTokenFromStorage() {
    return currentState.token;
  },
  login(token: string) {
    store.set({
      isAuthenticated: true,
      token,
      user: null,
      permissions: [],
      flash: null,
    });
  },
  logout(opts?: { reason?: "expired" | "manual" }) {
    const flash =
      opts?.reason === "expired"
        ? { title: "登录失效", message: "令牌已过期或无效，请重新登录" }
        : null;

    store.set({ isAuthenticated: false, token: null, user: null, permissions: [], flash });
  },
  clearFlash() {
    store.update((s) => ({ ...s, flash: null }));
  },
  syncUser(user: AuthUser | null) {
    store.update((s) => {
      if (!s.isAuthenticated || !s.token) return s;
      if (!user) {
        return { ...s, user: null, permissions: [] };
      }
      return { ...s, user, permissions: user.permissions };
    });
  },
  can(permCode: string) {
    return createPermissionSet(currentState.permissions).can(permCode);
  },
};
