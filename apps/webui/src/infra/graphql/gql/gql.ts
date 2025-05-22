/* eslint-disable */
import * as types from './graphql';
import { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';

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
    "\n  query GetSubscriptions(\n  $page: PageInput!,\n  $filters: SubscriptionsFilterInput!,\n  $orderBy: SubscriptionsOrderInput!\n) {\n    subscriptions(\n      pagination: {\n        page: $page\n      }\n      filters: $filters\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n": typeof types.GetSubscriptionsDocument,
    "\n    mutation UpdateSubscriptions(\n    $data: SubscriptionsUpdateInput!,\n    $filters: SubscriptionsFilterInput!,\n    ) {\n    subscriptionsUpdate (\n        data: $data\n        filter: $filters\n    ) {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n    }\n}\n": typeof types.UpdateSubscriptionsDocument,
    "\n    mutation DeleteSubscriptions($filters: SubscriptionsFilterInput) {\n        subscriptionsDelete(filter: $filters)\n    }\n": typeof types.DeleteSubscriptionsDocument,
    "\nquery GetSubscriptionDetail ($id: Int!) {\n  subscriptions(filters: { id: {\n    eq: $id\n  } }) {\n    nodes {\n      id\n      displayName\n      createdAt\n      updatedAt\n      category\n      sourceUrl\n      enabled\n      bangumi {\n        nodes {\n          createdAt\n          updatedAt\n          id\n          mikanBangumiId\n          displayName\n          rawName\n          season\n          seasonRaw\n          fansub\n          mikanFansubId\n          rssLink\n          posterLink\n          savePath\n          homepage\n        }\n      }\n    }\n  }\n}\n": typeof types.GetSubscriptionDetailDocument,
    "\n    mutation CreateSubscription($input: SubscriptionsInsertInput!) {\n        subscriptionsCreateOne(data: $input) {\n            id\n            displayName\n            sourceUrl\n            enabled\n            category\n        }\n    }\n": typeof types.CreateSubscriptionDocument,
};
const documents: Documents = {
    "\n  query GetSubscriptions(\n  $page: PageInput!,\n  $filters: SubscriptionsFilterInput!,\n  $orderBy: SubscriptionsOrderInput!\n) {\n    subscriptions(\n      pagination: {\n        page: $page\n      }\n      filters: $filters\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n": types.GetSubscriptionsDocument,
    "\n    mutation UpdateSubscriptions(\n    $data: SubscriptionsUpdateInput!,\n    $filters: SubscriptionsFilterInput!,\n    ) {\n    subscriptionsUpdate (\n        data: $data\n        filter: $filters\n    ) {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n    }\n}\n": types.UpdateSubscriptionsDocument,
    "\n    mutation DeleteSubscriptions($filters: SubscriptionsFilterInput) {\n        subscriptionsDelete(filter: $filters)\n    }\n": types.DeleteSubscriptionsDocument,
    "\nquery GetSubscriptionDetail ($id: Int!) {\n  subscriptions(filters: { id: {\n    eq: $id\n  } }) {\n    nodes {\n      id\n      displayName\n      createdAt\n      updatedAt\n      category\n      sourceUrl\n      enabled\n      bangumi {\n        nodes {\n          createdAt\n          updatedAt\n          id\n          mikanBangumiId\n          displayName\n          rawName\n          season\n          seasonRaw\n          fansub\n          mikanFansubId\n          rssLink\n          posterLink\n          savePath\n          homepage\n        }\n      }\n    }\n  }\n}\n": types.GetSubscriptionDetailDocument,
    "\n    mutation CreateSubscription($input: SubscriptionsInsertInput!) {\n        subscriptionsCreateOne(data: $input) {\n            id\n            displayName\n            sourceUrl\n            enabled\n            category\n        }\n    }\n": types.CreateSubscriptionDocument,
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
export function gql(source: "\n  query GetSubscriptions(\n  $page: PageInput!,\n  $filters: SubscriptionsFilterInput!,\n  $orderBy: SubscriptionsOrderInput!\n) {\n    subscriptions(\n      pagination: {\n        page: $page\n      }\n      filters: $filters\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n"): (typeof documents)["\n  query GetSubscriptions(\n  $page: PageInput!,\n  $filters: SubscriptionsFilterInput!,\n  $orderBy: SubscriptionsOrderInput!\n) {\n    subscriptions(\n      pagination: {\n        page: $page\n      }\n      filters: $filters\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n    mutation UpdateSubscriptions(\n    $data: SubscriptionsUpdateInput!,\n    $filters: SubscriptionsFilterInput!,\n    ) {\n    subscriptionsUpdate (\n        data: $data\n        filter: $filters\n    ) {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n    }\n}\n"): (typeof documents)["\n    mutation UpdateSubscriptions(\n    $data: SubscriptionsUpdateInput!,\n    $filters: SubscriptionsFilterInput!,\n    ) {\n    subscriptionsUpdate (\n        data: $data\n        filter: $filters\n    ) {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n    }\n}\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n    mutation DeleteSubscriptions($filters: SubscriptionsFilterInput) {\n        subscriptionsDelete(filter: $filters)\n    }\n"): (typeof documents)["\n    mutation DeleteSubscriptions($filters: SubscriptionsFilterInput) {\n        subscriptionsDelete(filter: $filters)\n    }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\nquery GetSubscriptionDetail ($id: Int!) {\n  subscriptions(filters: { id: {\n    eq: $id\n  } }) {\n    nodes {\n      id\n      displayName\n      createdAt\n      updatedAt\n      category\n      sourceUrl\n      enabled\n      bangumi {\n        nodes {\n          createdAt\n          updatedAt\n          id\n          mikanBangumiId\n          displayName\n          rawName\n          season\n          seasonRaw\n          fansub\n          mikanFansubId\n          rssLink\n          posterLink\n          savePath\n          homepage\n        }\n      }\n    }\n  }\n}\n"): (typeof documents)["\nquery GetSubscriptionDetail ($id: Int!) {\n  subscriptions(filters: { id: {\n    eq: $id\n  } }) {\n    nodes {\n      id\n      displayName\n      createdAt\n      updatedAt\n      category\n      sourceUrl\n      enabled\n      bangumi {\n        nodes {\n          createdAt\n          updatedAt\n          id\n          mikanBangumiId\n          displayName\n          rawName\n          season\n          seasonRaw\n          fansub\n          mikanFansubId\n          rssLink\n          posterLink\n          savePath\n          homepage\n        }\n      }\n    }\n  }\n}\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n    mutation CreateSubscription($input: SubscriptionsInsertInput!) {\n        subscriptionsCreateOne(data: $input) {\n            id\n            displayName\n            sourceUrl\n            enabled\n            category\n        }\n    }\n"): (typeof documents)["\n    mutation CreateSubscription($input: SubscriptionsInsertInput!) {\n        subscriptionsCreateOne(data: $input) {\n            id\n            displayName\n            sourceUrl\n            enabled\n            category\n        }\n    }\n"];

export function gql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> = TDocumentNode extends DocumentNode<  infer TType,  any>  ? TType  : never;