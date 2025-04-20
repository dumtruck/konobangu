import { DOCUMENT } from '@/platform/injection';
import { LocalStorageService } from '@/storage/web-storage.service';
import { Injectable, inject } from '@outposts/injection-js';

export type PreferColorSchemaType = 'dark' | 'light' | 'system';
export type PreferColorSchemaClass = 'dark' | 'light';

@Injectable()
export class ThemeService {
  document = inject(DOCUMENT);
  localStorage = inject(LocalStorageService);

  get systemColorSchema(): PreferColorSchemaClass {
    return this.document.defaultView?.matchMedia('(prefers-color-scheme: dark)')
      .matches
      ? 'dark'
      : 'light';
  }

  private getColorSchemaByType(
    themeType: PreferColorSchemaType
  ): PreferColorSchemaClass {
    this.document.documentElement.classList.remove('dark', 'light');
    if (themeType === 'dark' || themeType === 'light') {
      return themeType;
    }
    return this.systemColorSchema;
  }

  get colorSchema() {
    const theme = this.localStorage.getItem('prefers-color-scheme');
    return this.getColorSchemaByType(theme as PreferColorSchemaType);
  }

  set colorSchema(themeType: PreferColorSchemaType) {
    this.localStorage.setItem('prefers-color-scheme', themeType);
    const themeClass = this.getColorSchemaByType(themeType);
    this.document.documentElement.classList.remove('dark', 'light');
    this.document.documentElement.classList.add(themeClass);
  }
}
