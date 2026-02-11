/* eslint-disable */
/* This file is generated - do not edit */

import { z } from "zod";

export const PatchCurrentUserPasswordRequest = z
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
  .object({ identifier: z.string().min(1).max(320), password: z.string().min(1).max(256) })
  .passthrough();
export const CreateSessionResponse = z
  .object({ expires_in: z.number().int().gte(0), token: z.string() })
  .passthrough();
export const AppSettings = z
  .object({ check_interval_secs: z.number().int().gte(0), welcome_message: z.string() })
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
export const UserResponse = z
  .object({
    avatar_url: z.union([z.string(), z.null()]).optional(),
    created_at: z.string().datetime({ offset: true }),
    display_name: z.string(),
    email: z.string(),
    id: z.string().uuid(),
    is_active: z.boolean(),
    metadata: z.unknown(),
    phone: z.union([z.string(), z.null()]).optional(),
    updated_at: z.string().datetime({ offset: true }),
    username: z.union([z.string(), z.null()]).optional(),
  })
  .passthrough();
export const CreateUserRequest = z
  .object({
    avatar_url: z.union([z.string(), z.null()]).optional(),
    display_name: z.string().min(1).max(128),
    email: z.string().min(3).max(320),
    metadata: z.unknown().optional(),
    phone: z.union([z.string(), z.null()]).optional(),
    username: z.union([z.string(), z.null()]).optional(),
  })
  .passthrough();
export const PatchUserRequest = z
  .object({
    avatar_url: z.union([z.string(), z.null()]),
    display_name: z.union([z.string(), z.null()]),
    email: z.union([z.string(), z.null()]),
    is_active: z.union([z.boolean(), z.null()]),
    metadata: z.unknown(),
    phone: z.union([z.string(), z.null()]),
    username: z.union([z.string(), z.null()]),
  })
  .partial()
  .passthrough();
