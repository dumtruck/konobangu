import {
  type GetTasksQuery,
  SubscriberTaskTypeEnum,
} from '@/infra/graphql/gql/graphql';
import { gql } from '@apollo/client';
import { type } from 'arktype';
import { SubscriptionSchema } from './subscriptions';

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

export const TaskTypedSyncOneSubscriptionFeedsIncrementalSchema = type({
  taskType: `'${SubscriberTaskTypeEnum.SyncOneSubscriptionFeedsIncremental}'`,
}).and(SubscriptionSchema);

export const TaskTypedSyncOneSubscriptionFeedsFullSchema = type({
  taskType: `'${SubscriberTaskTypeEnum.SyncOneSubscriptionFeedsFull}'`,
}).and(SubscriptionSchema);

export const TaskTypedSyncOneSubscriptionSourcesSchema = type({
  taskType: `'${SubscriberTaskTypeEnum.SyncOneSubscriptionSources}'`,
}).and(SubscriptionSchema);

export const TaskTypedSchema = TaskTypedSyncOneSubscriptionFeedsFullSchema.or(
  TaskTypedSyncOneSubscriptionFeedsIncrementalSchema
).or(TaskTypedSyncOneSubscriptionSourcesSchema);

export type TaskTypedDto = typeof TaskTypedSchema.infer;

export type TaskDto = Omit<
  GetTasksQuery['subscriberTasks']['nodes'][number],
  'job'
> & {
  job: TaskTypedDto;
};
