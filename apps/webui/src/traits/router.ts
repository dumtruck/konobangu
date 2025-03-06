import type { Injector } from '@outposts/injection-js';
import type { LucideIcon } from 'lucide-solid';
import type { OidcSecurityService } from 'oidc-client-rx';
import type { Accessor } from 'solid-js';
import type { ProLinkProps } from '~/components/ui/pro-link';

export type RouterContext = {
  isAuthenticated: Accessor<boolean>;
  injector: Injector;
  oidcSecurityService: OidcSecurityService;
};

export type RouteBreadcrumbItem = {
  label?: string;
  icon?: LucideIcon;
  link?: Omit<ProLinkProps, 'aria-current' | 'current'>;
};

export interface RouteStateDataOption {
  breadcrumb?: RouteBreadcrumbItem;
}
