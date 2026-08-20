import { test as base } from "@playwright/test";
import { startDeployment } from "./deployment.mts";

type Deployment = Awaited<ReturnType<typeof startDeployment>>;

// Only suites importing this fixture start the WebUI/API/IdP deployment, after discovery.
export const test = base.extend<{}, { deployment: Deployment }>({
  deployment: [
    // biome-ignore lint/correctness/noEmptyPattern: Playwright requires destructuring to declare fixture dependencies.
    async ({}, use) => {
      await using deployment = await startDeployment();
      await use(deployment);
    },
    { scope: "worker", auto: true, timeout: 15 * 60 * 1000 },
  ],
});
