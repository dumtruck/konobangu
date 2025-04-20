import { UnreachableError } from '@/errors/common';
import type { Injector, Provider } from '@outposts/injection-js';
import type { AnyRouter } from '@tanstack/react-router';
import { atomSignal } from 'jotai-signal';
import type { Atom } from 'jotai/vanilla';
import {
  CHECK_AUTH_RESULT_EVENT,
  type CheckAuthResultEventType,
  OidcSecurityService,
  provideAuth as provideOidcAuth,
  withCheckAuthResultEvent,
  withDefaultFeatures,
} from 'oidc-client-rx';
import { withTanstackRouter } from 'oidc-client-rx/adapters/@tanstack/react-router';
import { NEVER, type Observable, map, of } from 'rxjs';
import { AppAuthMethod, AuthMethodEnum, buildOidcConfig } from './config';

export function provideAuth(router: AnyRouter): Provider[] {
  if (AppAuthMethod === AuthMethodEnum.OIDC) {
    return provideOidcAuth(
      {
        config: buildOidcConfig(),
      },
      withDefaultFeatures({
        router: { enabled: false },
        securityStorage: { type: 'local-storage' },
      }),
      withTanstackRouter(router),
      withCheckAuthResultEvent()
    );
  }
  return [];
}

export function setupAuthContext(injector: Injector) {
  if (AppAuthMethod === AuthMethodEnum.OIDC) {
    const oidcSecurityService = injector.get(OidcSecurityService);
    oidcSecurityService.checkAuth().subscribe();
  }
}
export interface OidcAuthContext {
  type: typeof AuthMethodEnum.OIDC;
  oidcSecurityService: OidcSecurityService;
  isAuthenticated$: Observable<boolean>;
  userData$: Observable<any>;
  checkAuthResultEvent$: Observable<CheckAuthResultEventType>;
}

export interface BasicAuthContext {
  type: typeof AuthMethodEnum.BASIC;
  isAuthenticated$: Observable<true>;
  userData$: Observable<{}>;
  checkAuthResultEvent$: Observable<CheckAuthResultEventType>;
}

export type AuthContext = OidcAuthContext | BasicAuthContext;

const BASIC_AUTH_IS_AUTHENTICATED$ = of(true) as Observable<true>;

const BASIC_AUTH_USER_DATA$ = of({});

export function authContextFromInjector(injector: Injector): AuthContext {
  if (AppAuthMethod === AuthMethodEnum.OIDC) {
    const oidcSecurityService = injector.get(OidcSecurityService)!;

    return {
      type: AuthMethodEnum.OIDC,
      oidcSecurityService: injector.get(OidcSecurityService),
      isAuthenticated$: oidcSecurityService.isAuthenticated$.pipe(
        map((s) => s.isAuthenticated)
      ),
      userData$: oidcSecurityService.userData$.pipe(map((s) => s.userData)),
      checkAuthResultEvent$: injector.get(CHECK_AUTH_RESULT_EVENT),
    };
  }
  if (AppAuthMethod === AuthMethodEnum.BASIC) {
    return {
      type: AuthMethodEnum.BASIC,
      isAuthenticated$: BASIC_AUTH_IS_AUTHENTICATED$,
      userData$: BASIC_AUTH_USER_DATA$,
      checkAuthResultEvent$: NEVER,
    };
  }
  throw new UnreachableError('Invalid auth method');
}
