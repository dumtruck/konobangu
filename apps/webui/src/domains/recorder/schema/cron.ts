import type { GetCronsQuery } from '@/infra/graphql/gql/graphql';
import { gql } from '@apollo/client';

export const GET_CRONS = gql`
query GetCrons($filter: CronFilterInput!, $orderBy: CronOrderInput!, $pagination: PaginationInput!) {
    cron(pagination: $pagination, filter: $filter, orderBy: $orderBy) {
        nodes {
            id
            cronExpr
            nextRun
            lastRun
            lastError
            status
            lockedAt
            lockedBy
            createdAt
            updatedAt
            timeoutMs
            maxAttempts
            priority
            attempts
            subscriberTaskCron
            subscriberTask {
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
                    priority,
                    subscription {
                        displayName
                        sourceUrl
                    }
                }
            }
        }
        paginationInfo {
            total
            pages
        }
    }
  }
`;

export type CronDto = GetCronsQuery['cron']['nodes'][number];

export const DELETE_CRONS = gql`
    mutation DeleteCrons($filter: CronFilterInput!) {
        cronDelete(filter: $filter)
    }
`;
