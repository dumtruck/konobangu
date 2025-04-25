import { injectInjector } from '@/infra/di/inject';
import { inject, runInInjectionContext } from '@outposts/injection-js';
import {
  CHECK_AUTH_RESULT_EVENT,
  OidcSecurityService,
  autoLoginPartialRoutesGuard,
} from 'oidc-client-rx';
import { type Observable, map } from 'rxjs';
import { AuthProvider } from '../auth.provider';
import { AUTH_METHOD } from '../defs';

export class OidcAuthProvider extends AuthProvider {
  authMethod = AUTH_METHOD.OIDC;
  oidcSecurityService = inject(OidcSecurityService);
  checkAuthResultEvent$ = inject(CHECK_AUTH_RESULT_EVENT);
  injector = injectInjector();

  setup() {
    this.oidcSecurityService.checkAuth().subscribe();
  }

  get isAuthenticated$() {
    return this.oidcSecurityService.isAuthenticated$.pipe(
      map((s) => s.isAuthenticated)
    );
  }

  get userData$() {
    return this.oidcSecurityService.userData$.pipe(map((s) => s.userData));
  }

  getAccessToken(): Observable<string | undefined> {
    return this.oidcSecurityService.getAccessToken();
  }

  autoLoginPartialRoutesGuard() {
    return runInInjectionContext(this.injector, () =>
      autoLoginPartialRoutesGuard()
    );
  }
}
