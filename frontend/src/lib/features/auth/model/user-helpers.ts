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

type AdminEditableUserCurrent = EditableUserCurrent & Pick<User, "is_active">;
type AdminEditableUserDraft = EditableUserDraft & Pick<User, "is_active">;

type BuildUserPatchPayloadInput =
  | {
      mode: "admin-edit";
      current: AdminEditableUserCurrent;
      draft: AdminEditableUserDraft;
    }
  | {
      mode: "self-edit";
      current: CurrentUserEditable;
      draft: CurrentUserDraft;
    };

type BuildUserPatchPayloadResult =
  | { ok: true; payload: PatchUserRequest }
  | { ok: false; message: string };

export function buildUserPatchPayload(
  input: BuildUserPatchPayloadInput,
): BuildUserPatchPayloadResult {
  const payload: PatchUserRequest = {};

  const displayName = input.draft.display_name.trim();
  const email = input.draft.email.trim();
  const phone = input.draft.phone.trim();
  const avatarUrl = input.draft.avatar_url.trim();

  if (!displayName) {
    return { ok: false, message: "display_name 不能为空" };
  }
  if (!isEmail(email)) {
    return { ok: false, message: "email 格式不合法" };
  }

  if (displayName !== input.current.display_name) {
    payload.display_name = displayName;
  }
  if (email !== input.current.email) {
    payload.email = email;
  }

  const nextPhone = normalizeOptionalPatchField(phone, input.current.phone);
  if (typeof nextPhone !== "undefined") {
    payload.phone = nextPhone;
  }

  const nextAvatarUrl = normalizeOptionalPatchField(avatarUrl, input.current.avatar_url);
  if (typeof nextAvatarUrl !== "undefined") {
    payload.avatar_url = nextAvatarUrl;
  }

  if (input.mode === "admin-edit") {
    const username = input.draft.username.trim();
    const nextUsername = normalizeOptionalPatchField(username, input.current.username);
    if (typeof nextUsername !== "undefined") {
      payload.username = nextUsername;
    }
    if (input.draft.is_active !== input.current.is_active) {
      payload.is_active = input.draft.is_active;
    }
  }

  if (Object.keys(payload).length === 0) {
    return { ok: false, message: "没有可更新的字段" };
  }

  return { ok: true, payload };
}

export function buildPatchUserPayload(
  current: EditableUserCurrent,
  draft: EditableUserDraft,
): { ok: true; payload: PatchUserRequest } | { ok: false; message: string } {
  const result = buildUserPatchPayload({
    mode: "admin-edit",
    current: {
      ...current,
      is_active: true,
    },
    draft: {
      ...draft,
      is_active: true,
    },
  });
  return result;
}

export function buildCurrentUserPatchPayload(
  current: CurrentUserEditable,
  draft: CurrentUserDraft,
): { ok: true; payload: PatchUserRequest } | { ok: false; message: string } {
  return buildUserPatchPayload({ mode: "self-edit", current, draft });
}

export function toAuthUser(
  user: Pick<User, "id" | "display_name" | "email" | "avatar_url"> & {
    permissions?: unknown;
  },
): AuthUser | null {
  const sub = user.id.trim();
  const displayName = user.display_name.trim();
  const email = user.email.trim();
  const avatarUrl = user.avatar_url?.trim() ?? "";
  if (!sub || !displayName || !email) return null;
  return {
    sub,
    displayName,
    email,
    avatarUrl,
    permissions: normalizePermissions(user.permissions),
  };
}

function normalizePermissions(raw: unknown): string[] {
  if (!Array.isArray(raw)) return [];
  return raw
    .filter((item): item is string => typeof item === "string")
    .map((item) => item.trim())
    .filter((item) => item.length > 0);
}

function isEmail(input: string): boolean {
  const value = input.trim();
  if (!value) return false;
  const at = value.indexOf("@");
  if (at <= 0 || at >= value.length - 1) return false;
  return value.slice(at + 1).includes(".");
}

function normalizeOptionalPatchField(
  draftValue: string,
  currentValue: string | null | undefined,
): string | null | undefined {
  const current = currentValue?.trim() ?? "";
  if (draftValue === current) {
    return undefined;
  }
  if (!draftValue) {
    return null;
  }
  return draftValue;
}
