import { buildVirtualBranchRouteOptions } from '@/utils/route';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_app/subscriptions')(
  buildVirtualBranchRouteOptions({
    title: 'Subscriptions',
  })
);
