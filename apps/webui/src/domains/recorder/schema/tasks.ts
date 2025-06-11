import { gql } from '@apollo/client';

export const GET_TASKS = gql`
  query GetTasks($filters: SubscriberTasksFilterInput!, $orderBy: SubscriberTasksOrderInput!, $pagination: PaginationInput!) {
    subscriberTasks(
      pagination: $pagination
      filters: $filters
      orderBy: $orderBy
    ) {
      nodes {
        id,
        status,
        attempts,
        maxAttempts,
        runAt,
        lastError,
        lockAt,
        lockBy,
        doneAt,
        priority
      }
      paginationInfo {
        total
        pages
      }
    }
  }
`;
