import { CHECK_AUTH_RESULT_EVENT } from 'oidc-client-rx';
import {
  InjectorContextVoidInjector,
  useOidcClient,
} from 'oidc-client-rx/adapters/solid-js';
import { NEVER, map, of } from 'rxjs';
import { from } from 'solid-js';
import { isBasicAuth, isOidcAuth } from './config';

const BASIC_AUTH_IS_AUTHENTICATED$ = of({
  isAuthenticated: true,
  allConfigsAuthenticated: [],
});

const BASIC_AUTH_USER_DATA$ = of({
  userData: {},
  allUserData: [],
});

const useOidcClientExt = isOidcAuth
  ? useOidcClient
  : () => ({
      oidcSecurityService: undefined,
      injector: InjectorContextVoidInjector,
    });

export function useAuth() {
  const { oidcSecurityService, injector } = useOidcClientExt();

  const isAuthenticated$ = (
    oidcSecurityService?.isAuthenticated$ ?? BASIC_AUTH_IS_AUTHENTICATED$
  ).pipe(map((s) => s.isAuthenticated));

  const userData$ = (
    oidcSecurityService?.userData$ ?? BASIC_AUTH_USER_DATA$
  ).pipe(map((s) => s.userData));

  const isAuthenticated = from(isAuthenticated$);

  const userData = from(userData$);

  const checkAuthResultEvent$ = isBasicAuth
    ? NEVER
    : injector.get(CHECK_AUTH_RESULT_EVENT);

  return {
    oidcSecurityService,
    isAuthenticated$,
    isAuthenticated,
    userData$,
    userData,
    injector,
    checkAuthResultEvent$,
  };
}
