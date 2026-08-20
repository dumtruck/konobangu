import {
  defer,
  finalize,
  firstValueFrom,
  forkJoin,
  fromEvent,
  type Observable,
  type SchedulerLike,
  takeUntil,
  tap,
  throwError,
  timeout,
} from "rxjs";
import type { AuthSetup } from "./auth/setup";
import {
  loadPublicAuthConfig,
  type PublicAuthConfig,
} from "./config/auth-config";
import { loadRuntimeConfig, type RuntimeConfig } from "./config/runtime";
import type { KonobanguBootstrapEnvironment } from "./environment/bootstrap";

export const BOOTSTRAP_DEADLINE_MS = 10_000;

/** What the composition root needs to exist at all. */
export type BootstrapResult = {
  readonly config: RuntimeConfig;
  readonly auth: AuthSetup;
};

/**
 * The stages this controller runs. Each is required: a bootstrap that could
 * substitute its own loader would not be the one the application ships with.
 * Tests provide the same complete object, not a partial options bag.
 */
export type BootstrapStages = {
  readonly loadConfig: (signal: AbortSignal) => Promise<RuntimeConfig>;
  readonly loadAuthConfig: (
    config: RuntimeConfig,
    signal: AbortSignal,
  ) => Promise<PublicAuthConfig>;
  readonly prepareAuth: (config: PublicAuthConfig) => Promise<AuthSetup>;
};

export type BootstrapOptions = {
  /** Only a deadline scheduler is substitutable, for deterministic timing. */
  readonly scheduler?: SchedulerLike;
};

/**
 * Builds the stages from the bootstrap environment. The SDK parser is imported
 * only once the polyfills resolved, because its resource-management
 * compat fixes the dispose symbol at module load; until then a failure stays a
 * plain Error for the standalone startup page.
 */
export function createBootstrapStages(
  environment: KonobanguBootstrapEnvironment,
): BootstrapStages {
  return {
    loadConfig: (signal) =>
      loadRuntimeConfig({
        origin: environment.documentOrigin,
        allowInsecureLoopback: environment.allowInsecureLoopback,
        client: environment.configClient,
        signal,
      }),
    loadAuthConfig: (config, signal) =>
      loadPublicAuthConfig({
        apiBaseUrl: new URL(config.apiBaseUrl, environment.documentOrigin),
        client: environment.configClient,
        signal,
      }),
    prepareAuth: async (auth) => {
      const { prepareAuthSetup } = await import("./auth/projection");
      return prepareAuthSetup(auth, {
        documentOrigin: environment.documentOrigin,
      });
    },
  };
}

function withDeadline<T>(
  operation: (signal: AbortSignal) => Promise<T>,
  parent: AbortSignal,
  deadlineMs: number,
  label: string,
  scheduler: SchedulerLike | undefined,
): Observable<T> {
  return defer(() => {
    if (parent.aborted) return throwError(() => parent.reason);
    const controller = new AbortController();
    let finished = false;
    return defer(() => operation(controller.signal)).pipe(
      timeout({
        first: deadlineMs,
        with: () =>
          throwError(() => new Error(`${label} exceeded ${deadlineMs}ms`)),
        ...(scheduler ? { scheduler } : {}),
      }),
      tap({
        next: () => {
          finished = true;
        },
        error: (reason: unknown) => controller.abort(reason),
      }),
      // A failed barrier unsubscribes its siblings; abort unfinished requests
      // while leaving successful stages and application ownership untouched.
      finalize(() => {
        if (!finished) controller.abort(parent.reason);
      }),
    );
  });
}

/**
 * Capability loading and the deployment read are independent, so they run
 * concurrently; the application modules load only after both succeed. This
 * controller exists before the resource-management polyfill, so it is not
 * itself Disposable and must not touch a resource stack.
 */
export function createBootstrap(
  loadApp: () => Promise<{
    startApp: (result: BootstrapResult) => Disposable;
  }>,
  environment: KonobanguBootstrapEnvironment,
  stages: BootstrapStages,
  options: BootstrapOptions = {},
) {
  let pending: Promise<void> | undefined;
  let lifetime: DisposableStack | undefined;
  let disposed = false;
  const cancellation = new AbortController();
  // The sidecar names the API, so the public configuration can only be read
  // after it resolves; both reads share one deadline.
  const loadDeployment = async (signal: AbortSignal) => {
    const config = await stages.loadConfig(signal);
    return { config, auth: await stages.loadAuthConfig(config, signal) };
  };
  const prerequisites$ = forkJoin({
    capabilities: withDeadline(
      environment.ensurePolyfills,
      cancellation.signal,
      environment.deadlineMs,
      "Capability loading",
      options.scheduler,
    ),
    deployment: withDeadline(
      loadDeployment,
      cancellation.signal,
      environment.deadlineMs,
      "Runtime configuration",
      options.scheduler,
    ),
  }).pipe(takeUntil(fromEvent(cancellation.signal, "abort")));
  const dispose = () => {
    if (disposed) return;
    disposed = true;
    cancellation.abort(new Error("Bootstrap disposed"));
    const owner = lifetime;
    lifetime = undefined;
    owner?.[Symbol.dispose]();
  };
  return {
    start() {
      if (disposed || lifetime) return Promise.resolve();
      if (pending) return pending;
      pending = (async () => {
        // Only cancellation completes this Promise-backed barrier without a
        // value; disposal settles quietly while loader errors still reject.
        const loaded = await firstValueFrom(prerequisites$, {
          defaultValue: undefined,
        });
        if (!loaded || disposed) return;
        const auth = await stages.prepareAuth(loaded.deployment.auth);
        if (disposed) return;
        const entry = await loadApp();
        if (disposed) return;
        using ownership = new DisposableStack();
        ownership.use(
          entry.startApp({ config: loaded.deployment.config, auth }),
        );
        // A startup callback can synchronously invalidate this bootstrap.
        if (!disposed) lifetime = ownership.move();
      })().finally(() => {
        // Settle after assignment, even when a capability loader throws synchronously.
        pending = undefined;
      });
      return pending;
    },
    dispose,
  };
}
