/* eslint-disable */
/* This file is generated - do not edit */

import { z } from "zod";

export const PatchAdminPasswordRequest = z
  .object({
    current_password: z.string().min(1).max(256),
    new_password: z.string().min(8).max(256),
  })
  .passthrough();
export const ErrorResponseBody = z
  .object({
    code: z.number().int().gte(0),
    details: z.unknown().optional(),
    message: z.string(),
    request_id: z.string(),
  })
  .passthrough();
export const CreateSessionRequest = z
  .object({ password: z.string().min(1).max(256) })
  .passthrough();
export const CreateSessionResponse = z
  .object({ expires_in: z.number().int().gte(0), token: z.string() })
  .passthrough();
export const AppSettings = z
  .object({
    check_interval_secs: z.number().int().gte(0),
    welcome_message: z.string(),
  })
  .passthrough();
export const IntegrationsSettings = z
  .object({ example_api_base: z.string(), example_api_key_is_set: z.boolean() })
  .passthrough();
export const SettingsResponse = z
  .object({ app: AppSettings, integrations: IntegrationsSettings })
  .passthrough();
export const PatchAppSettings = z
  .object({
    check_interval_secs: z.union([z.number(), z.null()]),
    welcome_message: z.union([z.string(), z.null()]),
  })
  .partial()
  .passthrough();
export const PatchIntegrationsSettings = z
  .object({
    example_api_base: z.union([z.string(), z.null()]),
    example_api_key: z.union([z.string(), z.null()]),
  })
  .partial()
  .passthrough();
export const PatchSettingsRequest = z
  .object({
    app: z.union([z.null(), PatchAppSettings]),
    integrations: z.union([z.null(), PatchIntegrationsSettings]),
  })
  .partial()
  .passthrough();
