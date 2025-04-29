import { AppSkeleton } from '@/components/layout/app-skeleton';
import { buildLeafRouteStaticData } from '@/infra/routes/utils';
import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/_app/playground/graphql-api')({
  staticData: buildLeafRouteStaticData({ title: 'GraphQL Api' }),
  pendingComponent: AppSkeleton,
});
