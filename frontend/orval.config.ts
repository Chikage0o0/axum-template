import { defineConfig } from "orval";

export default defineConfig({
  api: {
    input: "../docs/openapi.json",
    output: {
      target: "src/lib/api/generated/client.ts",
      client: "fetch",
      mode: "single",
      override: {
        mutator: {
          path: "./src/lib/api/mutator.ts",
          name: "apiClient",
        },
        fetch: {
          includeHttpResponseReturnType: false,
        },
      },
    },
  },
});
