import { DOCUMENT } from '@/infra/platform/injection';
import { LocalStorageService } from '@/infra/storage/web-storage.service';
import { Injectable, inject } from '@outposts/injection-js';
import {
  BehaviorSubject,
  ReplaySubject,
  combineLatest,
  distinctUntilChanged,
  filter,
  fromEvent,
  map,
  shareReplay,
  startWith,
} from 'rxjs';
export type PreferColorSchemaType = 'dark' | 'light' | 'system';
export type PreferColorSchemaClass = 'dark' | 'light';

@Injectable()
export class ThemeService {
  document = inject(DOCUMENT);
  localStorage = inject(LocalStorageService);
  systemColorSchema$ = new BehaviorSubject(this.systemColorSchema);
  storageColorSchema$ = new BehaviorSubject(
    this.getColorSchemaType(this.localStorage.getItem('prefers-color-scheme'))
  );
  colorSchema$ = new BehaviorSubject(
    this.getColorSchemaByType(
      this.storageColorSchema$.value,
      this.systemColorSchema$.value
    )
  );

  setup() {
    const mediaQuery = this.document.defaultView?.matchMedia(
      '(prefers-color-scheme: dark)'
    );

    if (mediaQuery) {
      fromEvent(mediaQuery, 'change')
        .pipe(
          map(() => (mediaQuery.matches ? 'dark' : 'light')),
          startWith(this.systemColorSchema),
          distinctUntilChanged()
        )
        .subscribe(this.systemColorSchema$);
    }

    if (this.document.defaultView?.localStorage) {
      fromEvent(this.document.defaultView, 'storage')
        .pipe(
          filter(
            (e): e is StorageEvent =>
              (e as StorageEvent)?.key === 'prefers-color-scheme'
          ),
          map((event) => this.getColorSchemaType(event.newValue)),
          distinctUntilChanged()
        )
        .subscribe(this.storageColorSchema$);
    }

    combineLatest({
      system: this.systemColorSchema$,
      storage: this.storageColorSchema$,
    })
      .pipe(
        map(({ system, storage }) => this.getColorSchemaByType(storage, system))
      )
      .subscribe(this.colorSchema$);
  }

  private getColorSchemaType(themeType: string | null): PreferColorSchemaType {
    if (themeType === 'dark' || themeType === 'light') {
      return themeType as PreferColorSchemaType;
    }
    return 'system';
  }

  private getColorSchemaByType(
    themeType: PreferColorSchemaType,
    systemColorSchema: PreferColorSchemaClass
  ): PreferColorSchemaClass {
    if (themeType === 'dark' || themeType === 'light') {
      return themeType;
    }
    return systemColorSchema;
  }

  get systemColorSchema(): PreferColorSchemaClass {
    return this.document.defaultView?.matchMedia('(prefers-color-scheme: dark)')
      .matches
      ? 'dark'
      : 'light';
  }

  get colorSchema() {
    return this.colorSchema$.value;
  }

  set colorSchema(themeType: PreferColorSchemaType) {
    this.localStorage.setItem('prefers-color-scheme', themeType);
    const themeClass = this.getColorSchemaByType(
      themeType,
      this.systemColorSchema
    );
    this.document.documentElement.classList.remove('dark', 'light');
    this.document.documentElement.classList.add(themeClass);
  }
}
