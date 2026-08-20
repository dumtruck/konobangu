import { defineConfig } from "@playwright/test";
import { chromiumFixtureArgs } from "./tests/e2e/support/trusted-tls.mts";

const files = "**/*.{spec,test}.{ts,tsx,mts,cts,js,jsx,mjs,cjs}";

export default defineConfig({
  testDir: "./tests",
  fullyParallel: true,
  // Docker network lifecycle changes invalidate in-flight browser requests.
  // One worker also shares the real Rust/WebUI build across protocol cases.
  workers: 1,
  retries: 0,
  timeout: 30000,
  outputDir: "../../temp/verification/browser/playwright-results",
  reporter: "list",
  use: {
    browserName: "chromium",
    trace: "retain-on-failure",
    launchOptions: {
      args: [...chromiumFixtureArgs(), "--test-third-party-cookie-phaseout"],
    },
  },
  projects: [
    {
      name: "integration",
      testMatch: `**/integration/${files}`,
      // Candidate image validation runs after platform inputs exist. An
      // explicitly selected runtime image file without inputs finds no tests and
      // fails, rather than reporting a skipped required check as successful.
      testIgnore:
        process.env.KONOBANGU_RUNTIME_CONTEXT ||
        process.env.KONOBANGU_RUNTIME_IMAGE
          ? undefined
          : "**/integration/runtime-image/**",
    },
    { name: "e2e", testMatch: `**/e2e/${files}` },
  ],
});
