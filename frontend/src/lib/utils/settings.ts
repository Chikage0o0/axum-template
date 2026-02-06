import { apiFetchJson } from "$lib/utils/api";

export type CreateSessionRequest = { password: string };
export type CreateSessionResponse = { token: string; expires_in: number };

export type SettingsResponse = {
  app: { check_interval_secs: number; welcome_message: string };
  integrations: { example_api_base: string; example_api_key_is_set: boolean };
};

export type PatchSettingsRequest = {
  app?: { check_interval_secs?: number; welcome_message?: string };
  integrations?: { example_api_base?: string; example_api_key?: string };
};

export type PatchAdminPasswordRequest = {
  current_password: string;
  new_password: string;
};

export function createSession(
  payload: CreateSessionRequest,
): Promise<CreateSessionResponse> {
  return apiFetchJson<CreateSessionResponse>("/sessions", {
    method: "POST",
    body: JSON.stringify(payload),
  });
}

export function getSettings(): Promise<SettingsResponse> {
  return apiFetchJson<SettingsResponse>("/settings");
}

export function patchSettings(
  payload: PatchSettingsRequest,
): Promise<SettingsResponse> {
  return apiFetchJson<SettingsResponse>("/settings", {
    method: "PATCH",
    body: JSON.stringify(payload),
  });
}

export function changeAdminPassword(
  payload: PatchAdminPasswordRequest,
): Promise<void> {
  return apiFetchJson<void>("/security/admin-password", {
    method: "PATCH",
    body: JSON.stringify(payload),
  });
}
