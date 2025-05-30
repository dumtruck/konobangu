import {
  type InjectionToken,
  Injector,
  type Type,
  inject,
} from '@outposts/injection-js';
import { useInjector } from 'oidc-client-rx/adapters/react';
import { useMemo } from 'react';

export function injectInjector(): Injector {
  return inject(Injector as any as InjectionToken<Injector>);
}

export function useInject<T>(token: InjectionToken<T> | Type<T>): T {
  const injector = useInjector();
  return useMemo(() => injector.get(token), [injector, token]);
}
