import { gql } from '@apollo/client';

import type {
  GetSubscriptionDetailQuery,
  GetSubscriptionsQuery,
} from '@/infra/graphql/gql/graphql';

export const GET_SUBSCRIPTIONS = gql`
  query GetSubscriptions(
  $page: PageInput!,
  $filters: SubscriptionsFilterInput!,
  $orderBy: SubscriptionsOrderInput!
) {
    subscriptions(
      pagination: {
        page: $page
      }
      filters: $filters
      orderBy: $orderBy
    ) {
      nodes {
        id
        createdAt
        updatedAt
        displayName
        category
        sourceUrl
        enabled
      }
      paginationInfo {
        total
        pages
      }
    }
  }
`;

export type SubscriptionDto =
  GetSubscriptionsQuery['subscriptions']['nodes'][number];

export const UPDATE_SUBSCRIPTIONS = gql`
    mutation UpdateSubscriptions(
    $data: SubscriptionsUpdateInput!,
    $filters: SubscriptionsFilterInput!,
    ) {
    subscriptionsUpdate (
        data: $data
        filter: $filters
    ) {
        id
        createdAt
        updatedAt
        displayName
        category
        sourceUrl
        enabled
    }
}
`;

export const DELETE_SUBSCRIPTIONS = gql`
    mutation DeleteSubscriptions($filters: SubscriptionsFilterInput) {
        subscriptionsDelete(filter: $filters)
    }
`;

export const GET_SUBSCRIPTION_DETAIL = gql`
query GetSubscriptionDetail ($id: Int!) {
  subscriptions(filters: { id: {
    eq: $id
  } }) {
    nodes {
      id
      displayName
      createdAt
      updatedAt
      category
      sourceUrl
      enabled
      bangumi {
        nodes {
          createdAt
          updatedAt
          id
          mikanBangumiId
          displayName
          rawName
          season
          seasonRaw
          fansub
          mikanFansubId
          rssLink
          posterLink
          savePath
          deleted
          homepage
        }
      }
    }
  }
}
`;

export type SubscriptionDetailDto =
  GetSubscriptionDetailQuery['subscriptions']['nodes'][number];

export type SubscriptionDetailBangumiDto =
  SubscriptionDetailDto['bangumi']['nodes'][number];
