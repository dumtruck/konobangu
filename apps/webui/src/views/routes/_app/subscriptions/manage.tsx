import type { GetSubscriptionsQuery } from '@/infra/graphql/gql/graphql';
import type { RouteStateDataOption } from '@/infra/routes/traits';
import { Badge } from '@/views/components/ui/badge';
import { Button } from '@/views/components/ui/button';
import {
  Card,
  CardFooter,
  CardHeader,
  CardTitle,
} from '@/views/components/ui/card';
import {
  HoverCard,
  HoverCardContent,
  HoverCardTrigger,
} from '@/views/components/ui/hover-card';
import { Image } from '@/views/components/ui/image';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/views/components/ui/select';
import { useQuery } from '@apollo/client';
import { gql } from '@apollo/client';
import { createFileRoute } from '@tanstack/react-router';
import { useNavigate } from '@tanstack/react-router';
import { useState } from 'react';

export const Route = createFileRoute('/_app/subscriptions/manage')({
  component: SubscriptionManageRouteComponent,
  staticData: {
    breadcrumb: { label: 'Manage' },
  } satisfies RouteStateDataOption,
});

// GraphQL query
const GET_SUBSCRIPTIONS = gql`
  query GetSubscriptions($page: Int!, $pageSize: Int!) {
    subscriptions(
      pagination: {
        page: {
          page: $page,
          limit: $pageSize
        }
      }
    ) {
      nodes {
        id
        displayName
        category
        enabled
        bangumi {
          nodes {
            id
            displayName
            posterLink
            season
            fansub
            homepage
          }
        }
      }
    }
  }
`;

function SubscriptionManageRouteComponent() {
  const navigate = useNavigate();
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(10);
  const [sortBy, setSortBy] = useState('createdAt');
  const [sortOrder, setSortOrder] = useState('desc');

  const { loading, error, data } = useQuery<GetSubscriptionsQuery>(
    GET_SUBSCRIPTIONS,
    {
      variables: {
        page,
        pageSize,
        sortBy,
        sortOrder,
      },
    }
  );

  if (loading) {
    return <div>Loading...</div>;
  }
  if (error) {
    return <div>Error: {error.message}</div>;
  }

  const { nodes: items } = data?.subscriptions ?? {};
  const totalPages = Math.ceil(total / pageSize);

  return (
    <div className="container mx-auto space-y-4 p-4">
      {/* Filters and sorting controls */}
      <div className="flex items-center gap-4">
        <Select
          value={pageSize.toString()}
          onValueChange={(value) => setPageSize(Number(value))}
        >
          <SelectTrigger className="w-[180px]">
            <SelectValue placeholder="Items per page" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="12">12 items/page</SelectItem>
            <SelectItem value="24">24 items/page</SelectItem>
            <SelectItem value="48">48 items/page</SelectItem>
          </SelectContent>
        </Select>

        <Select value={sortBy} onValueChange={setSortBy}>
          <SelectTrigger className="w-[180px]">
            <SelectValue placeholder="Sort by" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="createdAt">Created At</SelectItem>
            <SelectItem value="displayName">Name</SelectItem>
            <SelectItem value="season">Season</SelectItem>
          </SelectContent>
        </Select>

        <Select value={sortOrder} onValueChange={setSortOrder}>
          <SelectTrigger className="w-[180px]">
            <SelectValue placeholder="Sort order" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="asc">Ascending</SelectItem>
            <SelectItem value="desc">Descending</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Subscription list */}
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
        {items.map((subscription) => (
          <Card key={subscription.id} className="overflow-hidden">
            <HoverCard>
              <HoverCardTrigger>
                <div className="relative aspect-[2/3] w-full">
                  <Image
                    src={subscription.bangumi.posterLink || '/placeholder.png'}
                    alt={subscription.bangumi.displayName}
                    className="h-full w-full object-cover"
                  />
                </div>
              </HoverCardTrigger>
              <HoverCardContent className="w-80">
                <Image
                  src={subscription.bangumi.posterLink || '/placeholder.png'}
                  alt={subscription.bangumi.displayName}
                  className="h-auto w-full"
                />
              </HoverCardContent>
            </HoverCard>

            <CardHeader>
              <CardTitle className="line-clamp-2">
                {subscription.bangumi.extra?.nameZh ||
                  subscription.bangumi.displayName}
              </CardTitle>
              <div className="flex gap-2">
                <Badge variant={subscription.enabled ? 'default' : 'secondary'}>
                  {subscription.enabled ? 'Enabled' : 'Disabled'}
                </Badge>
                <Badge variant="outline">
                  {subscription.bangumi.fansub || 'Unknown Group'}
                </Badge>
              </div>
            </CardHeader>

            <CardFooter className="flex justify-between">
              <Button
                variant="outline"
                onClick={() =>
                  navigate({ to: `/subscriptions/${subscription.id}` })
                }
              >
                View Details
              </Button>
              <Button
                onClick={() =>
                  navigate({ to: `/subscriptions/${subscription.id}/edit` })
                }
              >
                Edit
              </Button>
            </CardFooter>
          </Card>
        ))}
      </div>

      {/* Pagination controls */}
      <div className="flex justify-center gap-2">
        <Button
          variant="outline"
          onClick={() => setPage(page - 1)}
          disabled={page === 1}
        >
          Previous
        </Button>
        <span className="py-2">
          Page {page} of {totalPages}
        </span>
        <Button
          variant="outline"
          onClick={() => setPage(page + 1)}
          disabled={page === totalPages}
        >
          Next
        </Button>
      </div>
    </div>
  );
}
