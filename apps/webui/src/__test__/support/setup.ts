import "temporal-polyfill/global";
import { afterEach, vi } from "vitest";

// One cleanup contract for every suite: no file restores spies, globals or
// mounted React trees on its own.
afterEach(async () => {
  if (typeof document !== "undefined")
    (await import("@testing-library/react")).cleanup();
  vi.useRealTimers();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
  vi.unstubAllEnvs();
});
