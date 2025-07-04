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
    "\n  query GetCredential3rd($filters: Credential3rdFilterInput!, $orderBy: Credential3rdOrderInput, $pagination: PaginationInput) {\n    credential3rd(filters: $filters, orderBy: $orderBy, pagination: $pagination) {\n      nodes {\n        id\n        cookies\n        username\n        password\n        userAgent\n        createdAt\n        updatedAt\n        credentialType\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n": typeof types.GetCredential3rdDocument,
    "\n  mutation InsertCredential3rd($data: Credential3rdInsertInput!) {\n    credential3rdCreateOne(data: $data) {\n      id\n      cookies\n      username\n      password\n      userAgent\n      createdAt\n      updatedAt\n      credentialType\n    }\n  }\n": typeof types.InsertCredential3rdDocument,
    "\n  mutation UpdateCredential3rd($data: Credential3rdUpdateInput!, $filters: Credential3rdFilterInput!) {\n    credential3rdUpdate(data: $data, filter: $filters) {\n      id\n      cookies\n      username\n      password\n      userAgent\n      createdAt\n      updatedAt\n      credentialType\n    }\n  }\n": typeof types.UpdateCredential3rdDocument,
    "\n  mutation DeleteCredential3rd($filters: Credential3rdFilterInput!) {\n    credential3rdDelete(filter: $filters)\n  }\n": typeof types.DeleteCredential3rdDocument,
    "\n  query GetCredential3rdDetail($id: Int!) {\n    credential3rd(filters: { id: { eq: $id } }) {\n      nodes {\n        id\n        cookies\n        username\n        password\n        userAgent\n        createdAt\n        updatedAt\n        credentialType\n      }\n    }\n  }\n": typeof types.GetCredential3rdDetailDocument,
    "\n  query CheckCredential3rdAvailable($id: Int!) {\n    credential3rdCheckAvailable(filter: { id: $id }) {\n       available\n    }\n  }\n": typeof types.CheckCredential3rdAvailableDocument,
    "\n    mutation InsertFeed($data: FeedsInsertInput!) {\n        feedsCreateOne(data: $data) {\n            id\n            createdAt\n            updatedAt\n            feedType\n            token\n        }\n    }\n": typeof types.InsertFeedDocument,
    "\n    mutation DeleteFeed($filters: FeedsFilterInput!) {\n        feedsDelete(filter: $filters)\n    }\n": typeof types.DeleteFeedDocument,
    "\n  query GetSubscriptions($filters: SubscriptionsFilterInput!, $orderBy: SubscriptionsOrderInput!, $pagination: PaginationInput!) {\n    subscriptions(\n      pagination: $pagination\n      filters: $filters\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n        credentialId\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n": typeof types.GetSubscriptionsDocument,
    "\n    mutation InsertSubscription($data: SubscriptionsInsertInput!) {\n        subscriptionsCreateOne(data: $data) {\n            id\n            createdAt\n            updatedAt\n            displayName\n            category\n            sourceUrl\n            enabled\n            credentialId\n        }\n    }\n": typeof types.InsertSubscriptionDocument,
    "\n    mutation UpdateSubscriptions(\n    $data: SubscriptionsUpdateInput!,\n    $filters: SubscriptionsFilterInput!,\n    ) {\n    subscriptionsUpdate (\n        data: $data\n        filter: $filters\n    ) {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n    }\n}\n": typeof types.UpdateSubscriptionsDocument,
    "\n    mutation DeleteSubscriptions($filters: SubscriptionsFilterInput) {\n        subscriptionsDelete(filter: $filters)\n    }\n": typeof types.DeleteSubscriptionsDocument,
    "\nquery GetSubscriptionDetail ($id: Int!) {\n  subscriptions(filters: { id: {\n    eq: $id\n  } }) {\n    nodes {\n      id\n      displayName\n      createdAt\n      updatedAt\n      category\n      sourceUrl\n      enabled\n      feed {\n        nodes {\n           id\n           createdAt\n           updatedAt\n           token\n           feedType\n           feedSource\n        }\n      }\n      credential3rd {\n         id\n         username\n      }\n      bangumi {\n        nodes {\n          createdAt\n          updatedAt\n          id\n          mikanBangumiId\n          displayName\n          season\n          seasonRaw\n          fansub\n          mikanFansubId\n          rssLink\n          posterLink\n          homepage\n        }\n      }\n    }\n  }\n}\n": typeof types.GetSubscriptionDetailDocument,
    "\n  mutation SyncSubscriptionFeedsIncremental($filter: SubscriptionsFilterInput!) {\n    subscriptionsSyncOneFeedsIncremental(filter: $filter) {\n      id\n    }\n  }\n": typeof types.SyncSubscriptionFeedsIncrementalDocument,
    "\n  mutation SyncSubscriptionFeedsFull($filter: SubscriptionsFilterInput!) {\n    subscriptionsSyncOneFeedsFull(filter: $filter) {\n      id\n    }\n  }\n": typeof types.SyncSubscriptionFeedsFullDocument,
    "\n  mutation SyncSubscriptionSources($filter: SubscriptionsFilterInput!) {\n    subscriptionsSyncOneSources(filter: $filter) {\n      id\n    }\n  }\n": typeof types.SyncSubscriptionSourcesDocument,
    "\n  query GetTasks($filters: SubscriberTasksFilterInput!, $orderBy: SubscriberTasksOrderInput!, $pagination: PaginationInput!) {\n    subscriberTasks(\n      pagination: $pagination\n      filters: $filters\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id,\n        job,\n        taskType,\n        status,\n        attempts,\n        maxAttempts,\n        runAt,\n        lastError,\n        lockAt,\n        lockBy,\n        doneAt,\n        priority\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n": typeof types.GetTasksDocument,
    "\n  mutation DeleteTasks($filters: SubscriberTasksFilterInput!) {\n    subscriberTasksDelete(filter: $filters)\n  }\n": typeof types.DeleteTasksDocument,
    "\n  mutation RetryTasks($filters: SubscriberTasksFilterInput!) {\n    subscriberTasksRetryOne(filter: $filters) {\n        id,\n        job,\n        taskType,\n        status,\n        attempts,\n        maxAttempts,\n        runAt,\n        lastError,\n        lockAt,\n        lockBy,\n        doneAt,\n        priority\n    }\n  }\n": typeof types.RetryTasksDocument,
};
const documents: Documents = {
    "\n  query GetCredential3rd($filters: Credential3rdFilterInput!, $orderBy: Credential3rdOrderInput, $pagination: PaginationInput) {\n    credential3rd(filters: $filters, orderBy: $orderBy, pagination: $pagination) {\n      nodes {\n        id\n        cookies\n        username\n        password\n        userAgent\n        createdAt\n        updatedAt\n        credentialType\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n": types.GetCredential3rdDocument,
    "\n  mutation InsertCredential3rd($data: Credential3rdInsertInput!) {\n    credential3rdCreateOne(data: $data) {\n      id\n      cookies\n      username\n      password\n      userAgent\n      createdAt\n      updatedAt\n      credentialType\n    }\n  }\n": types.InsertCredential3rdDocument,
    "\n  mutation UpdateCredential3rd($data: Credential3rdUpdateInput!, $filters: Credential3rdFilterInput!) {\n    credential3rdUpdate(data: $data, filter: $filters) {\n      id\n      cookies\n      username\n      password\n      userAgent\n      createdAt\n      updatedAt\n      credentialType\n    }\n  }\n": types.UpdateCredential3rdDocument,
    "\n  mutation DeleteCredential3rd($filters: Credential3rdFilterInput!) {\n    credential3rdDelete(filter: $filters)\n  }\n": types.DeleteCredential3rdDocument,
    "\n  query GetCredential3rdDetail($id: Int!) {\n    credential3rd(filters: { id: { eq: $id } }) {\n      nodes {\n        id\n        cookies\n        username\n        password\n        userAgent\n        createdAt\n        updatedAt\n        credentialType\n      }\n    }\n  }\n": types.GetCredential3rdDetailDocument,
    "\n  query CheckCredential3rdAvailable($id: Int!) {\n    credential3rdCheckAvailable(filter: { id: $id }) {\n       available\n    }\n  }\n": types.CheckCredential3rdAvailableDocument,
    "\n    mutation InsertFeed($data: FeedsInsertInput!) {\n        feedsCreateOne(data: $data) {\n            id\n            createdAt\n            updatedAt\n            feedType\n            token\n        }\n    }\n": types.InsertFeedDocument,
    "\n    mutation DeleteFeed($filters: FeedsFilterInput!) {\n        feedsDelete(filter: $filters)\n    }\n": types.DeleteFeedDocument,
    "\n  query GetSubscriptions($filters: SubscriptionsFilterInput!, $orderBy: SubscriptionsOrderInput!, $pagination: PaginationInput!) {\n    subscriptions(\n      pagination: $pagination\n      filters: $filters\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n        credentialId\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n": types.GetSubscriptionsDocument,
    "\n    mutation InsertSubscription($data: SubscriptionsInsertInput!) {\n        subscriptionsCreateOne(data: $data) {\n            id\n            createdAt\n            updatedAt\n            displayName\n            category\n            sourceUrl\n            enabled\n            credentialId\n        }\n    }\n": types.InsertSubscriptionDocument,
    "\n    mutation UpdateSubscriptions(\n    $data: SubscriptionsUpdateInput!,\n    $filters: SubscriptionsFilterInput!,\n    ) {\n    subscriptionsUpdate (\n        data: $data\n        filter: $filters\n    ) {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n    }\n}\n": types.UpdateSubscriptionsDocument,
    "\n    mutation DeleteSubscriptions($filters: SubscriptionsFilterInput) {\n        subscriptionsDelete(filter: $filters)\n    }\n": types.DeleteSubscriptionsDocument,
    "\nquery GetSubscriptionDetail ($id: Int!) {\n  subscriptions(filters: { id: {\n    eq: $id\n  } }) {\n    nodes {\n      id\n      displayName\n      createdAt\n      updatedAt\n      category\n      sourceUrl\n      enabled\n      feed {\n        nodes {\n           id\n           createdAt\n           updatedAt\n           token\n           feedType\n           feedSource\n        }\n      }\n      credential3rd {\n         id\n         username\n      }\n      bangumi {\n        nodes {\n          createdAt\n          updatedAt\n          id\n          mikanBangumiId\n          displayName\n          season\n          seasonRaw\n          fansub\n          mikanFansubId\n          rssLink\n          posterLink\n          homepage\n        }\n      }\n    }\n  }\n}\n": types.GetSubscriptionDetailDocument,
    "\n  mutation SyncSubscriptionFeedsIncremental($filter: SubscriptionsFilterInput!) {\n    subscriptionsSyncOneFeedsIncremental(filter: $filter) {\n      id\n    }\n  }\n": types.SyncSubscriptionFeedsIncrementalDocument,
    "\n  mutation SyncSubscriptionFeedsFull($filter: SubscriptionsFilterInput!) {\n    subscriptionsSyncOneFeedsFull(filter: $filter) {\n      id\n    }\n  }\n": types.SyncSubscriptionFeedsFullDocument,
    "\n  mutation SyncSubscriptionSources($filter: SubscriptionsFilterInput!) {\n    subscriptionsSyncOneSources(filter: $filter) {\n      id\n    }\n  }\n": types.SyncSubscriptionSourcesDocument,
    "\n  query GetTasks($filters: SubscriberTasksFilterInput!, $orderBy: SubscriberTasksOrderInput!, $pagination: PaginationInput!) {\n    subscriberTasks(\n      pagination: $pagination\n      filters: $filters\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id,\n        job,\n        taskType,\n        status,\n        attempts,\n        maxAttempts,\n        runAt,\n        lastError,\n        lockAt,\n        lockBy,\n        doneAt,\n        priority\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n": types.GetTasksDocument,
    "\n  mutation DeleteTasks($filters: SubscriberTasksFilterInput!) {\n    subscriberTasksDelete(filter: $filters)\n  }\n": types.DeleteTasksDocument,
    "\n  mutation RetryTasks($filters: SubscriberTasksFilterInput!) {\n    subscriberTasksRetryOne(filter: $filters) {\n        id,\n        job,\n        taskType,\n        status,\n        attempts,\n        maxAttempts,\n        runAt,\n        lastError,\n        lockAt,\n        lockBy,\n        doneAt,\n        priority\n    }\n  }\n": types.RetryTasksDocument,
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
export function gql(source: "\n  query GetCredential3rd($filters: Credential3rdFilterInput!, $orderBy: Credential3rdOrderInput, $pagination: PaginationInput) {\n    credential3rd(filters: $filters, orderBy: $orderBy, pagination: $pagination) {\n      nodes {\n        id\n        cookies\n        username\n        password\n        userAgent\n        createdAt\n        updatedAt\n        credentialType\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n"): (typeof documents)["\n  query GetCredential3rd($filters: Credential3rdFilterInput!, $orderBy: Credential3rdOrderInput, $pagination: PaginationInput) {\n    credential3rd(filters: $filters, orderBy: $orderBy, pagination: $pagination) {\n      nodes {\n        id\n        cookies\n        username\n        password\n        userAgent\n        createdAt\n        updatedAt\n        credentialType\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  mutation InsertCredential3rd($data: Credential3rdInsertInput!) {\n    credential3rdCreateOne(data: $data) {\n      id\n      cookies\n      username\n      password\n      userAgent\n      createdAt\n      updatedAt\n      credentialType\n    }\n  }\n"): (typeof documents)["\n  mutation InsertCredential3rd($data: Credential3rdInsertInput!) {\n    credential3rdCreateOne(data: $data) {\n      id\n      cookies\n      username\n      password\n      userAgent\n      createdAt\n      updatedAt\n      credentialType\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  mutation UpdateCredential3rd($data: Credential3rdUpdateInput!, $filters: Credential3rdFilterInput!) {\n    credential3rdUpdate(data: $data, filter: $filters) {\n      id\n      cookies\n      username\n      password\n      userAgent\n      createdAt\n      updatedAt\n      credentialType\n    }\n  }\n"): (typeof documents)["\n  mutation UpdateCredential3rd($data: Credential3rdUpdateInput!, $filters: Credential3rdFilterInput!) {\n    credential3rdUpdate(data: $data, filter: $filters) {\n      id\n      cookies\n      username\n      password\n      userAgent\n      createdAt\n      updatedAt\n      credentialType\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  mutation DeleteCredential3rd($filters: Credential3rdFilterInput!) {\n    credential3rdDelete(filter: $filters)\n  }\n"): (typeof documents)["\n  mutation DeleteCredential3rd($filters: Credential3rdFilterInput!) {\n    credential3rdDelete(filter: $filters)\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  query GetCredential3rdDetail($id: Int!) {\n    credential3rd(filters: { id: { eq: $id } }) {\n      nodes {\n        id\n        cookies\n        username\n        password\n        userAgent\n        createdAt\n        updatedAt\n        credentialType\n      }\n    }\n  }\n"): (typeof documents)["\n  query GetCredential3rdDetail($id: Int!) {\n    credential3rd(filters: { id: { eq: $id } }) {\n      nodes {\n        id\n        cookies\n        username\n        password\n        userAgent\n        createdAt\n        updatedAt\n        credentialType\n      }\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  query CheckCredential3rdAvailable($id: Int!) {\n    credential3rdCheckAvailable(filter: { id: $id }) {\n       available\n    }\n  }\n"): (typeof documents)["\n  query CheckCredential3rdAvailable($id: Int!) {\n    credential3rdCheckAvailable(filter: { id: $id }) {\n       available\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n    mutation InsertFeed($data: FeedsInsertInput!) {\n        feedsCreateOne(data: $data) {\n            id\n            createdAt\n            updatedAt\n            feedType\n            token\n        }\n    }\n"): (typeof documents)["\n    mutation InsertFeed($data: FeedsInsertInput!) {\n        feedsCreateOne(data: $data) {\n            id\n            createdAt\n            updatedAt\n            feedType\n            token\n        }\n    }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n    mutation DeleteFeed($filters: FeedsFilterInput!) {\n        feedsDelete(filter: $filters)\n    }\n"): (typeof documents)["\n    mutation DeleteFeed($filters: FeedsFilterInput!) {\n        feedsDelete(filter: $filters)\n    }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  query GetSubscriptions($filters: SubscriptionsFilterInput!, $orderBy: SubscriptionsOrderInput!, $pagination: PaginationInput!) {\n    subscriptions(\n      pagination: $pagination\n      filters: $filters\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n        credentialId\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n"): (typeof documents)["\n  query GetSubscriptions($filters: SubscriptionsFilterInput!, $orderBy: SubscriptionsOrderInput!, $pagination: PaginationInput!) {\n    subscriptions(\n      pagination: $pagination\n      filters: $filters\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n        credentialId\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n    mutation InsertSubscription($data: SubscriptionsInsertInput!) {\n        subscriptionsCreateOne(data: $data) {\n            id\n            createdAt\n            updatedAt\n            displayName\n            category\n            sourceUrl\n            enabled\n            credentialId\n        }\n    }\n"): (typeof documents)["\n    mutation InsertSubscription($data: SubscriptionsInsertInput!) {\n        subscriptionsCreateOne(data: $data) {\n            id\n            createdAt\n            updatedAt\n            displayName\n            category\n            sourceUrl\n            enabled\n            credentialId\n        }\n    }\n"];
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
export function gql(source: "\nquery GetSubscriptionDetail ($id: Int!) {\n  subscriptions(filters: { id: {\n    eq: $id\n  } }) {\n    nodes {\n      id\n      displayName\n      createdAt\n      updatedAt\n      category\n      sourceUrl\n      enabled\n      feed {\n        nodes {\n           id\n           createdAt\n           updatedAt\n           token\n           feedType\n           feedSource\n        }\n      }\n      credential3rd {\n         id\n         username\n      }\n      bangumi {\n        nodes {\n          createdAt\n          updatedAt\n          id\n          mikanBangumiId\n          displayName\n          season\n          seasonRaw\n          fansub\n          mikanFansubId\n          rssLink\n          posterLink\n          homepage\n        }\n      }\n    }\n  }\n}\n"): (typeof documents)["\nquery GetSubscriptionDetail ($id: Int!) {\n  subscriptions(filters: { id: {\n    eq: $id\n  } }) {\n    nodes {\n      id\n      displayName\n      createdAt\n      updatedAt\n      category\n      sourceUrl\n      enabled\n      feed {\n        nodes {\n           id\n           createdAt\n           updatedAt\n           token\n           feedType\n           feedSource\n        }\n      }\n      credential3rd {\n         id\n         username\n      }\n      bangumi {\n        nodes {\n          createdAt\n          updatedAt\n          id\n          mikanBangumiId\n          displayName\n          season\n          seasonRaw\n          fansub\n          mikanFansubId\n          rssLink\n          posterLink\n          homepage\n        }\n      }\n    }\n  }\n}\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  mutation SyncSubscriptionFeedsIncremental($filter: SubscriptionsFilterInput!) {\n    subscriptionsSyncOneFeedsIncremental(filter: $filter) {\n      id\n    }\n  }\n"): (typeof documents)["\n  mutation SyncSubscriptionFeedsIncremental($filter: SubscriptionsFilterInput!) {\n    subscriptionsSyncOneFeedsIncremental(filter: $filter) {\n      id\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  mutation SyncSubscriptionFeedsFull($filter: SubscriptionsFilterInput!) {\n    subscriptionsSyncOneFeedsFull(filter: $filter) {\n      id\n    }\n  }\n"): (typeof documents)["\n  mutation SyncSubscriptionFeedsFull($filter: SubscriptionsFilterInput!) {\n    subscriptionsSyncOneFeedsFull(filter: $filter) {\n      id\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  mutation SyncSubscriptionSources($filter: SubscriptionsFilterInput!) {\n    subscriptionsSyncOneSources(filter: $filter) {\n      id\n    }\n  }\n"): (typeof documents)["\n  mutation SyncSubscriptionSources($filter: SubscriptionsFilterInput!) {\n    subscriptionsSyncOneSources(filter: $filter) {\n      id\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  query GetTasks($filters: SubscriberTasksFilterInput!, $orderBy: SubscriberTasksOrderInput!, $pagination: PaginationInput!) {\n    subscriberTasks(\n      pagination: $pagination\n      filters: $filters\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id,\n        job,\n        taskType,\n        status,\n        attempts,\n        maxAttempts,\n        runAt,\n        lastError,\n        lockAt,\n        lockBy,\n        doneAt,\n        priority\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n"): (typeof documents)["\n  query GetTasks($filters: SubscriberTasksFilterInput!, $orderBy: SubscriberTasksOrderInput!, $pagination: PaginationInput!) {\n    subscriberTasks(\n      pagination: $pagination\n      filters: $filters\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id,\n        job,\n        taskType,\n        status,\n        attempts,\n        maxAttempts,\n        runAt,\n        lastError,\n        lockAt,\n        lockBy,\n        doneAt,\n        priority\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  mutation DeleteTasks($filters: SubscriberTasksFilterInput!) {\n    subscriberTasksDelete(filter: $filters)\n  }\n"): (typeof documents)["\n  mutation DeleteTasks($filters: SubscriberTasksFilterInput!) {\n    subscriberTasksDelete(filter: $filters)\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(source: "\n  mutation RetryTasks($filters: SubscriberTasksFilterInput!) {\n    subscriberTasksRetryOne(filter: $filters) {\n        id,\n        job,\n        taskType,\n        status,\n        attempts,\n        maxAttempts,\n        runAt,\n        lastError,\n        lockAt,\n        lockBy,\n        doneAt,\n        priority\n    }\n  }\n"): (typeof documents)["\n  mutation RetryTasks($filters: SubscriberTasksFilterInput!) {\n    subscriberTasksRetryOne(filter: $filters) {\n        id,\n        job,\n        taskType,\n        status,\n        attempts,\n        maxAttempts,\n        runAt,\n        lastError,\n        lockAt,\n        lockBy,\n        doneAt,\n        priority\n    }\n  }\n"];

export function gql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> = TDocumentNode extends DocumentNode<  infer TType,  any>  ? TType  : never;