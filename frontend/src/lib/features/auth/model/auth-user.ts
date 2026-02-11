import type { AuthRole } from "$lib/features/auth/model/token-role";

export type AuthUser = {
  sub: string;
  displayName: string;
  email: string;
  avatarUrl: string;
  role?: AuthRole;
};
