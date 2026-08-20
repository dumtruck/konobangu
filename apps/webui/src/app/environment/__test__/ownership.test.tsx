// @vitest-environment jsdom
import {
  inject,
  SecuritydeptDestroyRef,
  SecuritydeptInjectionToken,
  TIME_TRAIT_TOKEN,
} from "@securitydept/client";
import * as TanStackRouter from "@tanstack/react-router";
import { Subject, type Subscription } from "rxjs";
import { describe, expect, it, vi } from "vitest";
import { createKonobanguFixture, response } from "@/__test__/support/fixture";
import { stubMatchMedia } from "@/__test__/support/match-media";
import {
  createKonobanguEnvironmentForTest,
  createOidcAuthSetupForTest,
  type KonobanguTestEnvironment,
} from "@/app/environment/test";
import type { AppLifetime } from "@/app/lifetime/scope";
import { APP_LIFETIME } from "@/app/lifetime/scope";
import { AuthService } from "@/domains/auth/auth.service";
import { GraphQLService } from "@/infra/graphql/graphql.service";
import { ThemeService } from "@/infra/styles/theme.service";

// Native ESM exports are immutable; spy mode preserves their actual behavior.
vi.mock("@tanstack/react-router", { spy: true });

const fixture = createKonobanguFixture;
describe("application resource ownership", () => {
  it("rolls back acquired providers and the actual SDK owner when environment construction fails", () => {
    const first = new SecuritydeptInjectionToken<Subscription>(
      "First owned provider",
    );
    const second = new SecuritydeptInjectionToken<Subscription>(
      "Second owned provider",
    );
    const unused = new SecuritydeptInjectionToken<object>(
      "Unrequested lazy provider",
    );
    const events = new Subject<number>();
    const observedFirst = vi.fn();
    const observedSecond = vi.fn();
    const releaseFirst = vi.fn();
    const releaseSecond = vi.fn();
    const sdkDestroyed = vi.fn();
    const constructUnused = vi.fn(() => ({}));
    let sdkOwner: SecuritydeptDestroyRef | undefined;
    let rootSignal: AbortSignal | undefined;
    const failure = new Error("Injected environment initialization failure");
    let published: KonobanguTestEnvironment | undefined;
    try {
      expect(() => {
        published = createKonobanguEnvironmentForTest({
          providers: [
            {
              provide: first,
              useFactory: (
                lifetime: AppLifetime,
                destroy: SecuritydeptDestroyRef,
              ) => {
                sdkOwner = destroy;
                rootSignal = lifetime.signal;
                destroy.onDestroy(sdkDestroyed);
                return lifetime.own(
                  events.subscribe(observedFirst),
                  (subscription) => {
                    releaseFirst();
                    subscription.unsubscribe();
                  },
                );
              },
              deps: [APP_LIFETIME, SecuritydeptDestroyRef],
            },
            {
              provide: second,
              useFactory: (lifetime: AppLifetime) =>
                lifetime.own(
                  events.subscribe(observedSecond),
                  (subscription) => {
                    releaseSecond();
                    subscription.unsubscribe();
                  },
                ),
              deps: [APP_LIFETIME],
            },
            { provide: unused, useFactory: constructUnused },
            {
              provide: TIME_TRAIT_TOKEN,
              useFactory: (_first: Subscription, _second: Subscription) => {
                events.next(1);
                throw failure;
              },
              deps: [first, second],
            },
          ],
        });
      }).toThrow(failure.message);
      expect(observedFirst).toHaveBeenCalledExactlyOnceWith(1);
      expect(observedSecond).toHaveBeenCalledExactlyOnceWith(1);
      events.next(2);
      expect(observedFirst).toHaveBeenCalledTimes(1);
      expect(observedSecond).toHaveBeenCalledTimes(1);
      expect(releaseFirst).toHaveBeenCalledTimes(1);
      expect(releaseSecond).toHaveBeenCalledTimes(1);
      expect(rootSignal?.aborted).toBe(true);
      expect(sdkOwner?.destroyed).toBe(true);
      expect(sdkDestroyed).toHaveBeenCalledTimes(1);
      expect(constructUnused).not.toHaveBeenCalled();
      expect(published).toBeUndefined();
    } finally {
      events.complete();
    }
  });
  it("rolls back actual entry initialization without leaving a published injector or mounted root", async () => {
    const media = stubMatchMedia();
    vi.stubGlobal("fetch", async () => response(401));
    const mount = document.createElement("div");
    mount.id = "app";
    document.body.append(mount);
    const releases = vi.fn();
    const sdkDestroyed = vi.fn();
    let sdkOwner: SecuritydeptDestroyRef | undefined;
    let rootSignal: AbortSignal | undefined;
    let acquiredTheme: ThemeService | undefined;
    const createTheme = ThemeService.createOwned;
    vi.spyOn(ThemeService, "createOwned").mockImplementation((lifetime) => {
      rootSignal = lifetime.signal;
      sdkOwner = inject(SecuritydeptDestroyRef);
      sdkOwner.onDestroy(sdkDestroyed);
      lifetime.own({}, releases);
      acquiredTheme = createTheme(lifetime);
      return acquiredTheme;
    });
    const failAuth = new Error("Injected auth provider initialization failure");
    vi.spyOn(AuthService, "createOwned").mockImplementation(() => {
      throw failAuth;
    });
    const createApollo = vi.spyOn(GraphQLService, "createOwned");
    const createRouter = vi.mocked(TanStackRouter.createRouter);
    createRouter.mockClear();
    try {
      const { startApp } = await import("@/app/entry");
      expect(() =>
        startApp({
          config: { version: 1, apiBaseUrl: "/api/" },
          auth: createOidcAuthSetupForTest(window.location.origin),
        }),
      ).toThrow(failAuth.message);
      expect(acquiredTheme).toBeDefined();
      expect(rootSignal?.aborted).toBe(true);
      expect(releases).toHaveBeenCalledTimes(1);
      expect(sdkOwner?.destroyed).toBe(true);
      expect(sdkDestroyed).toHaveBeenCalledTimes(1);
      media.dark.change(true);
      expect(acquiredTheme?.systemTheme.get()).toBe("light");
      expect(createApollo).not.toHaveBeenCalled();
      const routerCreation = createRouter.mock.results.at(-1);
      expect(routerCreation?.type).toBe("return");
      if (routerCreation?.type !== "return")
        throw new Error("The actual entry router was not constructed");
      expect(routerCreation.value.options.context).toEqual({});
      expect(mount.childNodes.length).toBe(0);
    } finally {
      mount.remove();
    }
  });
  it("automatically owns a lazily acquired Apollo service", () => {
    const { environment } = fixture();
    const graphql = environment.injector.get(GraphQLService);
    const stop = vi.spyOn(graphql._apollo, "stop");
    environment[Symbol.dispose]();
    expect(stop).toHaveBeenCalledTimes(1);
    expect("dispose" in graphql).toBe(false);
    expect(Symbol.dispose in graphql).toBe(false);
  });
  it("does not construct unrequested providers while releasing", () => {
    const { environment } = fixture();
    const theme = vi.spyOn(ThemeService, "createOwned");
    environment[Symbol.dispose]();
    expect(theme).not.toHaveBeenCalled();
  });
  it("rejects duplicate ownership registration", () => {
    const { environment } = fixture();
    const lifetime = environment.injector.get(APP_LIFETIME);
    const resource = {};
    lifetime.own(resource, () => undefined);
    expect(() => lifetime.own(resource, () => undefined)).toThrow(
      "already has an application owner",
    );
  });
  it("unwinds in reverse acquisition order and continues after errors", () => {
    const { environment } = fixture();
    const lifetime = environment.injector.get(APP_LIFETIME);
    const released: number[] = [];
    lifetime.own({ id: 1 }, (value) => released.push(value.id));
    lifetime.own({ id: 2 }, (value) => {
      released.push(value.id);
      throw new Error("cleanup failed");
    });
    lifetime.own({ id: 3 }, (value) => released.push(value.id));
    expect(() => environment[Symbol.dispose]()).toThrow("cleanup failed");
    expect(released).toEqual([3, 2, 1]);
    expect(() => lifetime.own({ id: 4 }, () => undefined)).toThrow(
      "after the application ended",
    );
  });
});
