import { CHECK_AUTH_RESULT_EVENT } from 'oidc-client-rx';
import {
  InjectorContextVoidInjector,
  useOidcClient,
} from 'oidc-client-rx/adapters/solid-js';
import { NEVER, of } from 'rxjs';
import { createMemo, from } from 'solid-js';
import { isBasicAuth, isOidcAuth } from './config';

const BASIC_AUTH_IS_AUTHENTICATED$ = of({
  isAuthenticated: true,
  allConfigsAuthenticated: [],
});

const BASIC_AUTH_USER_DATA$ = of({
  userData: {},
  allUserData: [],
});

const useOidcClientExt = isOidcAuth ? useOidcClient : () => ({ oidcSecurityService: undefined, injector: InjectorContextVoidInjector })

export function useAuth() {
  const { oidcSecurityService, injector } = useOidcClientExt();

  const isAuthenticatedObj = from(
    oidcSecurityService?.isAuthenticated$ ?? BASIC_AUTH_IS_AUTHENTICATED$
  );

  const userDataObj = from(
    oidcSecurityService?.userData$ ?? BASIC_AUTH_USER_DATA$
  );

  const isAuthenticated = createMemo(
    () => isAuthenticatedObj()?.isAuthenticated ?? false
  );

  const userData = createMemo(() => userDataObj()?.userData ?? {});

  const checkAuthResultEvent = isBasicAuth
    ? NEVER
    : injector.get(CHECK_AUTH_RESULT_EVENT);

  return {
    oidcSecurityService,
    isAuthenticated,
    userData,
    injector,
    checkAuthResultEvent,
  };
}
