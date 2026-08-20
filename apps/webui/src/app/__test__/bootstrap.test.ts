import { afterEach, describe, expect, it, vi } from "vitest";
import {
  createKonobanguBootstrapEnvironmentForTest,
  createOidcAuthSetupForTest,
} from "@/app/environment/test";
import { type BootstrapStages, createBootstrap } from "../bootstrap";
import { parseRuntimeConfig } from "../config/runtime";
import {
  ensurePolyfills,
  ensureResourceManagement,
  ensureTemporal,
} from "../polyfills";

const controllers = new Set<ReturnType<typeof createBootstrap>>();
afterEach(() => {
  for (const controller of controllers) controller.dispose();
  controllers.clear();
});
const testConfig = parseRuntimeConfig(
  { version: 1, apiBaseUrl: "/api/" },
  { origin: "http://localhost", allowInsecureLoopback: true },
);
const testAuthConfig = {
  provider: "oidc",
  resource: {
    audience: "urn:konobangu:api",
    requiredScopes: ["konobangu:api"],
  },
  projection: {
    clientId: "konobangu-webui",
    issuerUrl: "https://idp.example",
    redirectUrl: "http://localhost/auth/callback",
    pkceEnabled: true,
    generatedAt: 1,
  },
} as const;
const testAuthSetup = createOidcAuthSetupForTest("http://localhost");
/**
 * The environment and stages are required, so the fixture builds complete
 * objects rather than a partial options bag; only the pieces a case drives
 * are overridden.
 */
function fixture(
  entry: Parameters<typeof createBootstrap>[0],
  ensure?: (signal: AbortSignal) => Promise<void>,
  overrides: Partial<BootstrapStages> & { deadlineMs?: number } = {},
) {
  const { deadlineMs, ...stageOverrides } = overrides;
  const environment = createKonobanguBootstrapEnvironmentForTest({
    ...(ensure ? { ensurePolyfills: ensure } : {}),
    ...(deadlineMs === undefined ? {} : { deadlineMs }),
  });
  const stages: BootstrapStages = {
    loadConfig: async () => testConfig,
    loadAuthConfig: async () => testAuthConfig,
    prepareAuth: async () => testAuthSetup,
    ...stageOverrides,
  };
  const controller = createBootstrap(entry, environment, stages);
  controllers.add(controller);
  return controller;
}
function application(dispose = vi.fn()) {
  return { startApp: vi.fn(() => ({ [Symbol.dispose]: dispose })) };
}
describe("application startup barrier", () => {
  it("creates and cancels the controller without any resource primitives", async () => {
    const pending = Promise.withResolvers<void>();
    const entry = vi.fn(async () => application());
    let done: Promise<void>;
    let keys: symbol[];
    try {
      // Model the pre-polyfill phase, including an unavailable protocol symbol.
      vi.stubGlobal("Symbol", undefined);
      vi.stubGlobal("DisposableStack", undefined);
      const boot = fixture(entry, () => pending.promise);
      keys = Reflect.ownKeys(boot).filter(
        (key): key is symbol => typeof key === "symbol",
      );
      done = boot.start();
      boot.dispose();
      boot.dispose();
    } finally {
      vi.unstubAllGlobals();
    }
    pending.resolve();
    await done;
    expect(keys).toEqual([]);
    expect(entry).not.toHaveBeenCalled();
  });
  it("preserves native resource management without loading its polyfill", async () => {
    const capabilities = {
      Symbol: { dispose: Symbol.dispose, asyncDispose: Symbol.asyncDispose },
      DisposableStack,
      AsyncDisposableStack,
      SuppressedError,
    };
    const load = vi.fn();
    await ensureResourceManagement(capabilities, load);
    expect(load).not.toHaveBeenCalled();
    expect(capabilities.Symbol.dispose).toBe(Symbol.dispose);
  });
  it("waits for missing resource primitives before evaluating SDK owners", async () => {
    const capabilities: Parameters<typeof ensureResourceManagement>[0] = {
      Symbol: {},
    };
    const pending = Promise.withResolvers<void>();
    const entry = vi.fn(async () => application());
    const boot = fixture(entry, () =>
      ensureResourceManagement(capabilities, () => pending.promise),
    );
    const done = boot.start();
    expect(entry).not.toHaveBeenCalled();
    pending.resolve();
    await done;
    expect(entry).toHaveBeenCalledTimes(1);
  });
  it("skips importing and retains the native object", async () => {
    const native = {};
    const capabilities = { Temporal: native };
    const load = vi.fn();
    await ensureTemporal(capabilities, load);
    expect(load).not.toHaveBeenCalled();
    expect(capabilities.Temporal).toBe(native);
  });
  it("loads missing Temporal before evaluating the app", async () => {
    const capabilities: { Temporal?: unknown } = {};
    const load = vi.fn(async () => {
      capabilities.Temporal = {};
    });
    const entry = vi.fn(async () => {
      expect(capabilities.Temporal).toBeDefined();
      return application();
    });
    const boot = fixture(entry, () => ensureTemporal(capabilities, load));
    await boot.start();
    expect(load).toHaveBeenCalledTimes(1);
    expect(entry).toHaveBeenCalledTimes(1);
  });
  it("starts independent loaders concurrently and waits for every one", async () => {
    const a = Promise.withResolvers<void>();
    const b = Promise.withResolvers<void>();
    const first = vi.fn(() => a.promise);
    const second = vi.fn(() => b.promise);
    const entry = vi.fn(async () => application());
    const boot = fixture(entry, () => ensurePolyfills([first, second]));
    const done = boot.start();
    expect(first).toHaveBeenCalledTimes(1);
    expect(second).toHaveBeenCalledTimes(1);
    a.resolve();
    await a.promise;
    expect(entry).not.toHaveBeenCalled();
    b.resolve();
    await done;
    expect(entry).toHaveBeenCalledTimes(1);
  });
  it("does not start on failure and retries without duplicate roots or subscriptions", async () => {
    const ensure = vi
      .fn()
      .mockRejectedValueOnce(new Error("offline"))
      .mockResolvedValue(undefined);
    const stop = vi.fn();
    const { startApp } = application(stop);
    const entry = vi.fn(async () => ({ startApp }));
    const boot = fixture(entry, ensure);
    await expect(boot.start()).rejects.toThrow("offline");
    expect(entry).not.toHaveBeenCalled();
    await Promise.all([boot.start(), boot.start()]);
    await boot.start();
    expect(startApp).toHaveBeenCalledTimes(1);
    boot.dispose();
    boot.dispose();
    expect(stop).toHaveBeenCalledTimes(1);
  });
  it("HMR disposal prevents pending initialization and a replacement owns one lifecycle", async () => {
    const pending = Promise.withResolvers<void>();
    const entry = vi.fn(async () => application());
    const boot = fixture(entry, () => pending.promise);
    const done = boot.start();
    boot.dispose();
    pending.resolve();
    await done;
    expect(entry).not.toHaveBeenCalled();
    const replacement = fixture(entry);
    await replacement.start();
    expect(entry).toHaveBeenCalledTimes(1);
    replacement.dispose();
  });
  it("retains post-barrier application ownership until controller disposal", async () => {
    const dispose = vi.fn();
    const boot = fixture(async () => application(dispose));
    await boot.start();
    expect(dispose).not.toHaveBeenCalled();
    boot.dispose();
    expect(dispose).toHaveBeenCalledTimes(1);
  });
  it("does not retry cleanup or startup after application disposal throws", async () => {
    const error = new Error("unmount failed");
    const dispose = vi.fn(() => {
      throw error;
    });
    const entry = vi.fn(async () => application(dispose));
    const boot = fixture(entry);
    await boot.start();
    expect(() => boot.dispose()).toThrow(error);
    boot.dispose();
    await boot.start();
    expect(dispose).toHaveBeenCalledTimes(1);
    expect(entry).toHaveBeenCalledTimes(1);
  });
  it("retries a capability loader that throws before returning a promise", async () => {
    const ensure = vi
      .fn()
      .mockImplementationOnce(() => {
        throw new Error("capability check failed");
      })
      .mockResolvedValue(undefined);
    const entry = vi.fn(async () => application());
    const boot = fixture(entry, ensure);
    await expect(boot.start()).rejects.toThrow("capability check failed");
    await boot.start();
    expect(ensure).toHaveBeenCalledTimes(2);
    expect(entry).toHaveBeenCalledTimes(1);
  });
  it("does not mount an entry that resolves after disposal", async () => {
    const pending = Promise.withResolvers<void>();
    const app = application();
    const entry = vi.fn(async () => {
      await pending.promise;
      return app;
    });
    const boot = fixture(entry);
    const done = boot.start();
    await vi.waitFor(() => expect(entry).toHaveBeenCalledTimes(1));
    boot.dispose();
    pending.resolve();
    await done;
    expect(app.startApp).not.toHaveBeenCalled();
  });
  it("releases a root when startup synchronously disposes the bootstrap", async () => {
    const dispose = vi.fn();
    const boot = fixture(async () => ({
      startApp() {
        boot.dispose();
        return { [Symbol.dispose]: dispose };
      },
    }));
    await boot.start();
    expect(dispose).toHaveBeenCalledTimes(1);
  });
});

describe("runtime configuration barrier", () => {
  it("loads capabilities and configuration concurrently, then starts with the result", async () => {
    const order: string[] = [];
    const started: unknown[] = [];
    const ensure = vi.fn(async () => {
      order.push("ensure:start");
      await Promise.resolve();
      order.push("ensure:end");
    });
    const loadConfig = vi.fn(async () => {
      order.push("config:start");
      await Promise.resolve();
      order.push("config:end");
      return testConfig;
    });
    const entry = vi.fn(async () => ({
      startApp: vi.fn((config: unknown) => {
        started.push(config);
        return { [Symbol.dispose]: vi.fn() };
      }),
    }));
    const boot = fixture(entry, ensure, { loadConfig });
    await boot.start();
    // Both begin before either finishes, so neither waits on the other.
    expect(order.slice(0, 2)).toEqual(["ensure:start", "config:start"]);
    expect(started).toEqual([{ config: testConfig, auth: testAuthSetup }]);
  });

  it("does not load application modules when configuration fails", async () => {
    const entry = vi.fn(async () => application());
    const boot = fixture(entry, undefined, {
      loadConfig: async () => {
        throw new Error("Invalid /konobangu.config.json");
      },
    });
    await expect(boot.start()).rejects.toThrow(
      "Invalid /konobangu.config.json",
    );
    expect(entry).not.toHaveBeenCalled();
  });

  it("gives each load a deadline that reaches the in-flight request", async () => {
    vi.useFakeTimers();
    try {
      const entry = vi.fn(async () => application());
      let observed: AbortSignal | undefined;
      const boot = fixture(entry, undefined, {
        deadlineMs: 25,
        loadConfig: (signal) =>
          new Promise((_resolve, reject) => {
            observed = signal;
            signal.addEventListener("abort", () => reject(signal.reason));
          }),
      });
      const done = expect(boot.start()).rejects.toThrow("exceeded 25ms");
      await vi.advanceTimersByTimeAsync(25);
      await done;
      expect(observed?.aborted).toBe(true);
      expect(entry).not.toHaveBeenCalled();
    } finally {
      vi.useRealTimers();
    }
  });

  it("disposal cancels an in-flight configuration request", async () => {
    const entry = vi.fn(async () => application());
    let observed: AbortSignal | undefined;
    const boot = fixture(entry, undefined, {
      loadConfig: (signal) =>
        new Promise((_resolve, reject) => {
          observed = signal;
          signal.addEventListener("abort", () => reject(signal.reason));
        }),
    });
    const done = boot.start().catch((error: unknown) => error);
    boot.dispose();
    await done;
    expect(observed?.aborted).toBe(true);
    expect(entry).not.toHaveBeenCalled();
  });
});

describe("stage deadlines do not depend on the operation observing abort", () => {
  it("aborts an unfinished sibling and clears both deadlines on failure", async () => {
    vi.useFakeTimers();
    try {
      const failure = new Error("capability load failed");
      const entry = vi.fn(async () => application());
      let observed: AbortSignal | undefined;
      const boot = fixture(
        entry,
        async () => {
          throw failure;
        },
        {
          loadConfig: (signal) => {
            observed = signal;
            return new Promise(() => {});
          },
        },
      );
      await expect(boot.start()).rejects.toBe(failure);
      expect(observed?.aborted).toBe(true);
      expect(vi.getTimerCount()).toBe(0);
      expect(entry).not.toHaveBeenCalled();
    } finally {
      vi.useRealTimers();
    }
  });

  it("leaves successful stage signals and the application alive after completion", async () => {
    const signals: AbortSignal[] = [];
    const dispose = vi.fn();
    const boot = fixture(
      async () => application(dispose),
      async (signal) => {
        signals.push(signal);
      },
      {
        loadConfig: async (signal) => {
          signals.push(signal);
          return testConfig;
        },
      },
    );
    await boot.start();
    expect(signals).toHaveLength(2);
    expect(signals.every((signal) => !signal.aborted)).toBe(true);
    expect(dispose).not.toHaveBeenCalled();
    boot.dispose();
    expect(dispose).toHaveBeenCalledTimes(1);
    // Completed stages no longer subscribe to bootstrap cancellation.
    expect(signals.every((signal) => !signal.aborted)).toBe(true);
  });

  it("shares a pending attempt and retries a timeout without accepting late results", async () => {
    vi.useFakeTimers();
    try {
      const pending = Promise.withResolvers<void>();
      const ensure = vi
        .fn()
        .mockReturnValueOnce(pending.promise)
        .mockResolvedValue(undefined);
      const entry = vi.fn(async () => application());
      const boot = fixture(entry, ensure, { deadlineMs: 25 });
      const first = boot.start();
      expect(boot.start()).toBe(first);
      const failed = expect(first).rejects.toThrow(
        "Capability loading exceeded 25ms",
      );
      await vi.advanceTimersByTimeAsync(25);
      await failed;
      await boot.start();
      pending.resolve();
      await pending.promise;
      expect(ensure).toHaveBeenCalledTimes(2);
      expect(entry).toHaveBeenCalledTimes(1);
      expect(vi.getTimerCount()).toBe(0);
    } finally {
      vi.useRealTimers();
    }
  });

  it("fails the stage on time even when the loader ignores its signal", async () => {
    vi.useFakeTimers();
    try {
      const entry = vi.fn(async () => application());
      // The default capability loader takes no signal, so an ignored abort must
      // still reject this stage rather than hang until the page is reloaded.
      const ensure = vi.fn(() => new Promise<void>(() => {}));
      const boot = fixture(entry, ensure, { deadlineMs: 40 });
      const done = expect(boot.start()).rejects.toThrow(
        "Capability loading exceeded 40ms",
      );
      await vi.advanceTimersByTimeAsync(40);
      await done;
      expect(entry).not.toHaveBeenCalled();
    } finally {
      vi.useRealTimers();
    }
  });

  it("keeps retrying cleanly and treats disposal as a quiet cancellation", async () => {
    const entry = vi.fn(async () => application());
    const boot = fixture(entry, undefined, {
      loadConfig: async () => {
        throw new Error("offline");
      },
    });
    // Repeated retries must not accumulate parent abort listeners, and each
    // attempt must still surface the real loader failure.
    for (let attempt = 0; attempt < 5; attempt += 1)
      await expect(boot.start()).rejects.toThrow("offline");
    expect(entry).not.toHaveBeenCalled();
    boot.dispose();
    await expect(boot.start()).resolves.toBeUndefined();
  });

  it("cleans up when a loader throws synchronously, before any race exists", async () => {
    vi.useFakeTimers();
    try {
      const entry = vi.fn(async () => application());
      const boot = fixture(entry, () => {
        throw new Error("capability check failed");
      });
      await expect(boot.start()).rejects.toThrow("capability check failed");
      // A synchronous throw used to escape before the race existed, leaving the
      // deadline timer armed and its parent abort listener attached.
      expect(vi.getTimerCount()).toBe(0);
      boot.dispose();
    } finally {
      vi.useRealTimers();
    }
  });

  it("settles quietly when disposal cancels a pending stage", async () => {
    const entry = vi.fn(async () => application());
    const boot = fixture(entry, () => new Promise<void>(() => {}));
    const done = boot.start();
    boot.dispose();
    // HMR and pagehide must not reach the startup error page.
    await expect(done).resolves.toBeUndefined();
    expect(entry).not.toHaveBeenCalled();
  });
});
