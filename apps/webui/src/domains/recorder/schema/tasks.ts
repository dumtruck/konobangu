import type { GetTasksQuery } from '@/infra/graphql/gql/graphql';
import { gql } from '@apollo/client';
import type { SubscriberTask } from 'recorder/bindings/SubscriberTask';

export const GET_TASKS = gql`
  query GetTasks($filter: SubscriberTasksFilterInput!, $orderBy: SubscriberTasksOrderInput!, $pagination: PaginationInput!) {
    subscriberTasks(
      pagination: $pagination
      filter: $filter
      orderBy: $orderBy
    ) {
      nodes {
        id,
        job,
        taskType,
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

export const INSERT_SUBSCRIBER_TASK = gql`
  mutation InsertSubscriberTask($data: SubscriberTasksInsertInput!) {
    subscriberTasksCreateOne(data: $data) {
      id
    }
  }
`;

export const DELETE_TASKS = gql`
  mutation DeleteTasks($filter: SubscriberTasksFilterInput!) {
    subscriberTasksDelete(filter: $filter)
  }
`;

export const RETRY_TASKS = gql`
  mutation RetryTasks($filter: SubscriberTasksFilterInput!) {
    subscriberTasksRetryOne(filter: $filter) {
        id,
        job,
        taskType,
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
  }
`;

export type SubscriberTaskInsertDto = Omit<SubscriberTask, 'subscriberId'>;

export type TaskDto = Omit<
  GetTasksQuery['subscriberTasks']['nodes'][number],
  'job'
> & {
  job: SubscriberTask;
};
