import type { SecuritydeptProvider } from "@securitydept/client";
import {
  createEnvironmentForNativeWeb,
  createStorageForNativeWeb,
} from "@securitydept/client/web";
import { createEnvironmentForReact } from "@securitydept/client-react";
import type { AnyRouter } from "@tanstack/react-router";
import type { AuthSetup } from "@/app/auth/setup";
import type { RuntimeConfig } from "@/app/config/runtime";
import { createResourceCrypto } from "./crypto";
import {
  createKonobanguEnvironment,
  type KonobanguEnvironment,
} from "./konobangu";
import {
  createUiStorageForMemory,
  type MediaQueryTrait,
  MissingEnvironmentTraitError,
  type PageVisibilityTrait,
  type UiStorageTrait,
  type ViewportTrait,
} from "./traits";

export interface CreateKonobanguEnvironmentForNativeWebOptions {
  window: Window;
  config: RuntimeConfig;
  auth: AuthSetup;
  appRouter: AnyRouter;
  providers?: readonly SecuritydeptProvider[];
}

function createMediaQueryForNativeWeb(
  view: Window,
  query: string,
): MediaQueryTrait {
  if (typeof view.matchMedia !== "function")
    throw new MissingEnvironmentTraitError("matchMedia");
  const list = view.matchMedia(query);
  return {
    get matches() {
      return list.matches;
    },
    changes(listener) {
      const change = (event: MediaQueryListEvent) => listener(event.matches);
      list.addEventListener("change", change);
      return () => list.removeEventListener("change", change);
    },
  };
}

function createUiStorageForNativeWeb(
  view: Window,
  kind: "localStorage" | "sessionStorage",
  prefix: string,
): UiStorageTrait {
  try {
    const target = view[kind];
    const probe = `${prefix}availability`;
    target.setItem(probe, "1");
    target.removeItem(probe);
    const result = createStorageForNativeWeb({
      storage: target,
      storageEventTarget: view,
      prefix,
      validatorKey:
        kind === "localStorage" ? "persistentStorage" : "sessionStorage",
    });
    if (!result?.storageEvent) throw new MissingEnvironmentTraitError(kind);
    return { ...result, storageEvent: result.storageEvent };
  } catch {
    return createUiStorageForMemory();
  }
}

function createPageVisibilityForNativeWeb(
  document: Document,
): PageVisibilityTrait {
  return {
    get visible() {
      return document.visibilityState !== "hidden";
    },
    changes(listener) {
      document.addEventListener("visibilitychange", listener);
      return () => document.removeEventListener("visibilitychange", listener);
    },
  };
}

function createViewportForNativeWeb(): ViewportTrait {
  const watchers = new Map<Element, () => void>();
  let observer: IntersectionObserver | undefined;
  return {
    nearViewport(element, run) {
      observer ??= new IntersectionObserver(
        (entries) => {
          for (const entry of entries)
            if (entry.isIntersecting) watchers.get(entry.target)?.();
        },
        { rootMargin: "300px" },
      );
      watchers.set(element, run);
      observer.observe(element);
      return () => {
        watchers.delete(element);
        observer?.unobserve(element);
        if (!watchers.size) {
          observer?.disconnect();
          observer = undefined;
        }
      };
    },
  };
}

/** The browser host: every native object this application uses is read here. */
export function createKonobanguEnvironmentForNativeWeb(
  options: CreateKonobanguEnvironmentForNativeWebOptions,
): KonobanguEnvironment {
  const { window: view, config, auth, appRouter, providers } = options;
  const document = view.document;
  if (typeof view.fetch !== "function")
    throw new MissingEnvironmentTraitError("fetch");
  const locale = view.navigator.languages[0] ?? view.navigator.language;
  if (!locale) throw new MissingEnvironmentTraitError("navigator language");
  const userAgent = view.navigator.userAgent;
  return createKonobanguEnvironment({
    createBaseEnvironment: (baseOptions) =>
      createEnvironmentForReact({
        ...baseOptions,
        createBaseEnvironment: createEnvironmentForNativeWeb,
      }),
    traits: {
      document,
      fetch: view.fetch.bind(view),
      locale,
      platformInfo: { userAgent },
      systemDarkTheme: createMediaQueryForNativeWeb(
        view,
        "(prefers-color-scheme: dark)",
      ),
      mobileViewport: createMediaQueryForNativeWeb(view, "(max-width: 767px)"),
      uiPreferenceStorage: createUiStorageForNativeWeb(
        view,
        "localStorage",
        "konobangu.ui:",
      ),
      uiSessionStorage: createUiStorageForNativeWeb(
        view,
        "sessionStorage",
        "konobangu.ui:",
      ),
      pageVisibility: createPageVisibilityForNativeWeb(document),
      viewport: createViewportForNativeWeb(),
      resourceCrypto: createResourceCrypto(view.crypto),
    },
    config,
    auth,
    documentOrigin: view.location.origin,
    appRouter,
    sessionStorage: createUiStorageForNativeWeb(
      view,
      "sessionStorage",
      "securitydept.web.client:",
    ),
    providers,
  });
}
