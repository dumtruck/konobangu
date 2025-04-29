import { useInjector } from 'oidc-client-rx/adapters/react';
import { useMemo } from 'react';
import { intlContextFromInjector } from './context';

export function useIntl() {
  const injector = useInjector();

  return useMemo(() => intlContextFromInjector(injector), [injector]);
}
