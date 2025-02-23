import { useObservableEagerState, useObservableState } from 'observable-hooks';
import {
  InjectorContextVoidInjector,
  useOidcClient,
} from 'oidc-client-rx/adapters/react';
import { useMemo } from 'react';
import { NEVER, type Observable, of } from 'rxjs';
import { isBasicAuth } from './config';
import {
  CHECK_AUTH_RESULT_EVENT,
  type CheckAuthResultEventType,
} from './event';

const BASIC_AUTH_IS_AUTHENTICATED$ = of({
  isAuthenticated: true,
  allConfigsAuthenticated: [],
});

const BASIC_AUTH_USER_DATA$ = of({
  userData: {},
  allUserData: [],
});

export function useAuth() {
  const { oidcSecurityService, injector } = isBasicAuth
    ? { oidcSecurityService: undefined, injector: InjectorContextVoidInjector }
    : // biome-ignore lint/correctness/useHookAtTopLevel: <explanation>
      useOidcClient();

  const { isAuthenticated } = useObservableEagerState(
    oidcSecurityService?.isAuthenticated$ ?? BASIC_AUTH_IS_AUTHENTICATED$
  );

  const { userData } = useObservableEagerState(
    oidcSecurityService?.userData$ ?? BASIC_AUTH_USER_DATA$
  );

  const checkAuthResultEvent = useObservableState(
    useMemo(
      () => (isBasicAuth ? NEVER : injector.get(CHECK_AUTH_RESULT_EVENT)),
      [injector]
    ) as Observable<CheckAuthResultEventType>
  );

  return {
    oidcSecurityService,
    isAuthenticated,
    userData,
    injector,
    checkAuthResultEvent,
  };
}
