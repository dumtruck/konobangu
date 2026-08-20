import {
  createInMemoryRecordStore,
  type EventStreamTrait,
  type RouterTrait,
  SecuritydeptInjectionToken,
  type StorageChangeEvent,
  type SyncStorageTrait,
} from "@securitydept/client";

/**
 * Konobangu traits extend the SDK environment the same way SDK traits do: one
 * interface, one token, resolved once by an environment creator. Services
 * inject these tokens and never reach for a platform global of their own.
 */
export type FetchTrait = typeof fetch;
export interface PlatformInfoTrait {
  readonly userAgent: string;
}
export interface MediaQueryTrait {
  readonly matches: boolean;
  changes(listener: (matches: boolean) => void): () => void;
}
export interface UiStorageTrait extends SyncStorageTrait {
  readonly storageEvent: EventStreamTrait<StorageChangeEvent>;
}
export interface PageVisibilityTrait {
  readonly visible: boolean;
  changes(listener: () => void): () => void;
}
export interface ViewportTrait {
  nearViewport(element: Element, run: () => void): () => void;
}
export interface ResourceCryptoTrait {
  hmac(key: Uint8Array, message: Uint8Array): Promise<Uint8Array>;
}

export const DOCUMENT = new SecuritydeptInjectionToken<Document>(
  "DOCUMENT_TRAIT",
);
export const FETCH = new SecuritydeptInjectionToken<FetchTrait>("FETCH_TRAIT");
export const LOCALE = new SecuritydeptInjectionToken<string>("LOCALE_TRAIT");
export const PLATFORM_INFO = new SecuritydeptInjectionToken<PlatformInfoTrait>(
  "PLATFORM_INFO_TRAIT",
);
export const SYSTEM_DARK_THEME =
  new SecuritydeptInjectionToken<MediaQueryTrait>("SYSTEM_DARK_THEME_TRAIT");
export const MOBILE_VIEWPORT = new SecuritydeptInjectionToken<MediaQueryTrait>(
  "MOBILE_VIEWPORT_TRAIT",
);
export const UI_PREFERENCE_STORAGE =
  new SecuritydeptInjectionToken<UiStorageTrait>("UI_PREFERENCE_STORAGE_TRAIT");
export const UI_SESSION_STORAGE =
  new SecuritydeptInjectionToken<UiStorageTrait>("UI_SESSION_STORAGE_TRAIT");
export const PAGE_VISIBILITY =
  new SecuritydeptInjectionToken<PageVisibilityTrait>("PAGE_VISIBILITY_TRAIT");
export const VIEWPORT = new SecuritydeptInjectionToken<ViewportTrait>(
  "VIEWPORT_TRAIT",
);
export const RESOURCE_CRYPTO =
  new SecuritydeptInjectionToken<ResourceCryptoTrait>("RESOURCE_CRYPTO_TRAIT");
/** The SDK router after this application's authentication navigation is applied. */
export const APP_ROUTER = new SecuritydeptInjectionToken<RouterTrait>(
  "APP_ROUTER_TRAIT",
);

export class MissingEnvironmentTraitError extends Error {
  constructor(trait: string) {
    super(`This environment does not provide the required ${trait}.`);
    this.name = "MissingEnvironmentTraitError";
  }
}

/** Shared by hosts without usable persistence and by the test host. */
export function createUiStorageForMemory(): UiStorageTrait {
  const storage = createInMemoryRecordStore();
  if (!storage.storageEvent)
    throw new MissingEnvironmentTraitError("memory storage events");
  return { ...storage, storageEvent: storage.storageEvent };
}
