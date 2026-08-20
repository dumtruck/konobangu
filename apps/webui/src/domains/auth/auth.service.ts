import {
  abortSignalToCancellationToken,
  createComputed,
  createSignal,
  ENVIRONMENT_TOKEN,
  inject,
  ResourceStatus,
  readonlySignal,
} from "@securitydept/client";
import {
  type BaseOidcModeClient,
  TokenSetAuthEventType,
  type TokenSetAuthSnapshot,
} from "@securitydept/token-set-context-client/orchestration";
import {
  type TokenSetClientRegistry,
  TokenSetClientRegistryEventType,
} from "@securitydept/token-set-context-client/registry";
import { defer, distinctUntilChanged, from, map, skip, startWith } from "rxjs";
import {
  AUTH_SETUP,
  OIDC_CLIENT_ENTRY,
  tokenSetRegistryFromInjector,
} from "@/app/auth/tokens";
import { APP_LIFETIME, type AppLifetime } from "@/app/lifetime/scope";
import { withinScope } from "@/infra/http/cancellation";
import {
  type AuthorizationPolicy,
  HTTP_CLIENT,
  type RequestAuthorization,
} from "@/infra/http/http-client";
import {
  type ApplicationPrincipal,
  identityKeyOf,
  parseApplicationPrincipal,
} from "./principal";
import { SignInFlow } from "./sign-in-flow";

export type AuthCheckState =
  | { status: "idle" | "checking" }
  | { status: "success"; isAuthenticated: boolean }
  | { status: "error"; message: string; error: unknown };

/** Application identity coordinator. SDK clients remain the sole auth owners. */
export class AuthService implements AuthorizationPolicy {
  private readonly injector = inject(ENVIRONMENT_TOKEN).injector;
  private readonly authSetup = inject(AUTH_SETUP);
  private readonly lifetime = inject(APP_LIFETIME);
  readonly lifetimeSignal = this.lifetime.signal;
  private readonly epochState = createSignal(0);
  readonly identityChanges = from(this.epochState.watchStream());
  private identityController = new AbortController();
  get identitySignal(): AbortSignal {
    return this.identityController.signal;
  }
  private pending: Promise<ApplicationPrincipal | null> | undefined;
  private pendingEpoch = -1;
  private pendingMaterialGeneration = -1;
  private renewal: Promise<void> | undefined;
  private logoutPending: Promise<void> | undefined;
  // These are borrowed SDK snapshot references, not another token store.
  private material: TokenSetAuthSnapshot | null = null;
  private confirmedMaterial: TokenSetAuthSnapshot | null = null;
  private materialGeneration = 0;
  private materialController = new AbortController();
  private confirmationScheduled = false;
  private readonly principalState = createSignal<ApplicationPrincipal | null>(
    null,
  );
  private readonly checkState = createSignal<AuthCheckState>({
    status: "idle",
  });
  readonly userData = readonlySignal(this.principalState);
  readonly check = readonlySignal(this.checkState);
  readonly isAuthenticated = createComputed(
    () => this.principalState.get() !== null,
  );
  readonly identityKey = createComputed(() => {
    const principal = this.principalState.get();
    return principal && this.isAuthenticated.get()
      ? identityKeyOf(principal)
      : null;
  });

  /**
   * Emits only when the verified identity itself changes. A token renewal or
   * a profile re-read of the same principal recomputes the key without
   * changing it, and must not make a dependent domain discard its state.
   */
  readonly identityKeyChanges = defer(() =>
    from(this.identityKey.watchStream()).pipe(
      map(() => this.identityKey.get()),
      // The baseline is the identity at subscription, not at construction.
      startWith(this.identityKey.get()),
      distinctUntilChanged(),
      skip(1),
    ),
  );

  static createOwned(lifetime: AppLifetime): AuthService {
    lifetime.signal.throwIfAborted();
    using rollback = new DisposableStack();
    const service = new AuthService();
    rollback.defer(() => service.end());
    const { clientKey } = service.authSetup;
    const registry = service.registry();
    const errors = registry.errors.subscribe({
      next: (error) => {
        if (!service.lifetimeSignal.aborted)
          service.checkState.set({
            status: "error",
            message: error.message,
            error,
          });
      },
    });
    rollback.defer(() => errors.unsubscribe());
    const events = registry.authEvents.subscribe({
      next: (event) => {
        if (service.lifetimeSignal.aborted || service.logoutPending) return;
        if (
          event.type === TokenSetAuthEventType.AuthMaterialRestored ||
          event.type === TokenSetAuthEventType.AuthRefreshSucceeded ||
          event.type === TokenSetAuthEventType.AuthAuthenticated
        ) {
          const view = registry.clientRecordOptionFor(clientKey)?.get();
          if (view?.status === ResourceStatus.Resolved)
            service.observeMaterial(
              view.client,
              event.type === TokenSetAuthEventType.AuthRefreshSucceeded
                ? "refresh"
                : "restore",
            );
        }
        if (
          (event.type === TokenSetAuthEventType.AuthMaterialCleared ||
            event.type === TokenSetAuthEventType.AuthUnauthenticated) &&
          (service.material || service.principalState.get())
        ) {
          service.endMaterial();
          service.invalidate();
          service.principalState.set(null);
          service.checkState.set({
            status: "success",
            isAuthenticated: false,
          });
        }
      },
    });
    rollback.defer(() => events.unsubscribe());
    // Factory.start can authenticate before the registry forwards non-replay
    // auth events. The ready snapshot closes that gap without eager startup.
    const ready = registry.events.subscribe({
      next: (event) => {
        if (service.lifetimeSignal.aborted || service.logoutPending) return;
        if (event.type !== TokenSetClientRegistryEventType.Ready) return;
        service.observeMaterial(event.client, "restore");
      },
    });
    rollback.defer(() => ready.unsubscribe());
    const owner = rollback.move();
    return lifetime.own(service, () => owner.dispose());
  }
  private schedulePrincipalConfirmation(): void {
    if (this.confirmationScheduled) return;
    this.confirmationScheduled = true;
    // Multiple auth/ready signals for one SDK commit share its confirmation.
    queueMicrotask(() => {
      this.confirmationScheduled = false;
      if (this.lifetimeSignal.aborted || this.logoutPending) return;
      void this.refresh().catch(() => undefined);
    });
  }
  private observeMaterial(
    client: BaseOidcModeClient,
    transition: "restore" | "refresh",
  ): void {
    const snapshot = client.authSnapshot.get();
    const material = "value" in snapshot ? snapshot.value : null;
    if (!material || material === this.material) return;
    this.materialController.abort(
      new DOMException("Authentication material superseded", "AbortError"),
    );
    this.materialController = new AbortController();
    this.materialGeneration++;
    this.material = material;
    this.confirmedMaterial = null;
    if (transition === "restore") {
      this.invalidate();
      this.principalState.set(null);
    }
    this.checkState.set({ status: "checking" });
    this.schedulePrincipalConfirmation();
  }
  private endMaterial(): void {
    this.materialController.abort(
      new DOMException("Authentication material ended", "AbortError"),
    );
    this.materialController = new AbortController();
    this.materialGeneration++;
    this.material = null;
    this.confirmedMaterial = null;
  }
  private assertCanAcquire(): void {
    this.lifetimeSignal.throwIfAborted();
    if (this.logoutPending)
      throw new DOMException(
        "Local authentication access is closing",
        "AbortError",
      );
  }
  epoch(): number {
    return this.epochState.get();
  }
  private invalidate(): void {
    this.identityController.abort(
      new DOMException("Identity ended", "AbortError"),
    );
    this.identityController = new AbortController();
    this.epochState.set(this.epoch() + 1);
  }
  private end(): void {
    this.identityController.abort(this.lifetimeSignal.reason);
    this.materialController.abort(this.lifetimeSignal.reason);
  }

  async authorize(
    _url: URL,
    signal: AbortSignal,
    policy: "principal-probe" | "protected",
  ): Promise<RequestAuthorization> {
    signal.throwIfAborted();
    this.assertCanAcquire();
    const record = await withinScope(
      this.registry().clientRecordFor(this.authSetup.clientKey, {
        initialize: true,
      }),
      signal,
    );
    signal.throwIfAborted();
    for (;;) {
      if (policy === "protected")
        await this.confirmCurrentMaterial(record.client, signal);
      const material = this.material;
      const header = await withinScope(
        record.client.authorizationHeaderValue.whenValue(),
        signal,
      );
      signal.throwIfAborted();
      if (
        policy === "protected" &&
        (material !== this.material || material !== this.confirmedMaterial)
      )
        continue;
      return {
        header: header ?? undefined,
        assertCurrent: () => {
          signal.throwIfAborted();
          this.assertCanAcquire();
          if (
            material !== this.material ||
            (policy === "protected" && material !== this.confirmedMaterial)
          )
            throw new DOMException(
              "Authorization material changed before request admission",
              "AbortError",
            );
        },
      };
    }
  }
  private async confirmCurrentMaterial(
    client: BaseOidcModeClient,
    signal: AbortSignal,
  ): Promise<void> {
    for (;;) {
      signal.throwIfAborted();
      this.assertCanAcquire();
      this.observeMaterial(client, "restore");
      const material = this.material;
      if (!material)
        throw new Error(
          "Protected OIDC requests require authentication material",
        );
      if (material === this.confirmedMaterial && this.principalState.get())
        return;
      const generation = this.materialGeneration;
      try {
        await withinScope(this.refresh(), signal);
      } catch (error) {
        if (!signal.aborted && generation !== this.materialGeneration) continue;
        throw error;
      }
      if (material !== this.material) continue;
      if (material !== this.confirmedMaterial || !this.principalState.get())
        throw new Error(
          "Authentication material has no verified application principal",
        );
      return;
    }
  }
  refresh(): Promise<ApplicationPrincipal | null> {
    try {
      this.assertCanAcquire();
    } catch (error) {
      return Promise.reject(error);
    }
    if (
      this.pending &&
      this.pendingEpoch === this.epoch() &&
      this.pendingMaterialGeneration === this.materialGeneration
    )
      return this.pending;
    const epoch = this.epoch();
    const generation = this.materialGeneration;
    this.pendingEpoch = epoch;
    this.pendingMaterialGeneration = generation;
    this.checkState.set({ status: "checking" });
    const operation = this.readPrincipal(this.materialController.signal)
      .then((principal) => {
        if (
          this.lifetimeSignal.aborted ||
          epoch !== this.epoch() ||
          generation !== this.materialGeneration ||
          this.logoutPending
        )
          throw new DOMException(
            "Application identity acquisition was cancelled",
            "AbortError",
          );
        const before = this.principalState.get();
        if (
          (before && identityKeyOf(before)) !==
          (principal && identityKeyOf(principal))
        )
          this.invalidate();
        this.principalState.set(principal);
        this.confirmedMaterial = principal ? this.material : null;
        this.checkState.set({
          status: "success",
          isAuthenticated: this.isAuthenticated.get(),
        });
        return principal;
      })
      .catch((error: unknown) => {
        if (
          !this.lifetimeSignal.aborted &&
          epoch === this.epoch() &&
          generation === this.materialGeneration
        ) {
          if (this.principalState.get()) this.invalidate();
          this.principalState.set(null);
          this.checkState.set({
            status: "error",
            error,
            message:
              error instanceof Error
                ? error.message
                : "Authentication check failed",
          });
        }
        throw error;
      })
      .finally(() => {
        if (this.pending === operation) this.pending = undefined;
      });
    this.pending = operation;
    return operation;
  }
  private async readPrincipal(
    signal: AbortSignal,
  ): Promise<ApplicationPrincipal | null> {
    const client = this.injector.get(HTTP_CLIENT);
    const response = await client.response({
      endpoint: "auth/user-info",
      policy: "principal-probe",
      signal,
    });
    if (response.status === 401) {
      await response.body?.cancel();
      return null;
    }
    return parseApplicationPrincipal(await response.json());
  }
  async renew(signal: AbortSignal): Promise<void> {
    this.assertCanAcquire();
    if (!this.renewal) {
      const epoch = this.epoch();
      const ownedSignal = AbortSignal.any([
        this.lifetimeSignal,
        this.identitySignal,
      ]);
      const operation = withinScope(
        this.registry().clientRecordFor(this.authSetup.clientKey, {
          initialize: true,
        }),
        ownedSignal,
      )
        .then(async (record) => {
          ownedSignal.throwIfAborted();
          await withinScope(
            record.client.refreshState({
              cancellationToken: abortSignalToCancellationToken(ownedSignal),
            }),
            ownedSignal,
          );
          await withinScope(
            this.confirmCurrentMaterial(record.client, ownedSignal),
            ownedSignal,
          );
          if (epoch !== this.epoch())
            throw new Error("Identity changed during renewal");
        })
        .finally(() => {
          if (this.renewal === operation) this.renewal = undefined;
        });
      this.renewal = operation;
    }
    await withinScope(this.renewal, signal);
  }
  async login(returnPath = "/"): Promise<void> {
    this.assertCanAcquire();
    const record = await this.registry().clientRecordFor(
      this.authSetup.clientKey,
      { initialize: true },
    );
    this.lifetimeSignal.throwIfAborted();
    await record.client.loginWithRedirect({ postAuthRedirectUri: returnPath });
  }
  /** The sign-in a public page starts; see {@link SignInFlow}. */
  readonly signIn = new SignInFlow({
    lifetimeSignal: this.lifetimeSignal,
    login: (returnPath) => this.login(returnPath),
  });
  logout(): Promise<void> {
    this.lifetimeSignal.throwIfAborted();
    if (this.logoutPending) return this.logoutPending;
    // Publish the acquisition barrier before identity observers can start work.
    const operation = Promise.resolve()
      .then(() => this.clearLocalState())
      .finally(() => {
        if (this.logoutPending === operation) this.logoutPending = undefined;
      });
    this.logoutPending = operation;
    this.endMaterial();
    this.invalidate();
    this.principalState.set(null);
    this.checkState.set({ status: "success", isAuthenticated: false });
    return operation;
  }
  private async clearLocalState(): Promise<void> {
    await Promise.allSettled([this.pending, this.renewal]);
    if (this.lifetimeSignal.aborted) return;
    const registry = this.registry();
    const view = registry
      .clientRecordOptionFor(this.authSetup.clientKey)
      ?.get();
    try {
      if (view?.status === ResourceStatus.Resolved)
        await view.client.clearState();
    } finally {
      if (!this.lifetimeSignal.aborted) {
        registry.unregister(this.authSetup.clientKey);
        registry.register(this.injector.get(OIDC_CLIENT_ENTRY)());
      }
    }
  }
  private registry(): TokenSetClientRegistry {
    return tokenSetRegistryFromInjector(this.injector);
  }
}
export type { ApplicationPrincipal };
