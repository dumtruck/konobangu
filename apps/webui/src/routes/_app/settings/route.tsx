import { createFileRoute } from '@tanstack/solid-router';
import { buildVirtualBranchRouteOptions } from '~/utils/route';

export const Route = createFileRoute('/_app/settings')(
  buildVirtualBranchRouteOptions({
    title: 'Settings',
  })
);
