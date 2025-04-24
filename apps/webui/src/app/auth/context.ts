import { AuthService } from '@/domains/auth/auth.service';
import type { Injector, Provider } from '@outposts/injection-js';
import type { AnyRouter } from '@tanstack/react-router';
import {
  type CheckAuthResultEventType,
  provideAuth as provideOidcAuth,
  withCheckAuthResultEvent,
  withDefaultFeatures,
} from 'oidc-client-rx';
import { withTanstackRouter } from 'oidc-client-rx/adapters/@tanstack/react-router';
import type { Observable } from 'rxjs';
import {
  AppAuthMethod,
  AuthMethodEnum,
  type AuthMethodType,
  buildOidcConfig,
} from './config';

export function provideAuth(router: AnyRouter): Provider[] {
  const providers: Provider[] = [AuthService];
  if (AppAuthMethod === AuthMethodEnum.OIDC) {
    providers.push(
      ...provideOidcAuth(
        {
          config: buildOidcConfig(),
        },
        withDefaultFeatures({
          router: { enabled: false },
          securityStorage: { type: 'local-storage' },
        }),
        withTanstackRouter(router),
        withCheckAuthResultEvent()
      )
    );
  }
  return providers;
}

export function setupAuthContext(injector: Injector) {
  const { authService } = authContextFromInjector(injector);
  authService.setup();
}

export interface AuthContext {
  type: AuthMethodType;
  authService: AuthService;
  isAuthenticated$: Observable<boolean>;
  userData$: Observable<{}>;
  checkAuthResultEvent$: Observable<CheckAuthResultEventType>;
}

export function authContextFromInjector(injector: Injector): AuthContext {
  const authService = injector.get(AuthService);
  return {
    type: AppAuthMethod,
    authService,
    isAuthenticated$: authService.isAuthenticated$,
    userData$: authService.userData$,
    checkAuthResultEvent$: authService.checkAuthResultEvent$,
  };
}
