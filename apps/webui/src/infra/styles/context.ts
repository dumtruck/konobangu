import { useSignal } from "@securitydept/client-react";
import { APP_LIFETIME, type AppLifetime } from "@/app/lifetime/scope";
import { useInject } from "@/infra/di/inject";
import { type PreferColorSchemaType, ThemeService } from "./theme.service";

export function provideStyles() {
  return [
    {
      provide: ThemeService,
      useFactory: (lifetime: AppLifetime) => ThemeService.createOwned(lifetime),
      deps: [APP_LIFETIME],
    },
  ];
}

export function useTheme() {
  const themeService = useInject(ThemeService);
  return {
    colorTheme: useSignal(themeService.colorTheme),
    preference: useSignal(themeService.preference),
    setPreference: (preference: PreferColorSchemaType) =>
      themeService.setPreference(preference),
  };
}
