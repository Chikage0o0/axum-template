import { describe, expect, it } from "bun:test";
import { ApiError } from "$lib/api/mutator";
import { useApiFormSubmit } from "./use-api-form-submit.svelte";

describe("useApiFormSubmit", () => {
  it("should map ApiError details into field errors", async () => {
    const submit = useApiFormSubmit();
    let fieldDetails: unknown = null;
    let unknownCalled = false;

    await submit.run(
      async () => {
        throw new ApiError("bad request", 400, {
          details: { password: ["长度不足"] },
        });
      },
      {
        onFieldErrors(details) {
          fieldDetails = details;
          return true;
        },
        onUnknownError() {
          unknownCalled = true;
        },
      },
    );

    expect(fieldDetails).toEqual({ password: ["长度不足"] });
    expect(unknownCalled).toBeFalse();
  });

  it("should always reset submitting in finally", async () => {
    const submit = useApiFormSubmit();
    const states: boolean[] = [];

    await submit.run(
      async () => {
        throw new Error("network down");
      },
      {
        setSubmitting(next) {
          states.push(next);
        },
        onUnknownError() {
          // noop
        },
      },
    );

    expect(states).toEqual([true, false]);
  });
});
