import type { PatchUserRequest, UserResponse } from "$lib/api/generated/client";
import type { AuthUser } from "$lib/features/auth/model/auth-user";

export type User = UserResponse;

type EditableUserCurrent = Pick<
  User,
  "username" | "display_name" | "email" | "phone" | "avatar_url"
>;

export type EditableUserDraft = {
  username: string;
  display_name: string;
  email: string;
  phone: string;
  avatar_url: string;
};

export type CurrentUserEditable = Pick<User, "display_name" | "email" | "phone" | "avatar_url">;

export type CurrentUserDraft = {
  display_name: string;
  email: string;
  phone: string;
  avatar_url: string;
};

export function buildPatchUserPayload(
  current: EditableUserCurrent,
  draft: EditableUserDraft,
): { ok: true; payload: PatchUserRequest } | { ok: false; message: string } {
  const payload: PatchUserRequest = {};

  const username = draft.username.trim();
  const displayName = draft.display_name.trim();
  const email = draft.email.trim();
  const phone = draft.phone.trim();
  const avatarUrl = draft.avatar_url.trim();

  if (!displayName) {
    return { ok: false, message: "display_name 不能为空" };
  }
  if (!isEmail(email)) {
    return { ok: false, message: "email 格式不合法" };
  }

  if (username && username !== (current.username ?? "")) {
    payload.username = username;
  }
  if (displayName !== current.display_name) {
    payload.display_name = displayName;
  }
  if (email !== current.email) {
    payload.email = email;
  }
  if (phone && phone !== (current.phone ?? "")) {
    payload.phone = phone;
  }
  if (avatarUrl && avatarUrl !== (current.avatar_url ?? "")) {
    payload.avatar_url = avatarUrl;
  }

  if (Object.keys(payload).length === 0) {
    return { ok: false, message: "没有可更新的字段" };
  }

  return { ok: true, payload };
}

export function buildCurrentUserPatchPayload(
  current: CurrentUserEditable,
  draft: CurrentUserDraft,
): { ok: true; payload: PatchUserRequest } | { ok: false; message: string } {
  const payload: PatchUserRequest = {};

  const displayName = draft.display_name.trim();
  const email = draft.email.trim();
  const phone = draft.phone.trim();
  const avatarUrl = draft.avatar_url.trim();

  if (!displayName) {
    return { ok: false, message: "display_name 不能为空" };
  }
  if (!isEmail(email)) {
    return { ok: false, message: "email 格式不合法" };
  }

  if (displayName !== current.display_name) {
    payload.display_name = displayName;
  }
  if (email !== current.email) {
    payload.email = email;
  }
  if (phone && phone !== (current.phone ?? "")) {
    payload.phone = phone;
  }
  if (avatarUrl && avatarUrl !== (current.avatar_url ?? "")) {
    payload.avatar_url = avatarUrl;
  }

  if (Object.keys(payload).length === 0) {
    return { ok: false, message: "没有可更新的字段" };
  }

  return { ok: true, payload };
}

export function toAuthUser(user: Pick<User, "id" | "display_name" | "email">): AuthUser | null {
  const sub = user.id.trim();
  const displayName = user.display_name.trim();
  const email = user.email.trim();
  if (!sub || !displayName || !email) return null;
  return { sub, displayName, email };
}

function isEmail(input: string): boolean {
  const value = input.trim();
  if (!value) return false;
  const at = value.indexOf("@");
  if (at <= 0 || at >= value.length - 1) return false;
  return value.slice(at + 1).includes(".");
}
