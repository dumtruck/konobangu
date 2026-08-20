import { createSignal, readonlySignal } from "@securitydept/client";
import type { Observable } from "rxjs";

type Subscription = { unsubscribe(): void };

export type ResourceBindingState =
  /** Not connected, or the bounded address is being derived or renewed. */
  | { readonly status: "pending" }
  | { readonly status: "ready"; readonly url: string }
  | { readonly status: "failed"; readonly error: unknown };

const PENDING: ResourceBindingState = { status: "pending" };

/**
 * The address a view may currently use for one resource locator. It is a
 * value of the resource domain and knows nothing about rendering: a view
 * connects while it shows the address and disconnects when it stops.
 *
 * Connections are counted and the binding can be connected again after its
 * last disconnect, so a renderer that mounts, unmounts and remounts the same
 * view never ends it. Each view asks the service for its own binding; they
 * need no shared identity because grants are already shared by the service.
 */
export class ResourceBinding {
  readonly input: string;
  private readonly options: {
    /** A public locator is its own address and never needs a grant. */
    private: boolean;
    observe(input: string): Observable<{ readonly url: string } | undefined>;
  };
  private readonly current;
  readonly state;
  private connections = 0;
  private subscription: Subscription | undefined;

  constructor(options: { input: string } & ResourceBinding["options"]) {
    this.input = options.input;
    this.options = options;
    this.current = createSignal<ResourceBindingState>(
      options.private ? PENDING : { status: "ready", url: options.input },
    );
    this.state = readonlySignal(this.current);
  }

  connect(): void {
    this.connections++;
    if (this.connections === 1 && this.options.private) this.start();
  }

  disconnect(): void {
    if (this.connections === 0) return;
    this.connections--;
    if (this.connections > 0 || !this.options.private) return;
    this.stop();
    // A bounded address must not outlive the view that was using it.
    this.current.set(PENDING);
  }

  /** Derives the address again after a failure, if anything is connected. */
  retry(): void {
    if (this.connections > 0 && this.options.private) this.start();
  }

  private stop(): void {
    this.subscription?.unsubscribe();
    this.subscription = undefined;
  }

  private start(): void {
    this.stop();
    this.current.set(PENDING);
    this.subscription = this.options.observe(this.input).subscribe({
      next: (resource) =>
        this.current.set(
          resource ? { status: "ready", url: resource.url } : PENDING,
        ),
      error: (error: unknown) => this.current.set({ status: "failed", error }),
    });
  }
}

export type ImageBindingState = {
  /**
   * A native image element cannot be reused for another attempt or identity:
   * it would replay its previous request. A new generation needs a new element.
   */
  readonly generation: number;
} & (
  | { readonly status: "pending" }
  | { readonly status: "ready"; readonly url: string }
  | { readonly status: "failed" }
);

/**
 * The address one image may load. Unlike a link it is derived once and kept:
 * a loaded image never changes address because a grant was renewed. It is
 * derived again for a new identity, once automatically when the element could
 * not load a fresh address, and on an explicit retry.
 */
export class ImageBinding {
  readonly input: string;
  readonly private: boolean;
  private readonly options: {
    acquire(
      input: string,
      renewal?: "renew",
    ): Promise<{ readonly url: string }>;
    /** Keeps grant renewal running while an image is shown. */
    activate(): Disposable;
    identityChanges: { subscribe(next: () => void): Subscription };
  };
  private readonly current;
  readonly state;
  private connections = 0;
  private generation = 0;
  // Counted per identity: a forced renewal follows the first failed load.
  private renewals = 0;
  private elementLoaded = false;
  private derivation: AbortController | undefined;
  private lease: Disposable | undefined;
  private identity: Subscription | undefined;

  constructor(
    options: { input: string; private: boolean } & ImageBinding["options"],
  ) {
    this.input = options.input;
    this.private = options.private;
    this.options = options;
    this.current = createSignal<ImageBindingState>(this.settled());
    this.state = readonlySignal(this.current);
  }

  connect(): void {
    this.connections++;
    if (this.connections > 1) return;
    this.identity = this.options.identityChanges.subscribe(() => {
      this.renewals = 0;
      this.renew();
    });
    if (this.private) this.lease = this.options.activate();
    this.derive();
  }

  disconnect(): void {
    if (this.connections === 0) return;
    this.connections--;
    if (this.connections > 0) return;
    this.identity?.unsubscribe();
    this.identity = undefined;
    this.derivation?.abort();
    this.lease?.[Symbol.dispose]();
    this.lease = undefined;
    // A bounded address must not outlive the view that was using it.
    this.elementLoaded = false;
    this.current.set(this.settled());
  }

  /** The element finished loading the current address. */
  loaded(): void {
    this.elementLoaded = true;
  }

  /**
   * The element could not load the current address. Returns whether a
   * recovery started; otherwise the binding is now failed.
   */
  loadFailed(): boolean {
    if (
      this.private &&
      !this.elementLoaded &&
      this.renewals === 0 &&
      this.current.get().status === "ready"
    ) {
      this.renewals++;
      this.renew();
      return true;
    }
    this.current.set({ status: "failed", generation: this.generation });
    return false;
  }

  retry(): void {
    this.renewals++;
    this.renew();
  }

  private settled(): ImageBindingState {
    return this.private
      ? { status: "pending", generation: this.generation }
      : { status: "ready", url: this.input, generation: this.generation };
  }

  private renew(): void {
    this.generation++;
    this.derive();
  }

  private derive(): void {
    this.derivation?.abort();
    this.elementLoaded = false;
    this.current.set(this.settled());
    if (!this.private || this.connections === 0) return;
    const current = new AbortController();
    this.derivation = current;
    const generation = this.generation;
    this.options.acquire(this.input, this.renewals ? "renew" : undefined).then(
      (resource) => {
        if (!current.signal.aborted)
          this.current.set({ status: "ready", url: resource.url, generation });
      },
      () => {
        if (!current.signal.aborted)
          this.current.set({ status: "failed", generation });
      },
    );
  }
}
