import { arkValidatorToTypeNarrower } from '@/infra/errors/arktype';
import {
  type GetSubscriptionsQuery,
  SubscriptionCategoryEnum,
} from '@/infra/graphql/gql/graphql';
import { gql } from '@apollo/client';
import { type } from 'arktype';
import {
  MikanSubscriptionSeasonSourceUrlSchema,
  extractMikanSubscriptionBangumiSourceUrl,
  extractMikanSubscriptionSeasonSourceUrl,
  extractMikanSubscriptionSubscriberSourceUrl,
} from './mikan';

export const GET_SUBSCRIPTIONS = gql`
  query GetSubscriptions($filters: SubscriptionsFilterInput!, $orderBy: SubscriptionsOrderInput!, $pagination: PaginationInput!) {
    subscriptions(
      pagination: $pagination
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
        credentialId
      }
      paginationInfo {
        total
        pages
      }
    }
  }
`;

export const INSERT_SUBSCRIPTION = gql`
    mutation InsertSubscription($data: SubscriptionsInsertInput!) {
        subscriptionsCreateOne(data: $data) {
            id
            createdAt
            updatedAt
            displayName
            category
            sourceUrl
            enabled
            credentialId
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
      credential3rd {
         id
      }
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
          homepage
        }
      }
    }
  }
}
`;

export const SubscriptionTypedMikanSeasonSchema =
  MikanSubscriptionSeasonSourceUrlSchema.and(
    type({
      credentialId: 'number>0',
    })
  );

export const SubscriptionTypedMikanBangumiSchema = type({
  category: `'${SubscriptionCategoryEnum.MikanBangumi}'`,
  sourceUrl: type.string
    .atLeastLength(1)
    .narrow(
      arkValidatorToTypeNarrower(extractMikanSubscriptionBangumiSourceUrl)
    ),
});

export const SubscriptionTypedMikanSubscriberSchema = type({
  category: `'${SubscriptionCategoryEnum.MikanSubscriber}'`,
  sourceUrl: type.string
    .atLeastLength(1)
    .narrow(
      arkValidatorToTypeNarrower(extractMikanSubscriptionSubscriberSourceUrl)
    ),
});

export const SubscriptionTypedSchema = SubscriptionTypedMikanSeasonSchema.or(
  SubscriptionTypedMikanBangumiSchema
).or(SubscriptionTypedMikanSubscriberSchema);

export const SubscriptionInsertFormSchema = type({
  enabled: 'boolean',
  displayName: 'string>0',
}).and(SubscriptionTypedSchema);

export type SubscriptionInsertForm = typeof SubscriptionInsertFormSchema.infer;
