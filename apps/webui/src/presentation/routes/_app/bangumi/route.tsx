import { buildVirtualBranchRouteOptions } from '@/infra/routes/utils';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_app/bangumi')(
  buildVirtualBranchRouteOptions({
    title: 'Bangumi',
  })
);
