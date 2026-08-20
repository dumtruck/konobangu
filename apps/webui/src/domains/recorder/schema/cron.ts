import { gql } from "@/infra/graphql/gql";
import type { GetCronsQuery } from "@/infra/graphql/gql/graphql";

// A cron row is the plan. Attempts, leases and outcomes belong to its task runs,
// which carry the retry command, so the plan selects no execution fields.
export const GET_CRONS = gql(`
query GetCrons($filter: CronFilterInput!, $orderBy: CronOrderInput!, $pagination: PaginationInput!) {
    cron(pagination: $pagination, filter: $filter, orderBy: $orderBy) {
        nodes {
            id
            cronExpr
            cronTimezone
            nextRun
            lastRun
            lastError
            createdAt
            updatedAt
            enabled
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
                    generation,
                    cancelRequestedAt,
                    doneAt,
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
`);

export type CronDto = GetCronsQuery["cron"]["nodes"][number];

export const DELETE_CRONS = gql(`
    mutation DeleteCrons($filter: CronFilterInput!) {
        cronDelete(filter: $filter)
    }
`);

export const UPDATE_CRONS = gql(`
    mutation UpdateCrons($filter: CronFilterInput!, $data: CronUpdateInput!) {
        cronUpdate(filter: $filter, data: $data) {
            id
            cronExpr
            cronTimezone
            nextRun
            lastRun
            lastError
            createdAt
            updatedAt
            enabled
            subscriberTaskCron
        }
    }
`);

export const INSERT_CRON = gql(`
    mutation InsertCron($data: CronInsertInput!) {
        cronCreateOne(data: $data) {
            id
            cronExpr
            cronTimezone
            nextRun
            lastRun
            lastError
            createdAt
            updatedAt
            enabled
            subscriberTaskCron
        }
    }
`);
