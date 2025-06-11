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

  private setupSilentRenew() {
    const parent = document.defaultView?.parent;
    if (parent) {
      const event = new CustomEvent('oidc-silent-renew-message', {
        detail: document.defaultView?.location!,
      });
      parent.dispatchEvent(event);
    }
  }

  setup() {
    this.oidcSecurityService.checkAuth().subscribe(() => {
      this.setupSilentRenew();
    });
  }

  get isAuthenticated$() {
    return this.oidcSecurityService.isAuthenticated$.pipe(
      map((s) => s.isAuthenticated)
    );
  }

  get authData$() {
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
