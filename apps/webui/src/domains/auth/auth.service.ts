import { AppAuthMethod, AuthMethodEnum } from '@/app/auth/config';
import { injectInjector } from '@/infra/di/inject';
import { Injectable, type Injector } from '@outposts/injection-js';
import {
  CHECK_AUTH_RESULT_EVENT,
  type CheckAuthResultEventType,
  OidcSecurityService,
} from 'oidc-client-rx';
import { NEVER, type Observable, map, of } from 'rxjs';

const BASIC_AUTH_IS_AUTHENTICATED$ = of(true) as Observable<true>;

const BASIC_AUTH_USER_DATA$ = of({});

@Injectable()
export class AuthService {
  private injector: Injector = injectInjector();
  oidcSecurityService: OidcSecurityService | undefined;
  checkAuthResultEvent$: Observable<CheckAuthResultEventType>;
  constructor() {
    if (AppAuthMethod === 'oidc') {
      this.oidcSecurityService = this.injector.get(OidcSecurityService);
      this.checkAuthResultEvent$ = this.injector.get(CHECK_AUTH_RESULT_EVENT);
    } else {
      this.checkAuthResultEvent$ = NEVER;
    }
  }

  setup() {
    if (AppAuthMethod === AuthMethodEnum.OIDC) {
      this.oidcSecurityService!.checkAuth().subscribe();
    }
  }

  get isAuthenticated$() {
    return (
      this.oidcSecurityService?.isAuthenticated$.pipe(
        map((s) => s.isAuthenticated)
      ) ?? BASIC_AUTH_IS_AUTHENTICATED$
    );
  }

  get userData$() {
    return (
      this.oidcSecurityService?.userData$?.pipe(map((s) => s.userData)) ??
      BASIC_AUTH_USER_DATA$
    );
  }

  getAccessToken(): Observable<string | undefined> {
    return this.oidcSecurityService?.getAccessToken() ?? of(undefined);
  }
}
