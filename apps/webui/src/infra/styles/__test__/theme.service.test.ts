// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import { createKonobanguFixture } from "@/__test__/support/fixture";
import { THEME_STORAGE_KEY, ThemeService } from "../theme.service";

describe("theme preference and environment traits", () => {
  it("reacts to SDK storage events and media, then releases listeners", () => {
    const { environment, injector } = createKonobanguFixture();
    const theme = injector.get(ThemeService);
    theme.setPreference("dark");
    expect(document.documentElement.classList.contains("dark")).toBe(true);
    environment.uiPreferenceStorage.set(THEME_STORAGE_KEY, "light");
    expect(theme.preference.get()).toBe("light");
    environment.systemDarkTheme.set(true);
    theme.setPreference("system");
    expect(theme.colorTheme.get()).toBe("dark");
    expect("dispose" in theme).toBe(false);
    expect(environment.systemDarkTheme.listenerCount).toBe(1);
    environment[Symbol.dispose]();
    expect(environment.systemDarkTheme.listenerCount).toBe(0);
    expect(environment.mobileViewport.listenerCount).toBe(0);
  });
});
