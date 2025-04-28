import type { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';
/* eslint-disable */
import * as types from './graphql';

/**
 * Map of all GraphQL operations in the project.
 *
 * This map has several performance disadvantages:
 * 1. It is not tree-shakeable, so it will include all operations in the project.
 * 2. It is not minifiable, so the string of a GraphQL query will be multiple times inside the bundle.
 * 3. It does not support dead code elimination, so it will add unused operations.
 *
 * Therefore it is highly recommended to use the babel or swc plugin for production.
 * Learn more about it here: https://the-guild.dev/graphql/codegen/plugins/presets/preset-client#reducing-bundle-size
 */
type Documents = {
  '\n    mutation CreateSubscription($input: SubscriptionsInsertInput!) {\n        subscriptionsCreateOne(data: $input) {\n            id\n            displayName\n            sourceUrl\n            enabled\n            category\n            subscriberId\n        }\n    }\n': typeof types.CreateSubscriptionDocument;
  '\n  query GetSubscriptions($page: Int!, $pageSize: Int!) {\n    subscriptions(\n      pagination: {\n        page: {\n          page: $page,\n          limit: $pageSize\n        }\n      }\n    ) {\n      nodes {\n        id\n        displayName\n        category\n        enabled\n        bangumi {\n          nodes {\n            id\n            displayName\n            posterLink\n            season\n            fansub\n            homepage\n          }\n        }\n      }\n    }\n  }\n': typeof types.GetSubscriptionsDocument;
};
const documents: Documents = {
  '\n    mutation CreateSubscription($input: SubscriptionsInsertInput!) {\n        subscriptionsCreateOne(data: $input) {\n            id\n            displayName\n            sourceUrl\n            enabled\n            category\n            subscriberId\n        }\n    }\n':
    types.CreateSubscriptionDocument,
  '\n  query GetSubscriptions($page: Int!, $pageSize: Int!) {\n    subscriptions(\n      pagination: {\n        page: {\n          page: $page,\n          limit: $pageSize\n        }\n      }\n    ) {\n      nodes {\n        id\n        displayName\n        category\n        enabled\n        bangumi {\n          nodes {\n            id\n            displayName\n            posterLink\n            season\n            fansub\n            homepage\n          }\n        }\n      }\n    }\n  }\n':
    types.GetSubscriptionsDocument,
};

/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 *
 *
 * @example
 * ```ts
 * const query = gql(`query GetUser($id: ID!) { user(id: $id) { name } }`);
 * ```
 *
 * The query argument is unknown!
 * Please regenerate the types.
 */
export function gql(source: string): unknown;

/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: '\n    mutation CreateSubscription($input: SubscriptionsInsertInput!) {\n        subscriptionsCreateOne(data: $input) {\n            id\n            displayName\n            sourceUrl\n            enabled\n            category\n            subscriberId\n        }\n    }\n'
): (typeof documents)['\n    mutation CreateSubscription($input: SubscriptionsInsertInput!) {\n        subscriptionsCreateOne(data: $input) {\n            id\n            displayName\n            sourceUrl\n            enabled\n            category\n            subscriberId\n        }\n    }\n'];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: '\n  query GetSubscriptions($page: Int!, $pageSize: Int!) {\n    subscriptions(\n      pagination: {\n        page: {\n          page: $page,\n          limit: $pageSize\n        }\n      }\n    ) {\n      nodes {\n        id\n        displayName\n        category\n        enabled\n        bangumi {\n          nodes {\n            id\n            displayName\n            posterLink\n            season\n            fansub\n            homepage\n          }\n        }\n      }\n    }\n  }\n'
): (typeof documents)['\n  query GetSubscriptions($page: Int!, $pageSize: Int!) {\n    subscriptions(\n      pagination: {\n        page: {\n          page: $page,\n          limit: $pageSize\n        }\n      }\n    ) {\n      nodes {\n        id\n        displayName\n        category\n        enabled\n        bangumi {\n          nodes {\n            id\n            displayName\n            posterLink\n            season\n            fansub\n            homepage\n          }\n        }\n      }\n    }\n  }\n'];

export function gql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> =
  TDocumentNode extends DocumentNode<infer TType, any> ? TType : never;
