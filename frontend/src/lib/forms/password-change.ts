import { PatchCurrentUserPasswordRequest as PatchCurrentUserPasswordRequestSchema } from "$lib/api/generated/schemas";

import {
  mergeFieldErrors,
  type FieldErrors,
  zodErrorToFieldErrors,
} from "./field-errors";

type PasswordChangeFormInput = {
  currentPassword: string;
  newPassword: string;
  confirmPassword: string;
};

type PasswordChangePayload = {
  current_password: string;
  new_password: string;
};

export function validatePasswordChangeForm(
  input: PasswordChangeFormInput,
): { payload: PasswordChangePayload | null; errors: FieldErrors } {
  const currentPassword = input.currentPassword.trim();
  const newPassword = input.newPassword.trim();
  const confirmPassword = input.confirmPassword.trim();

  const parsed = PatchCurrentUserPasswordRequestSchema.safeParse({
    current_password: currentPassword,
    new_password: newPassword,
  });
  let errors = parsed.success ? {} : zodErrorToFieldErrors(parsed.error);

  if (newPassword !== confirmPassword) {
    errors = mergeFieldErrors(errors, {
      confirm_password: ["两次输入的新密码不一致"],
    });
  }

  if (Object.keys(errors).length > 0) {
    return { payload: null, errors };
  }

  return {
    payload: {
      current_password: currentPassword,
      new_password: newPassword,
    },
    errors: {},
  };
}
