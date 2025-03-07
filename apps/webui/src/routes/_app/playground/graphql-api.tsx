import { createFileRoute } from '@tanstack/solid-router';
import { AppSkeleton } from '~/components/layout/app-skeleton';
import { buildLeafRouteStaticData } from '~/utils/route';

export const Route = createFileRoute('/_app/playground/graphql-api')({
  staticData: buildLeafRouteStaticData({ title: 'GraphQL Api' }),
  pendingComponent: AppSkeleton,
});
