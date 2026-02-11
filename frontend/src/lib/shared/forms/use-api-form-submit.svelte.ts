import { ApiError } from "$lib/api/mutator";

type SubmitOptions = {
  setSubmitting?: (value: boolean) => void;
  onFieldErrors?: (details: unknown) => boolean | void;
  onUnknownError?: (error: unknown) => void;
};

export function useApiFormSubmit() {
  async function run(action: () => Promise<void>, options: SubmitOptions = {}): Promise<void> {
    const { setSubmitting, onFieldErrors, onUnknownError } = options;

    setSubmitting?.(true);
    try {
      await action();
    } catch (error) {
      if (error instanceof ApiError) {
        const handled = onFieldErrors?.(error.body?.details) ?? false;
        if (handled) {
          return;
        }
      }
      onUnknownError?.(error);
    } finally {
      setSubmitting?.(false);
    }
  }

  return { run };
}
