import { goto } from "$app/navigation";
import { resolve } from "$app/paths";
import { getCurrentUserHandler, refreshSessionHandler } from "$lib/api/generated/client";
import { toAuthUser } from "$lib/features/auth/model/user-helpers";
import { auth } from "$lib/features/auth/state/auth";
import { fromStore } from "svelte/store";

export function useAuthBootstrap() {
  const authState = fromStore(auth);
  let syncedToken = $state<string | null>(null);
  let ensuringSession = $state(false);
  let syncingUser = $state(false);

  async function logoutAndRedirect() {
    auth.logout({ reason: "manual" });
    await goto(resolve("/login"));
  }

  $effect(() => {
    if (authState.current.isAuthenticated || ensuringSession) return;

    let cancelled = false;
    ensuringSession = true;

    void (async () => {
      try {
        const refreshed = await refreshSessionHandler();
        if (cancelled) return;
        auth.login(refreshed.token);
      } catch {
        if (cancelled) return;
        await logoutAndRedirect();
      } finally {
        ensuringSession = false;
      }
    })();

    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    const token = authState.current.token;
    const currentUser = authState.current.user;
    if (!token) {
      syncedToken = null;
      syncingUser = false;
      return;
    }
    if (token === syncedToken && currentUser) {
      syncingUser = false;
      return;
    }

    let cancelled = false;
    syncingUser = true;
    void (async () => {
      try {
        const me = await getCurrentUserHandler();
        if (cancelled) return;
        const mapped = toAuthUser(me);
        if (mapped) {
          auth.syncUser(mapped);
        }
      } finally {
        if (!cancelled) {
          syncedToken = token;
          syncingUser = false;
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  });

  return {
    get ensuringSession() {
      return ensuringSession;
    },
    get syncingUser() {
      return syncingUser;
    },
    logoutAndRedirect,
  };
}
