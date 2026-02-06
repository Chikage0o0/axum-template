import type { ZodError, core } from "zod";
import { setupZodErrorMap, translateZodIssue } from "./zod-error-map";

export type FieldErrors = Record<string, string[]>;
export type FieldErrorItem = { message: string };

setupZodErrorMap();

function pushFieldError(map: FieldErrors, key: string, message: string): void {
  if (!message) return;
  if (!map[key]) {
    map[key] = [message];
    return;
  }
  map[key].push(message);
}

function normalizeIssuePath(path: readonly PropertyKey[]): string {
  if (path.length === 0) return "_form";
  return path
    .map((segment) => (typeof segment === "symbol" ? segment.toString() : String(segment)))
    .join(".");
}

export function zodIssuesToFieldErrors(issues: core.$ZodIssue[]): FieldErrors {
  const out: FieldErrors = {};
  for (const issue of issues) {
    pushFieldError(out, normalizeIssuePath(issue.path), translateZodIssue(issue));
  }
  return out;
}

export function zodErrorToFieldErrors(error: ZodError): FieldErrors {
  return zodIssuesToFieldErrors(error.issues);
}

export function detailsToFieldErrors(details: unknown): FieldErrors {
  if (!details || typeof details !== "object" || Array.isArray(details)) {
    return {};
  }

  const out: FieldErrors = {};
  for (const [key, value] of Object.entries(details)) {
    if (typeof value === "string") {
      pushFieldError(out, key, value);
      continue;
    }
    if (!Array.isArray(value)) continue;
    for (const item of value) {
      if (typeof item === "string" && item) {
        pushFieldError(out, key, item);
      }
    }
  }

  return out;
}

export function toFieldErrorItems(
  errors: FieldErrors,
  ...keys: string[]
): FieldErrorItem[] | undefined {
  const messages: string[] = [];

  for (const key of keys) {
    const current = errors[key];
    if (!current || current.length === 0) continue;
    for (const message of current) {
      if (!messages.includes(message)) {
        messages.push(message);
      }
    }
  }

  if (messages.length === 0) return undefined;
  return messages.map((message) => ({ message }));
}

export function hasFieldError(errors: FieldErrors, ...keys: string[]): boolean {
  for (const key of keys) {
    const current = errors[key];
    if (current && current.length > 0) {
      return true;
    }
  }
  return false;
}

export function mergeFieldErrors(...parts: FieldErrors[]): FieldErrors {
  const out: FieldErrors = {};
  for (const part of parts) {
    for (const [key, messages] of Object.entries(part)) {
      for (const message of messages) {
        pushFieldError(out, key, message);
      }
    }
  }
  return out;
}
