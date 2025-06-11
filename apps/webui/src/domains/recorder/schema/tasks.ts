import {
  type GetTasksQuery,
  SubscriberTaskTypeEnum,
} from '@/infra/graphql/gql/graphql';
import { gql } from '@apollo/client';
import { type } from 'arktype';
import { SubscriptionSchema } from './subscriptions';

export const GET_TASKS = gql`
  query GetTasks($filters: SubscriberTasksFilterInput!, $orderBy: SubscriberTasksOrderInput!, $pagination: PaginationInput!) {
    subscriberTasks(
      pagination: $pagination
      filters: $filters
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
