import type { GetSubscriptionDetailQuery } from '@/infra/graphql/gql/graphql';
import { useQuery } from '@apollo/client';
import { createFileRoute } from '@tanstack/react-router';
import { GET_SUBSCRIPTION_DETAIL } from '../../../../domains/recorder/schema/subscriptions.js';

export const Route = createFileRoute(
  '/_app/subscriptions/detail/$subscriptionId'
)({
  component: DetailRouteComponent,
});

function DetailRouteComponent() {
  const { subscriptionId } = Route.useParams();
  const { data, loading, error } = useQuery<GetSubscriptionDetailQuery>(
    GET_SUBSCRIPTION_DETAIL,
    {
      variables: {
        id: Number.parseInt(subscriptionId),
      },
    }
  );

  if (loading) {
    return <div>Loading...</div>;
  }

  if (error) {
    return <div>Error: {error.message}</div>;
  }

  const detail = data?.subscriptions?.nodes?.[0];

  return (
    <div
      dangerouslySetInnerHTML={{ __html: JSON.stringify(detail, null, 2) }}
    />
  );
}
