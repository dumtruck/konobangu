import type { Injector } from '@outposts/injection-js';
import type { OidcSecurityService } from 'oidc-client-rx';
import { type Accessor, createSignal } from 'solid-js';
import { isBasicAuth } from './config';

export const [isAuthenticated, setIsAuthenticated] = createSignal(isBasicAuth);

export type RouterContext = {
  isAuthenticated: Accessor<boolean>;
  injector: Injector;
  oidcSecurityService: OidcSecurityService;
};
