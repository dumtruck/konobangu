import { ThemeService } from './theme.service';

export function provideStyles() {
  return [
    {
      provide: ThemeService,
      useClass: ThemeService,
    },
  ];
}
