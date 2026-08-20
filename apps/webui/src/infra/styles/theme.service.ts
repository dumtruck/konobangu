import {
  createComputed,
  createSignal,
  inject,
  readonlySignal,
} from "@securitydept/client";
import {
  DOCUMENT,
  MOBILE_VIEWPORT,
  SYSTEM_DARK_THEME,
  UI_PREFERENCE_STORAGE,
} from "@/app/environment/traits";
import type { AppLifetime } from "@/app/lifetime/scope";

export type PreferColorSchemaType = "dark" | "light" | "system";
export type PreferColorSchemaClass = "dark" | "light";
export const THEME_STORAGE_KEY = "prefers-color-scheme";
export class ThemeService {
  private readonly document = inject(DOCUMENT);
  private readonly storage = inject(UI_PREFERENCE_STORAGE);
  private readonly dark = inject(SYSTEM_DARK_THEME);
  private readonly mobile = inject(MOBILE_VIEWPORT);
  private readonly preferenceState = createSignal(
    this.parse(this.storage.get(THEME_STORAGE_KEY)),
  );
  private readonly systemState = createSignal<PreferColorSchemaClass>(
    this.dark.matches ? "dark" : "light",
  );
  private readonly mobileState = createSignal(this.mobile.matches);
  readonly preference = readonlySignal(this.preferenceState);
  readonly systemTheme = readonlySignal(this.systemState);
  readonly isMobile = readonlySignal(this.mobileState);
  readonly colorTheme = createComputed(() =>
    this.preferenceState.get() === "system"
      ? this.systemState.get()
      : this.preferenceState.get(),
  );

  static createOwned(lifetime: AppLifetime): ThemeService {
    using rollback = new DisposableStack();
    lifetime.signal.throwIfAborted();
    const service = new ThemeService();
    service.apply();
    rollback.defer(
      service.dark.changes((matches) => {
        service.systemState.set(matches ? "dark" : "light");
        service.apply();
      }),
    );
    rollback.defer(
      service.mobile.changes((matches) => service.mobileState.set(matches)),
    );
    const subscription = service.storage.storageEvent.subscribe({
      next: (event) => {
        if (event.key !== null && event.key !== THEME_STORAGE_KEY) return;
        service.preferenceState.set(service.parse(event.newValue));
        service.apply();
      },
    });
    rollback.defer(() => subscription.unsubscribe());
    const owner = rollback.move();
    return lifetime.own(service, () => owner.dispose());
  }
  setPreference(preference: PreferColorSchemaType): void {
    this.preferenceState.set(preference);
    this.apply();
    this.storage.set(THEME_STORAGE_KEY, preference);
  }
  private parse(value: string | null): PreferColorSchemaType {
    return value === "dark" || value === "light" ? value : "system";
  }
  private apply(): void {
    this.document.documentElement.classList.remove("dark", "light");
    this.document.documentElement.classList.add(this.colorTheme.get());
  }
}
