import type { RouteStateDataOption } from '@/infra/routes/traits';
import { guardRouteIndexAsNotFound } from '@/views/components/layout/app-not-found';
import { Outlet } from '@tanstack/react-router';

export interface BuildVirtualBranchRouteOptions {
  title: string;
}

export function buildVirtualBranchRouteOptions(
  options: BuildVirtualBranchRouteOptions
) {
  return {
    beforeLoad: guardRouteIndexAsNotFound,
    staticData: {
      breadcrumb: {
        label: options.title,
        link: undefined,
      },
    } satisfies RouteStateDataOption,
    component: Outlet,
  };
}

export interface BuildLeafRouteStaticDataOptions {
  title: string;
}

export function buildLeafRouteStaticData(
  options: BuildLeafRouteStaticDataOptions
): RouteStateDataOption {
  return {
    breadcrumb: {
      label: options.title,
    },
  };
}
