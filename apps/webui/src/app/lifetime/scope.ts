import { SecuritydeptInjectionToken } from "@securitydept/client";

/** Resource registration belongs exclusively to provider factories. */
export class AppLifetime implements Disposable {
  private readonly resources = new DisposableStack();
  private readonly cancellation = new AbortController();
  private readonly acquired = new WeakSet<object>();
  readonly signal = this.cancellation.signal;

  own<T extends object>(value: T, release: (value: T) => void): T {
    if (this.signal.aborted)
      throw new Error("Cannot create a service after the application ended");
    if (this.acquired.has(value))
      throw new Error("A resource already has an application owner");
    this.acquired.add(value);
    this.resources.adopt(value, release);
    return value;
  }

  [Symbol.dispose](): void {
    if (this.signal.aborted)
      throw new Error("Application ownership was already consumed");
    this.cancellation.abort(
      new DOMException("Application ended", "AbortError"),
    );
    this.resources.dispose();
  }
}

/** Internal factory capability; business services expose borrowed interfaces. */
export const APP_LIFETIME = new SecuritydeptInjectionToken<AppLifetime>(
  "APP_LIFETIME",
);
