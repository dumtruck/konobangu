import type { CronPreset } from '@/components/domains/cron';
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

export const INSERT_CRON = gql`
    mutation InsertCron($data: CronInsertInput!) {
        cronCreateOne(data: $data) {
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
        }
    }
`;

export const SUBSCRIPTION_TASK_CRON_PRESETS: CronPreset[] = [
  {
    label: 'Daily at midnight',
    value: '0 0 0 * * *',
    description: 'Runs once daily at 00:00',
    category: 'daily',
  },
  {
    label: 'Daily at 9 AM',
    value: '0 0 9 * * *',
    description: 'Runs daily at 9:00 AM',
    category: 'daily',
  },
  {
    label: 'Weekdays at 9 AM',
    value: '0 0 9 * * 1-5',
    description: 'Runs Monday to Friday at 9:00 AM',
    category: 'weekly',
  },
  {
    label: 'Every Sunday',
    value: '0 0 0 * * 0',
    description: 'Runs every Sunday at midnight',
    category: 'weekly',
  },
  {
    label: 'First day of month',
    value: '0 0 0 1 * *',
    description: 'Runs on the 1st day of every month',
    category: 'monthly',
  },
  {
    label: 'Every year',
    value: '0 0 0 1 1 *',
    description: 'Runs on January 1st every year',
    category: 'yearly',
  },
];
