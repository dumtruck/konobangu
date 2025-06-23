import { gql } from '@apollo/client';

export const INSERT_FEED = gql`
    mutation InsertFeed($data: FeedsInsertInput!) {
        feedsCreateOne(data: $data) {
            id
            createdAt
            updatedAt
            feedType
            token
        }
    }
`;

export const DELETE_FEED = gql`
    mutation DeleteFeed($filters: FeedsFilterInput!) {
        feedsDelete(filter: $filters)
    }
`;
