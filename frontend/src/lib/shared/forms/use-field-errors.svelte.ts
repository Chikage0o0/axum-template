import {
  detailsToFieldErrors,
  hasFieldError,
  mergeFieldErrors,
  toFieldErrorItems,
  type FieldErrorItem,
  type FieldErrors,
} from "./field-errors";

type FieldKey<TFieldKey extends string> = TFieldKey | string;

export function useFieldErrors<TFieldKey extends string>() {
  let errors = $state<FieldErrors>({});

  function setErrors(next: FieldErrors): void {
    errors = next;
  }

  function clearErrors(): void {
    errors = {};
  }

  function invalid(...keys: FieldKey<TFieldKey>[]): boolean {
    return hasFieldError(errors, ...keys);
  }

  function items(...keys: FieldKey<TFieldKey>[]): FieldErrorItem[] | undefined {
    return toFieldErrorItems(errors, ...keys);
  }

  function mergeApiDetails(details: unknown): void {
    errors = mergeFieldErrors(errors, detailsToFieldErrors(details));
  }

  return {
    get errors() {
      return errors;
    },
    setErrors,
    clearErrors,
    invalid,
    items,
    mergeApiDetails,
  };
}
