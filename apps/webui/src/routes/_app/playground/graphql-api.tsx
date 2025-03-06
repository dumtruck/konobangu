import { createFileRoute } from '@tanstack/solid-router';
import { buildLeafRouteStaticData } from '~/utils/route';

export const Route = createFileRoute('/_app/playground/graphql-api')({
  staticData: buildLeafRouteStaticData({ title: 'GraphQL Api' }),
});
