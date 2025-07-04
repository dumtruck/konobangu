/* eslint-disable */
import { TypedDocumentNode as DocumentNode } from '@graphql-typed-document-node/core';
export type Maybe<T> = T | null;
export type InputMaybe<T> = Maybe<T>;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
export type MakeEmpty<T extends { [key: string]: unknown }, K extends keyof T> = { [_ in K]?: never };
export type Incremental<T> = T | { [P in keyof T]?: P extends ' $fragmentName' | '__typename' ? T[P] : never };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string; }
  String: { input: string; output: string; }
  Boolean: { input: boolean; output: boolean; }
  Int: { input: number; output: number; }
  Float: { input: number; output: number; }
  /** The `JSON` scalar type represents raw JSON values */
  Json: { input: any; output: any; }
  JsonbFilterInput: { input: any; output: any; }
};

export type Bangumi = {
  __typename?: 'Bangumi';
  bangumiType: BangumiTypeEnum;
  createdAt: Scalars['String']['output'];
  displayName: Scalars['String']['output'];
  episode: EpisodesConnection;
  fansub?: Maybe<Scalars['String']['output']>;
  filter?: Maybe<Scalars['Json']['output']>;
  homepage?: Maybe<Scalars['String']['output']>;
  id: Scalars['Int']['output'];
  mikanBangumiId?: Maybe<Scalars['String']['output']>;
  mikanFansubId?: Maybe<Scalars['String']['output']>;
  originName: Scalars['String']['output'];
  originPosterLink?: Maybe<Scalars['String']['output']>;
  posterLink?: Maybe<Scalars['String']['output']>;
  rssLink?: Maybe<Scalars['String']['output']>;
  season: Scalars['Int']['output'];
  seasonRaw?: Maybe<Scalars['String']['output']>;
  subscriber?: Maybe<Subscribers>;
  subscriberId: Scalars['Int']['output'];
  subscription: SubscriptionsConnection;
  subscriptionBangumi: SubscriptionBangumiConnection;
  updatedAt: Scalars['String']['output'];
};


export type BangumiEpisodeArgs = {
  filters?: InputMaybe<EpisodesFilterInput>;
  orderBy?: InputMaybe<EpisodesOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type BangumiSubscriptionArgs = {
  filters?: InputMaybe<SubscriptionsFilterInput>;
  orderBy?: InputMaybe<SubscriptionsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type BangumiSubscriptionBangumiArgs = {
  filters?: InputMaybe<SubscriptionBangumiFilterInput>;
  orderBy?: InputMaybe<SubscriptionBangumiOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type BangumiBasic = {
  __typename?: 'BangumiBasic';
  bangumiType: BangumiTypeEnum;
  createdAt: Scalars['String']['output'];
  displayName: Scalars['String']['output'];
  fansub?: Maybe<Scalars['String']['output']>;
  filter?: Maybe<Scalars['Json']['output']>;
  homepage?: Maybe<Scalars['String']['output']>;
  id: Scalars['Int']['output'];
  mikanBangumiId?: Maybe<Scalars['String']['output']>;
  mikanFansubId?: Maybe<Scalars['String']['output']>;
  originName: Scalars['String']['output'];
  originPosterLink?: Maybe<Scalars['String']['output']>;
  posterLink?: Maybe<Scalars['String']['output']>;
  rssLink?: Maybe<Scalars['String']['output']>;
  season: Scalars['Int']['output'];
  seasonRaw?: Maybe<Scalars['String']['output']>;
  subscriberId: Scalars['Int']['output'];
  updatedAt: Scalars['String']['output'];
};

export type BangumiConnection = {
  __typename?: 'BangumiConnection';
  edges: Array<BangumiEdge>;
  nodes: Array<Bangumi>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type BangumiEdge = {
  __typename?: 'BangumiEdge';
  cursor: Scalars['String']['output'];
  node: Bangumi;
};

export type BangumiFilterInput = {
  and?: InputMaybe<Array<BangumiFilterInput>>;
  bangumiType?: InputMaybe<BangumiTypeEnumFilterInput>;
  createdAt?: InputMaybe<TextFilterInput>;
  displayName?: InputMaybe<StringFilterInput>;
  fansub?: InputMaybe<StringFilterInput>;
  homepage?: InputMaybe<StringFilterInput>;
  id?: InputMaybe<IntegerFilterInput>;
  mikanBangumiId?: InputMaybe<StringFilterInput>;
  mikanFansubId?: InputMaybe<StringFilterInput>;
  or?: InputMaybe<Array<BangumiFilterInput>>;
  originName?: InputMaybe<StringFilterInput>;
  originPosterLink?: InputMaybe<StringFilterInput>;
  posterLink?: InputMaybe<StringFilterInput>;
  rssLink?: InputMaybe<StringFilterInput>;
  season?: InputMaybe<IntegerFilterInput>;
  seasonRaw?: InputMaybe<StringFilterInput>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  updatedAt?: InputMaybe<TextFilterInput>;
};

export type BangumiInsertInput = {
  bangumiType: BangumiTypeEnum;
  createdAt?: InputMaybe<Scalars['String']['input']>;
  displayName: Scalars['String']['input'];
  fansub?: InputMaybe<Scalars['String']['input']>;
  filter?: InputMaybe<Scalars['Json']['input']>;
  homepage?: InputMaybe<Scalars['String']['input']>;
  id?: InputMaybe<Scalars['Int']['input']>;
  mikanBangumiId?: InputMaybe<Scalars['String']['input']>;
  mikanFansubId?: InputMaybe<Scalars['String']['input']>;
  originName: Scalars['String']['input'];
  originPosterLink?: InputMaybe<Scalars['String']['input']>;
  posterLink?: InputMaybe<Scalars['String']['input']>;
  rssLink?: InputMaybe<Scalars['String']['input']>;
  season: Scalars['Int']['input'];
  seasonRaw?: InputMaybe<Scalars['String']['input']>;
  subscriberId?: InputMaybe<Scalars['Int']['input']>;
  updatedAt?: InputMaybe<Scalars['String']['input']>;
};

export type BangumiOrderInput = {
  bangumiType?: InputMaybe<OrderByEnum>;
  createdAt?: InputMaybe<OrderByEnum>;
  displayName?: InputMaybe<OrderByEnum>;
  fansub?: InputMaybe<OrderByEnum>;
  filter?: InputMaybe<OrderByEnum>;
  homepage?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  mikanBangumiId?: InputMaybe<OrderByEnum>;
  mikanFansubId?: InputMaybe<OrderByEnum>;
  originName?: InputMaybe<OrderByEnum>;
  originPosterLink?: InputMaybe<OrderByEnum>;
  posterLink?: InputMaybe<OrderByEnum>;
  rssLink?: InputMaybe<OrderByEnum>;
  season?: InputMaybe<OrderByEnum>;
  seasonRaw?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  updatedAt?: InputMaybe<OrderByEnum>;
};

export const BangumiTypeEnum = {
  Mikan: 'mikan'
} as const;

export type BangumiTypeEnum = typeof BangumiTypeEnum[keyof typeof BangumiTypeEnum];
export type BangumiTypeEnumFilterInput = {
  eq?: InputMaybe<BangumiTypeEnum>;
  gt?: InputMaybe<BangumiTypeEnum>;
  gte?: InputMaybe<BangumiTypeEnum>;
  is_in?: InputMaybe<Array<BangumiTypeEnum>>;
  is_not_in?: InputMaybe<Array<BangumiTypeEnum>>;
  is_not_null?: InputMaybe<BangumiTypeEnum>;
  is_null?: InputMaybe<BangumiTypeEnum>;
  lt?: InputMaybe<BangumiTypeEnum>;
  lte?: InputMaybe<BangumiTypeEnum>;
  ne?: InputMaybe<BangumiTypeEnum>;
};

export type BangumiUpdateInput = {
  bangumiType?: InputMaybe<BangumiTypeEnum>;
  createdAt?: InputMaybe<Scalars['String']['input']>;
  displayName?: InputMaybe<Scalars['String']['input']>;
  fansub?: InputMaybe<Scalars['String']['input']>;
  filter?: InputMaybe<Scalars['Json']['input']>;
  homepage?: InputMaybe<Scalars['String']['input']>;
  id?: InputMaybe<Scalars['Int']['input']>;
  mikanBangumiId?: InputMaybe<Scalars['String']['input']>;
  mikanFansubId?: InputMaybe<Scalars['String']['input']>;
  originName?: InputMaybe<Scalars['String']['input']>;
  originPosterLink?: InputMaybe<Scalars['String']['input']>;
  posterLink?: InputMaybe<Scalars['String']['input']>;
  rssLink?: InputMaybe<Scalars['String']['input']>;
  season?: InputMaybe<Scalars['Int']['input']>;
  seasonRaw?: InputMaybe<Scalars['String']['input']>;
  updatedAt?: InputMaybe<Scalars['String']['input']>;
};

export type BooleanFilterInput = {
  eq?: InputMaybe<Scalars['Boolean']['input']>;
  gt?: InputMaybe<Scalars['Boolean']['input']>;
  gte?: InputMaybe<Scalars['Boolean']['input']>;
  is_in?: InputMaybe<Array<Scalars['Boolean']['input']>>;
  is_not_in?: InputMaybe<Array<Scalars['Boolean']['input']>>;
  is_not_null?: InputMaybe<Scalars['Boolean']['input']>;
  is_null?: InputMaybe<Scalars['Boolean']['input']>;
  lt?: InputMaybe<Scalars['Boolean']['input']>;
  lte?: InputMaybe<Scalars['Boolean']['input']>;
  ne?: InputMaybe<Scalars['Boolean']['input']>;
};

export type Credential3rd = {
  __typename?: 'Credential3rd';
  cookies?: Maybe<Scalars['String']['output']>;
  createdAt: Scalars['String']['output'];
  credentialType: Credential3rdTypeEnum;
  id: Scalars['Int']['output'];
  password?: Maybe<Scalars['String']['output']>;
  subscriber?: Maybe<Subscribers>;
  subscriberId: Scalars['Int']['output'];
  subscription: SubscriptionsConnection;
  updatedAt: Scalars['String']['output'];
  userAgent?: Maybe<Scalars['String']['output']>;
  username?: Maybe<Scalars['String']['output']>;
};


export type Credential3rdSubscriptionArgs = {
  filters?: InputMaybe<SubscriptionsFilterInput>;
  orderBy?: InputMaybe<SubscriptionsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type Credential3rdBasic = {
  __typename?: 'Credential3rdBasic';
  cookies?: Maybe<Scalars['String']['output']>;
  createdAt: Scalars['String']['output'];
  credentialType: Credential3rdTypeEnum;
  id: Scalars['Int']['output'];
  password?: Maybe<Scalars['String']['output']>;
  subscriberId: Scalars['Int']['output'];
  updatedAt: Scalars['String']['output'];
  userAgent?: Maybe<Scalars['String']['output']>;
  username?: Maybe<Scalars['String']['output']>;
};

/** The output of the credential3rdCheckAvailable query */
export type Credential3rdCheckAvailableInfo = {
  __typename?: 'Credential3rdCheckAvailableInfo';
  available: Scalars['Boolean']['output'];
};

/** The input of the credential3rdCheckAvailable query */
export type Credential3rdCheckAvailableInput = {
  id: Scalars['Int']['input'];
};

export type Credential3rdConnection = {
  __typename?: 'Credential3rdConnection';
  edges: Array<Credential3rdEdge>;
  nodes: Array<Credential3rd>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type Credential3rdEdge = {
  __typename?: 'Credential3rdEdge';
  cursor: Scalars['String']['output'];
  node: Credential3rd;
};

export type Credential3rdFilterInput = {
  and?: InputMaybe<Array<Credential3rdFilterInput>>;
  cookies?: InputMaybe<StringFilterInput>;
  createdAt?: InputMaybe<TextFilterInput>;
  credentialType?: InputMaybe<Credential3rdTypeEnumFilterInput>;
  id?: InputMaybe<IntegerFilterInput>;
  or?: InputMaybe<Array<Credential3rdFilterInput>>;
  password?: InputMaybe<StringFilterInput>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  updatedAt?: InputMaybe<TextFilterInput>;
  userAgent?: InputMaybe<StringFilterInput>;
  username?: InputMaybe<StringFilterInput>;
};

export type Credential3rdInsertInput = {
  cookies?: InputMaybe<Scalars['String']['input']>;
  createdAt?: InputMaybe<Scalars['String']['input']>;
  credentialType: Credential3rdTypeEnum;
  id?: InputMaybe<Scalars['Int']['input']>;
  password?: InputMaybe<Scalars['String']['input']>;
  subscriberId?: InputMaybe<Scalars['Int']['input']>;
  updatedAt?: InputMaybe<Scalars['String']['input']>;
  userAgent?: InputMaybe<Scalars['String']['input']>;
  username?: InputMaybe<Scalars['String']['input']>;
};

export type Credential3rdOrderInput = {
  cookies?: InputMaybe<OrderByEnum>;
  createdAt?: InputMaybe<OrderByEnum>;
  credentialType?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  password?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  updatedAt?: InputMaybe<OrderByEnum>;
  userAgent?: InputMaybe<OrderByEnum>;
  username?: InputMaybe<OrderByEnum>;
};

export const Credential3rdTypeEnum = {
  Mikan: 'mikan'
} as const;

export type Credential3rdTypeEnum = typeof Credential3rdTypeEnum[keyof typeof Credential3rdTypeEnum];
export type Credential3rdTypeEnumFilterInput = {
  eq?: InputMaybe<Credential3rdTypeEnum>;
  gt?: InputMaybe<Credential3rdTypeEnum>;
  gte?: InputMaybe<Credential3rdTypeEnum>;
  is_in?: InputMaybe<Array<Credential3rdTypeEnum>>;
  is_not_in?: InputMaybe<Array<Credential3rdTypeEnum>>;
  is_not_null?: InputMaybe<Credential3rdTypeEnum>;
  is_null?: InputMaybe<Credential3rdTypeEnum>;
  lt?: InputMaybe<Credential3rdTypeEnum>;
  lte?: InputMaybe<Credential3rdTypeEnum>;
  ne?: InputMaybe<Credential3rdTypeEnum>;
};

export type Credential3rdUpdateInput = {
  cookies?: InputMaybe<Scalars['String']['input']>;
  createdAt?: InputMaybe<Scalars['String']['input']>;
  credentialType?: InputMaybe<Credential3rdTypeEnum>;
  id?: InputMaybe<Scalars['Int']['input']>;
  password?: InputMaybe<Scalars['String']['input']>;
  updatedAt?: InputMaybe<Scalars['String']['input']>;
  userAgent?: InputMaybe<Scalars['String']['input']>;
  username?: InputMaybe<Scalars['String']['input']>;
};

export type CursorInput = {
  cursor?: InputMaybe<Scalars['String']['input']>;
  limit: Scalars['Int']['input'];
};

export const DownloadMimeEnum = {
  Applicationoctetstream: 'applicationoctetstream',
  Applicationxbittorrent: 'applicationxbittorrent'
} as const;

export type DownloadMimeEnum = typeof DownloadMimeEnum[keyof typeof DownloadMimeEnum];
export type DownloadMimeEnumFilterInput = {
  eq?: InputMaybe<DownloadMimeEnum>;
  gt?: InputMaybe<DownloadMimeEnum>;
  gte?: InputMaybe<DownloadMimeEnum>;
  is_in?: InputMaybe<Array<DownloadMimeEnum>>;
  is_not_in?: InputMaybe<Array<DownloadMimeEnum>>;
  is_not_null?: InputMaybe<DownloadMimeEnum>;
  is_null?: InputMaybe<DownloadMimeEnum>;
  lt?: InputMaybe<DownloadMimeEnum>;
  lte?: InputMaybe<DownloadMimeEnum>;
  ne?: InputMaybe<DownloadMimeEnum>;
};

export const DownloadStatusEnum = {
  Completed: 'completed',
  Deleted: 'deleted',
  Downloading: 'downloading',
  Failed: 'failed',
  Paused: 'paused',
  Pending: 'pending'
} as const;

export type DownloadStatusEnum = typeof DownloadStatusEnum[keyof typeof DownloadStatusEnum];
export type DownloadStatusEnumFilterInput = {
  eq?: InputMaybe<DownloadStatusEnum>;
  gt?: InputMaybe<DownloadStatusEnum>;
  gte?: InputMaybe<DownloadStatusEnum>;
  is_in?: InputMaybe<Array<DownloadStatusEnum>>;
  is_not_in?: InputMaybe<Array<DownloadStatusEnum>>;
  is_not_null?: InputMaybe<DownloadStatusEnum>;
  is_null?: InputMaybe<DownloadStatusEnum>;
  lt?: InputMaybe<DownloadStatusEnum>;
  lte?: InputMaybe<DownloadStatusEnum>;
  ne?: InputMaybe<DownloadStatusEnum>;
};

export const DownloaderCategoryEnum = {
  Dandanplay: 'dandanplay',
  Qbittorrent: 'qbittorrent'
} as const;

export type DownloaderCategoryEnum = typeof DownloaderCategoryEnum[keyof typeof DownloaderCategoryEnum];
export type DownloaderCategoryEnumFilterInput = {
  eq?: InputMaybe<DownloaderCategoryEnum>;
  gt?: InputMaybe<DownloaderCategoryEnum>;
  gte?: InputMaybe<DownloaderCategoryEnum>;
  is_in?: InputMaybe<Array<DownloaderCategoryEnum>>;
  is_not_in?: InputMaybe<Array<DownloaderCategoryEnum>>;
  is_not_null?: InputMaybe<DownloaderCategoryEnum>;
  is_null?: InputMaybe<DownloaderCategoryEnum>;
  lt?: InputMaybe<DownloaderCategoryEnum>;
  lte?: InputMaybe<DownloaderCategoryEnum>;
  ne?: InputMaybe<DownloaderCategoryEnum>;
};

export type Downloaders = {
  __typename?: 'Downloaders';
  category: DownloaderCategoryEnum;
  createdAt: Scalars['String']['output'];
  download: DownloadsConnection;
  endpoint: Scalars['String']['output'];
  id: Scalars['Int']['output'];
  password: Scalars['String']['output'];
  savePath: Scalars['String']['output'];
  subscriber?: Maybe<Subscribers>;
  subscriberId: Scalars['Int']['output'];
  updatedAt: Scalars['String']['output'];
  username: Scalars['String']['output'];
};


export type DownloadersDownloadArgs = {
  filters?: InputMaybe<DownloadsFilterInput>;
  orderBy?: InputMaybe<DownloadsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type DownloadersBasic = {
  __typename?: 'DownloadersBasic';
  category: DownloaderCategoryEnum;
  createdAt: Scalars['String']['output'];
  endpoint: Scalars['String']['output'];
  id: Scalars['Int']['output'];
  password: Scalars['String']['output'];
  savePath: Scalars['String']['output'];
  subscriberId: Scalars['Int']['output'];
  updatedAt: Scalars['String']['output'];
  username: Scalars['String']['output'];
};

export type DownloadersConnection = {
  __typename?: 'DownloadersConnection';
  edges: Array<DownloadersEdge>;
  nodes: Array<Downloaders>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type DownloadersEdge = {
  __typename?: 'DownloadersEdge';
  cursor: Scalars['String']['output'];
  node: Downloaders;
};

export type DownloadersFilterInput = {
  and?: InputMaybe<Array<DownloadersFilterInput>>;
  category?: InputMaybe<DownloaderCategoryEnumFilterInput>;
  createdAt?: InputMaybe<TextFilterInput>;
  endpoint?: InputMaybe<StringFilterInput>;
  id?: InputMaybe<IntegerFilterInput>;
  or?: InputMaybe<Array<DownloadersFilterInput>>;
  password?: InputMaybe<StringFilterInput>;
  savePath?: InputMaybe<StringFilterInput>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  updatedAt?: InputMaybe<TextFilterInput>;
  username?: InputMaybe<StringFilterInput>;
};

export type DownloadersInsertInput = {
  category: DownloaderCategoryEnum;
  createdAt?: InputMaybe<Scalars['String']['input']>;
  endpoint: Scalars['String']['input'];
  id?: InputMaybe<Scalars['Int']['input']>;
  password: Scalars['String']['input'];
  savePath: Scalars['String']['input'];
  subscriberId?: InputMaybe<Scalars['Int']['input']>;
  updatedAt?: InputMaybe<Scalars['String']['input']>;
  username: Scalars['String']['input'];
};

export type DownloadersOrderInput = {
  category?: InputMaybe<OrderByEnum>;
  createdAt?: InputMaybe<OrderByEnum>;
  endpoint?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  password?: InputMaybe<OrderByEnum>;
  savePath?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  updatedAt?: InputMaybe<OrderByEnum>;
  username?: InputMaybe<OrderByEnum>;
};

export type DownloadersUpdateInput = {
  category?: InputMaybe<DownloaderCategoryEnum>;
  createdAt?: InputMaybe<Scalars['String']['input']>;
  endpoint?: InputMaybe<Scalars['String']['input']>;
  id?: InputMaybe<Scalars['Int']['input']>;
  password?: InputMaybe<Scalars['String']['input']>;
  savePath?: InputMaybe<Scalars['String']['input']>;
  updatedAt?: InputMaybe<Scalars['String']['input']>;
  username?: InputMaybe<Scalars['String']['input']>;
};

export type Downloads = {
  __typename?: 'Downloads';
  allSize?: Maybe<Scalars['Int']['output']>;
  createdAt: Scalars['String']['output'];
  currSize?: Maybe<Scalars['Int']['output']>;
  displayName: Scalars['String']['output'];
  downloader?: Maybe<Downloaders>;
  downloaderId: Scalars['Int']['output'];
  episode?: Maybe<Episodes>;
  episodeId: Scalars['Int']['output'];
  homepage?: Maybe<Scalars['String']['output']>;
  id: Scalars['Int']['output'];
  mime: DownloadMimeEnum;
  originName: Scalars['String']['output'];
  savePath?: Maybe<Scalars['String']['output']>;
  status: DownloadStatusEnum;
  subscriber?: Maybe<Subscribers>;
  subscriberId: Scalars['Int']['output'];
  updatedAt: Scalars['String']['output'];
  url: Scalars['String']['output'];
};

export type DownloadsBasic = {
  __typename?: 'DownloadsBasic';
  allSize?: Maybe<Scalars['Int']['output']>;
  createdAt: Scalars['String']['output'];
  currSize?: Maybe<Scalars['Int']['output']>;
  displayName: Scalars['String']['output'];
  downloaderId: Scalars['Int']['output'];
  episodeId: Scalars['Int']['output'];
  homepage?: Maybe<Scalars['String']['output']>;
  id: Scalars['Int']['output'];
  mime: DownloadMimeEnum;
  originName: Scalars['String']['output'];
  savePath?: Maybe<Scalars['String']['output']>;
  status: DownloadStatusEnum;
  subscriberId: Scalars['Int']['output'];
  updatedAt: Scalars['String']['output'];
  url: Scalars['String']['output'];
};

export type DownloadsConnection = {
  __typename?: 'DownloadsConnection';
  edges: Array<DownloadsEdge>;
  nodes: Array<Downloads>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type DownloadsEdge = {
  __typename?: 'DownloadsEdge';
  cursor: Scalars['String']['output'];
  node: Downloads;
};

export type DownloadsFilterInput = {
  allSize?: InputMaybe<IntegerFilterInput>;
  and?: InputMaybe<Array<DownloadsFilterInput>>;
  createdAt?: InputMaybe<TextFilterInput>;
  currSize?: InputMaybe<IntegerFilterInput>;
  displayName?: InputMaybe<StringFilterInput>;
  downloaderId?: InputMaybe<IntegerFilterInput>;
  episodeId?: InputMaybe<IntegerFilterInput>;
  homepage?: InputMaybe<StringFilterInput>;
  id?: InputMaybe<IntegerFilterInput>;
  mime?: InputMaybe<DownloadMimeEnumFilterInput>;
  or?: InputMaybe<Array<DownloadsFilterInput>>;
  originName?: InputMaybe<StringFilterInput>;
  savePath?: InputMaybe<StringFilterInput>;
  status?: InputMaybe<DownloadStatusEnumFilterInput>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  updatedAt?: InputMaybe<TextFilterInput>;
  url?: InputMaybe<StringFilterInput>;
};

export type DownloadsInsertInput = {
  allSize?: InputMaybe<Scalars['Int']['input']>;
  createdAt?: InputMaybe<Scalars['String']['input']>;
  currSize?: InputMaybe<Scalars['Int']['input']>;
  displayName: Scalars['String']['input'];
  downloaderId: Scalars['Int']['input'];
  episodeId: Scalars['Int']['input'];
  homepage?: InputMaybe<Scalars['String']['input']>;
  id?: InputMaybe<Scalars['Int']['input']>;
  mime: DownloadMimeEnum;
  originName: Scalars['String']['input'];
  savePath?: InputMaybe<Scalars['String']['input']>;
  status: DownloadStatusEnum;
  subscriberId?: InputMaybe<Scalars['Int']['input']>;
  updatedAt?: InputMaybe<Scalars['String']['input']>;
  url: Scalars['String']['input'];
};

export type DownloadsOrderInput = {
  allSize?: InputMaybe<OrderByEnum>;
  createdAt?: InputMaybe<OrderByEnum>;
  currSize?: InputMaybe<OrderByEnum>;
  displayName?: InputMaybe<OrderByEnum>;
  downloaderId?: InputMaybe<OrderByEnum>;
  episodeId?: InputMaybe<OrderByEnum>;
  homepage?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  mime?: InputMaybe<OrderByEnum>;
  originName?: InputMaybe<OrderByEnum>;
  savePath?: InputMaybe<OrderByEnum>;
  status?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  updatedAt?: InputMaybe<OrderByEnum>;
  url?: InputMaybe<OrderByEnum>;
};

export type DownloadsUpdateInput = {
  allSize?: InputMaybe<Scalars['Int']['input']>;
  createdAt?: InputMaybe<Scalars['String']['input']>;
  currSize?: InputMaybe<Scalars['Int']['input']>;
  displayName?: InputMaybe<Scalars['String']['input']>;
  downloaderId?: InputMaybe<Scalars['Int']['input']>;
  episodeId?: InputMaybe<Scalars['Int']['input']>;
  homepage?: InputMaybe<Scalars['String']['input']>;
  id?: InputMaybe<Scalars['Int']['input']>;
  mime?: InputMaybe<DownloadMimeEnum>;
  originName?: InputMaybe<Scalars['String']['input']>;
  savePath?: InputMaybe<Scalars['String']['input']>;
  status?: InputMaybe<DownloadStatusEnum>;
  updatedAt?: InputMaybe<Scalars['String']['input']>;
  url?: InputMaybe<Scalars['String']['input']>;
};

export const EpisodeTypeEnum = {
  Mikan: 'mikan'
} as const;

export type EpisodeTypeEnum = typeof EpisodeTypeEnum[keyof typeof EpisodeTypeEnum];
export type EpisodeTypeEnumFilterInput = {
  eq?: InputMaybe<EpisodeTypeEnum>;
  gt?: InputMaybe<EpisodeTypeEnum>;
  gte?: InputMaybe<EpisodeTypeEnum>;
  is_in?: InputMaybe<Array<EpisodeTypeEnum>>;
  is_not_in?: InputMaybe<Array<EpisodeTypeEnum>>;
  is_not_null?: InputMaybe<EpisodeTypeEnum>;
  is_null?: InputMaybe<EpisodeTypeEnum>;
  lt?: InputMaybe<EpisodeTypeEnum>;
  lte?: InputMaybe<EpisodeTypeEnum>;
  ne?: InputMaybe<EpisodeTypeEnum>;
};

export type Episodes = {
  __typename?: 'Episodes';
  bangumi?: Maybe<Bangumi>;
  bangumiId: Scalars['Int']['output'];
  createdAt: Scalars['String']['output'];
  displayName: Scalars['String']['output'];
  download: SubscriptionsConnection;
  enclosureContentLength?: Maybe<Scalars['Int']['output']>;
  enclosureMagnetLink?: Maybe<Scalars['String']['output']>;
  enclosurePubDate?: Maybe<Scalars['String']['output']>;
  enclosureTorrentLink?: Maybe<Scalars['String']['output']>;
  episodeIndex: Scalars['Int']['output'];
  episodeType: EpisodeTypeEnum;
  fansub?: Maybe<Scalars['String']['output']>;
  homepage?: Maybe<Scalars['String']['output']>;
  id: Scalars['Int']['output'];
  mikanEpisodeId?: Maybe<Scalars['String']['output']>;
  originName: Scalars['String']['output'];
  originPosterLink?: Maybe<Scalars['String']['output']>;
  posterLink?: Maybe<Scalars['String']['output']>;
  resolution?: Maybe<Scalars['String']['output']>;
  season: Scalars['Int']['output'];
  seasonRaw?: Maybe<Scalars['String']['output']>;
  source?: Maybe<Scalars['String']['output']>;
  subscriber?: Maybe<Subscribers>;
  subscriberId: Scalars['Int']['output'];
  subscription: DownloadsConnection;
  subscriptionEpisode: SubscriptionEpisodeConnection;
  subtitle?: Maybe<Scalars['String']['output']>;
  updatedAt: Scalars['String']['output'];
};


export type EpisodesDownloadArgs = {
  filters?: InputMaybe<SubscriptionsFilterInput>;
  orderBy?: InputMaybe<SubscriptionsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type EpisodesSubscriptionArgs = {
  filters?: InputMaybe<DownloadsFilterInput>;
  orderBy?: InputMaybe<DownloadsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type EpisodesSubscriptionEpisodeArgs = {
  filters?: InputMaybe<SubscriptionEpisodeFilterInput>;
  orderBy?: InputMaybe<SubscriptionEpisodeOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type EpisodesBasic = {
  __typename?: 'EpisodesBasic';
  bangumiId: Scalars['Int']['output'];
  createdAt: Scalars['String']['output'];
  displayName: Scalars['String']['output'];
  enclosureContentLength?: Maybe<Scalars['Int']['output']>;
  enclosureMagnetLink?: Maybe<Scalars['String']['output']>;
  enclosurePubDate?: Maybe<Scalars['String']['output']>;
  enclosureTorrentLink?: Maybe<Scalars['String']['output']>;
  episodeIndex: Scalars['Int']['output'];
  episodeType: EpisodeTypeEnum;
  fansub?: Maybe<Scalars['String']['output']>;
  homepage?: Maybe<Scalars['String']['output']>;
  id: Scalars['Int']['output'];
  mikanEpisodeId?: Maybe<Scalars['String']['output']>;
  originName: Scalars['String']['output'];
  originPosterLink?: Maybe<Scalars['String']['output']>;
  posterLink?: Maybe<Scalars['String']['output']>;
  resolution?: Maybe<Scalars['String']['output']>;
  season: Scalars['Int']['output'];
  seasonRaw?: Maybe<Scalars['String']['output']>;
  source?: Maybe<Scalars['String']['output']>;
  subscriberId: Scalars['Int']['output'];
  subtitle?: Maybe<Scalars['String']['output']>;
  updatedAt: Scalars['String']['output'];
};

export type EpisodesConnection = {
  __typename?: 'EpisodesConnection';
  edges: Array<EpisodesEdge>;
  nodes: Array<Episodes>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type EpisodesEdge = {
  __typename?: 'EpisodesEdge';
  cursor: Scalars['String']['output'];
  node: Episodes;
};

export type EpisodesFilterInput = {
  and?: InputMaybe<Array<EpisodesFilterInput>>;
  bangumiId?: InputMaybe<IntegerFilterInput>;
  createdAt?: InputMaybe<TextFilterInput>;
  displayName?: InputMaybe<StringFilterInput>;
  enclosureContentLength?: InputMaybe<IntegerFilterInput>;
  enclosureMagnetLink?: InputMaybe<StringFilterInput>;
  enclosurePubDate?: InputMaybe<TextFilterInput>;
  enclosureTorrentLink?: InputMaybe<StringFilterInput>;
  episodeIndex?: InputMaybe<IntegerFilterInput>;
  episodeType?: InputMaybe<EpisodeTypeEnumFilterInput>;
  fansub?: InputMaybe<StringFilterInput>;
  homepage?: InputMaybe<StringFilterInput>;
  id?: InputMaybe<IntegerFilterInput>;
  mikanEpisodeId?: InputMaybe<StringFilterInput>;
  or?: InputMaybe<Array<EpisodesFilterInput>>;
  originName?: InputMaybe<StringFilterInput>;
  originPosterLink?: InputMaybe<StringFilterInput>;
  posterLink?: InputMaybe<StringFilterInput>;
  resolution?: InputMaybe<StringFilterInput>;
  season?: InputMaybe<IntegerFilterInput>;
  seasonRaw?: InputMaybe<StringFilterInput>;
  source?: InputMaybe<StringFilterInput>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  subtitle?: InputMaybe<StringFilterInput>;
  updatedAt?: InputMaybe<TextFilterInput>;
};

export type EpisodesInsertInput = {
  bangumiId: Scalars['Int']['input'];
  createdAt?: InputMaybe<Scalars['String']['input']>;
  displayName: Scalars['String']['input'];
  enclosureContentLength?: InputMaybe<Scalars['Int']['input']>;
  enclosureMagnetLink?: InputMaybe<Scalars['String']['input']>;
  enclosurePubDate?: InputMaybe<Scalars['String']['input']>;
  enclosureTorrentLink?: InputMaybe<Scalars['String']['input']>;
  episodeIndex: Scalars['Int']['input'];
  episodeType: EpisodeTypeEnum;
  fansub?: InputMaybe<Scalars['String']['input']>;
  homepage?: InputMaybe<Scalars['String']['input']>;
  id?: InputMaybe<Scalars['Int']['input']>;
  mikanEpisodeId?: InputMaybe<Scalars['String']['input']>;
  originName: Scalars['String']['input'];
  originPosterLink?: InputMaybe<Scalars['String']['input']>;
  posterLink?: InputMaybe<Scalars['String']['input']>;
  resolution?: InputMaybe<Scalars['String']['input']>;
  season: Scalars['Int']['input'];
  seasonRaw?: InputMaybe<Scalars['String']['input']>;
  source?: InputMaybe<Scalars['String']['input']>;
  subscriberId?: InputMaybe<Scalars['Int']['input']>;
  subtitle?: InputMaybe<Scalars['String']['input']>;
  updatedAt?: InputMaybe<Scalars['String']['input']>;
};

export type EpisodesOrderInput = {
  bangumiId?: InputMaybe<OrderByEnum>;
  createdAt?: InputMaybe<OrderByEnum>;
  displayName?: InputMaybe<OrderByEnum>;
  enclosureContentLength?: InputMaybe<OrderByEnum>;
  enclosureMagnetLink?: InputMaybe<OrderByEnum>;
  enclosurePubDate?: InputMaybe<OrderByEnum>;
  enclosureTorrentLink?: InputMaybe<OrderByEnum>;
  episodeIndex?: InputMaybe<OrderByEnum>;
  episodeType?: InputMaybe<OrderByEnum>;
  fansub?: InputMaybe<OrderByEnum>;
  homepage?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  mikanEpisodeId?: InputMaybe<OrderByEnum>;
  originName?: InputMaybe<OrderByEnum>;
  originPosterLink?: InputMaybe<OrderByEnum>;
  posterLink?: InputMaybe<OrderByEnum>;
  resolution?: InputMaybe<OrderByEnum>;
  season?: InputMaybe<OrderByEnum>;
  seasonRaw?: InputMaybe<OrderByEnum>;
  source?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  subtitle?: InputMaybe<OrderByEnum>;
  updatedAt?: InputMaybe<OrderByEnum>;
};

export type EpisodesUpdateInput = {
  bangumiId?: InputMaybe<Scalars['Int']['input']>;
  createdAt?: InputMaybe<Scalars['String']['input']>;
  displayName?: InputMaybe<Scalars['String']['input']>;
  enclosureContentLength?: InputMaybe<Scalars['Int']['input']>;
  enclosureMagnetLink?: InputMaybe<Scalars['String']['input']>;
  enclosurePubDate?: InputMaybe<Scalars['String']['input']>;
  enclosureTorrentLink?: InputMaybe<Scalars['String']['input']>;
  episodeIndex?: InputMaybe<Scalars['Int']['input']>;
  episodeType?: InputMaybe<EpisodeTypeEnum>;
  fansub?: InputMaybe<Scalars['String']['input']>;
  homepage?: InputMaybe<Scalars['String']['input']>;
  id?: InputMaybe<Scalars['Int']['input']>;
  mikanEpisodeId?: InputMaybe<Scalars['String']['input']>;
  originName?: InputMaybe<Scalars['String']['input']>;
  originPosterLink?: InputMaybe<Scalars['String']['input']>;
  posterLink?: InputMaybe<Scalars['String']['input']>;
  resolution?: InputMaybe<Scalars['String']['input']>;
  season?: InputMaybe<Scalars['Int']['input']>;
  seasonRaw?: InputMaybe<Scalars['String']['input']>;
  source?: InputMaybe<Scalars['String']['input']>;
  subtitle?: InputMaybe<Scalars['String']['input']>;
  updatedAt?: InputMaybe<Scalars['String']['input']>;
};

export const FeedSourceEnum = {
  SubscriptionEpisode: 'subscription_episode'
} as const;

export type FeedSourceEnum = typeof FeedSourceEnum[keyof typeof FeedSourceEnum];
export type FeedSourceEnumFilterInput = {
  eq?: InputMaybe<FeedSourceEnum>;
  gt?: InputMaybe<FeedSourceEnum>;
  gte?: InputMaybe<FeedSourceEnum>;
  is_in?: InputMaybe<Array<FeedSourceEnum>>;
  is_not_in?: InputMaybe<Array<FeedSourceEnum>>;
  is_not_null?: InputMaybe<FeedSourceEnum>;
  is_null?: InputMaybe<FeedSourceEnum>;
  lt?: InputMaybe<FeedSourceEnum>;
  lte?: InputMaybe<FeedSourceEnum>;
  ne?: InputMaybe<FeedSourceEnum>;
};

export const FeedTypeEnum = {
  Rss: 'rss'
} as const;

export type FeedTypeEnum = typeof FeedTypeEnum[keyof typeof FeedTypeEnum];
export type FeedTypeEnumFilterInput = {
  eq?: InputMaybe<FeedTypeEnum>;
  gt?: InputMaybe<FeedTypeEnum>;
  gte?: InputMaybe<FeedTypeEnum>;
  is_in?: InputMaybe<Array<FeedTypeEnum>>;
  is_not_in?: InputMaybe<Array<FeedTypeEnum>>;
  is_not_null?: InputMaybe<FeedTypeEnum>;
  is_null?: InputMaybe<FeedTypeEnum>;
  lt?: InputMaybe<FeedTypeEnum>;
  lte?: InputMaybe<FeedTypeEnum>;
  ne?: InputMaybe<FeedTypeEnum>;
};

export type Feeds = {
  __typename?: 'Feeds';
  createdAt: Scalars['String']['output'];
  feedSource: FeedSourceEnum;
  feedType: FeedTypeEnum;
  id: Scalars['Int']['output'];
  subscriber?: Maybe<Subscribers>;
  subscriberId?: Maybe<Scalars['Int']['output']>;
  subscription?: Maybe<Subscriptions>;
  subscriptionId?: Maybe<Scalars['Int']['output']>;
  token: Scalars['String']['output'];
  updatedAt: Scalars['String']['output'];
};

export type FeedsBasic = {
  __typename?: 'FeedsBasic';
  createdAt: Scalars['String']['output'];
  feedSource: FeedSourceEnum;
  feedType: FeedTypeEnum;
  id: Scalars['Int']['output'];
  subscriberId?: Maybe<Scalars['Int']['output']>;
  subscriptionId?: Maybe<Scalars['Int']['output']>;
  token: Scalars['String']['output'];
  updatedAt: Scalars['String']['output'];
};

export type FeedsConnection = {
  __typename?: 'FeedsConnection';
  edges: Array<FeedsEdge>;
  nodes: Array<Feeds>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type FeedsEdge = {
  __typename?: 'FeedsEdge';
  cursor: Scalars['String']['output'];
  node: Feeds;
};

export type FeedsFilterInput = {
  and?: InputMaybe<Array<FeedsFilterInput>>;
  createdAt?: InputMaybe<TextFilterInput>;
  feedSource?: InputMaybe<FeedSourceEnumFilterInput>;
  feedType?: InputMaybe<FeedTypeEnumFilterInput>;
  id?: InputMaybe<IntegerFilterInput>;
  or?: InputMaybe<Array<FeedsFilterInput>>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  subscriptionId?: InputMaybe<IntegerFilterInput>;
  token?: InputMaybe<StringFilterInput>;
  updatedAt?: InputMaybe<TextFilterInput>;
};

export type FeedsInsertInput = {
  createdAt?: InputMaybe<Scalars['String']['input']>;
  feedSource: FeedSourceEnum;
  feedType: FeedTypeEnum;
  id?: InputMaybe<Scalars['Int']['input']>;
  subscriberId?: InputMaybe<Scalars['Int']['input']>;
  subscriptionId?: InputMaybe<Scalars['Int']['input']>;
  token?: InputMaybe<Scalars['String']['input']>;
  updatedAt?: InputMaybe<Scalars['String']['input']>;
};

export type FeedsOrderInput = {
  createdAt?: InputMaybe<OrderByEnum>;
  feedSource?: InputMaybe<OrderByEnum>;
  feedType?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  subscriptionId?: InputMaybe<OrderByEnum>;
  token?: InputMaybe<OrderByEnum>;
  updatedAt?: InputMaybe<OrderByEnum>;
};

export type FeedsUpdateInput = {
  createdAt?: InputMaybe<Scalars['String']['input']>;
  feedSource?: InputMaybe<FeedSourceEnum>;
  feedType?: InputMaybe<FeedTypeEnum>;
  id?: InputMaybe<Scalars['Int']['input']>;
  subscriptionId?: InputMaybe<Scalars['Int']['input']>;
  token?: InputMaybe<Scalars['String']['input']>;
  updatedAt?: InputMaybe<Scalars['String']['input']>;
};

export type IntegerFilterInput = {
  between?: InputMaybe<Array<Scalars['Int']['input']>>;
  eq?: InputMaybe<Scalars['Int']['input']>;
  gt?: InputMaybe<Scalars['Int']['input']>;
  gte?: InputMaybe<Scalars['Int']['input']>;
  is_in?: InputMaybe<Array<Scalars['Int']['input']>>;
  is_not_in?: InputMaybe<Array<Scalars['Int']['input']>>;
  is_not_null?: InputMaybe<Scalars['Int']['input']>;
  is_null?: InputMaybe<Scalars['Int']['input']>;
  lt?: InputMaybe<Scalars['Int']['input']>;
  lte?: InputMaybe<Scalars['Int']['input']>;
  ne?: InputMaybe<Scalars['Int']['input']>;
  not_between?: InputMaybe<Array<Scalars['Int']['input']>>;
};

export type Mutation = {
  __typename?: 'Mutation';
  _ping?: Maybe<Scalars['String']['output']>;
  bangumiCreateBatch: Array<BangumiBasic>;
  bangumiCreateOne: BangumiBasic;
  bangumiDelete: Scalars['Int']['output'];
  bangumiUpdate: Array<BangumiBasic>;
  credential3rdCreateBatch: Array<Credential3rdBasic>;
  credential3rdCreateOne: Credential3rdBasic;
  credential3rdDelete: Scalars['Int']['output'];
  credential3rdUpdate: Array<Credential3rdBasic>;
  downloadersCreateBatch: Array<DownloadersBasic>;
  downloadersCreateOne: DownloadersBasic;
  downloadersDelete: Scalars['Int']['output'];
  downloadersUpdate: Array<DownloadersBasic>;
  downloadsCreateBatch: Array<DownloadsBasic>;
  downloadsCreateOne: DownloadsBasic;
  downloadsDelete: Scalars['Int']['output'];
  downloadsUpdate: Array<DownloadsBasic>;
  episodesCreateBatch: Array<EpisodesBasic>;
  episodesCreateOne: EpisodesBasic;
  episodesDelete: Scalars['Int']['output'];
  episodesUpdate: Array<EpisodesBasic>;
  feedsCreateBatch: Array<FeedsBasic>;
  feedsCreateOne: FeedsBasic;
  feedsDelete: Scalars['Int']['output'];
  feedsUpdate: Array<FeedsBasic>;
  subscriberTasksDelete: Scalars['Int']['output'];
  subscriberTasksRetryOne: SubscriberTasks;
  subscriptionBangumiCreateBatch: Array<SubscriptionBangumiBasic>;
  subscriptionBangumiCreateOne: SubscriptionBangumiBasic;
  subscriptionBangumiDelete: Scalars['Int']['output'];
  subscriptionBangumiUpdate: Array<SubscriptionBangumiBasic>;
  subscriptionEpisodeCreateBatch: Array<SubscriptionEpisodeBasic>;
  subscriptionEpisodeCreateOne: SubscriptionEpisodeBasic;
  subscriptionEpisodeDelete: Scalars['Int']['output'];
  subscriptionEpisodeUpdate: Array<SubscriptionEpisodeBasic>;
  subscriptionsCreateBatch: Array<SubscriptionsBasic>;
  subscriptionsCreateOne: SubscriptionsBasic;
  subscriptionsDelete: Scalars['Int']['output'];
  subscriptionsSyncOneFeedsFull: SubscriberTasks;
  subscriptionsSyncOneFeedsIncremental: SubscriberTasks;
  subscriptionsSyncOneSources: SubscriberTasks;
  subscriptionsUpdate: Array<SubscriptionsBasic>;
};


export type MutationBangumiCreateBatchArgs = {
  data: Array<BangumiInsertInput>;
};


export type MutationBangumiCreateOneArgs = {
  data: BangumiInsertInput;
};


export type MutationBangumiDeleteArgs = {
  filter?: InputMaybe<BangumiFilterInput>;
};


export type MutationBangumiUpdateArgs = {
  data: BangumiUpdateInput;
  filter?: InputMaybe<BangumiFilterInput>;
};


export type MutationCredential3rdCreateBatchArgs = {
  data: Array<Credential3rdInsertInput>;
};


export type MutationCredential3rdCreateOneArgs = {
  data: Credential3rdInsertInput;
};


export type MutationCredential3rdDeleteArgs = {
  filter?: InputMaybe<Credential3rdFilterInput>;
};


export type MutationCredential3rdUpdateArgs = {
  data: Credential3rdUpdateInput;
  filter?: InputMaybe<Credential3rdFilterInput>;
};


export type MutationDownloadersCreateBatchArgs = {
  data: Array<DownloadersInsertInput>;
};


export type MutationDownloadersCreateOneArgs = {
  data: DownloadersInsertInput;
};


export type MutationDownloadersDeleteArgs = {
  filter?: InputMaybe<DownloadersFilterInput>;
};


export type MutationDownloadersUpdateArgs = {
  data: DownloadersUpdateInput;
  filter?: InputMaybe<DownloadersFilterInput>;
};


export type MutationDownloadsCreateBatchArgs = {
  data: Array<DownloadsInsertInput>;
};


export type MutationDownloadsCreateOneArgs = {
  data: DownloadsInsertInput;
};


export type MutationDownloadsDeleteArgs = {
  filter?: InputMaybe<DownloadsFilterInput>;
};


export type MutationDownloadsUpdateArgs = {
  data: DownloadsUpdateInput;
  filter?: InputMaybe<DownloadsFilterInput>;
};


export type MutationEpisodesCreateBatchArgs = {
  data: Array<EpisodesInsertInput>;
};


export type MutationEpisodesCreateOneArgs = {
  data: EpisodesInsertInput;
};


export type MutationEpisodesDeleteArgs = {
  filter?: InputMaybe<EpisodesFilterInput>;
};


export type MutationEpisodesUpdateArgs = {
  data: EpisodesUpdateInput;
  filter?: InputMaybe<EpisodesFilterInput>;
};


export type MutationFeedsCreateBatchArgs = {
  data: Array<FeedsInsertInput>;
};


export type MutationFeedsCreateOneArgs = {
  data: FeedsInsertInput;
};


export type MutationFeedsDeleteArgs = {
  filter?: InputMaybe<FeedsFilterInput>;
};


export type MutationFeedsUpdateArgs = {
  data: FeedsUpdateInput;
  filter?: InputMaybe<FeedsFilterInput>;
};


export type MutationSubscriberTasksDeleteArgs = {
  filter?: InputMaybe<SubscriberTasksFilterInput>;
};


export type MutationSubscriberTasksRetryOneArgs = {
  filter?: InputMaybe<SubscriberTasksFilterInput>;
};


export type MutationSubscriptionBangumiCreateBatchArgs = {
  data: Array<SubscriptionBangumiInsertInput>;
};


export type MutationSubscriptionBangumiCreateOneArgs = {
  data: SubscriptionBangumiInsertInput;
};


export type MutationSubscriptionBangumiDeleteArgs = {
  filter?: InputMaybe<SubscriptionBangumiFilterInput>;
};


export type MutationSubscriptionBangumiUpdateArgs = {
  data: SubscriptionBangumiUpdateInput;
  filter?: InputMaybe<SubscriptionBangumiFilterInput>;
};


export type MutationSubscriptionEpisodeCreateBatchArgs = {
  data: Array<SubscriptionEpisodeInsertInput>;
};


export type MutationSubscriptionEpisodeCreateOneArgs = {
  data: SubscriptionEpisodeInsertInput;
};


export type MutationSubscriptionEpisodeDeleteArgs = {
  filter?: InputMaybe<SubscriptionEpisodeFilterInput>;
};


export type MutationSubscriptionEpisodeUpdateArgs = {
  data: SubscriptionEpisodeUpdateInput;
  filter?: InputMaybe<SubscriptionEpisodeFilterInput>;
};


export type MutationSubscriptionsCreateBatchArgs = {
  data: Array<SubscriptionsInsertInput>;
};


export type MutationSubscriptionsCreateOneArgs = {
  data: SubscriptionsInsertInput;
};


export type MutationSubscriptionsDeleteArgs = {
  filter?: InputMaybe<SubscriptionsFilterInput>;
};


export type MutationSubscriptionsSyncOneFeedsFullArgs = {
  filter?: InputMaybe<SubscriptionsFilterInput>;
};


export type MutationSubscriptionsSyncOneFeedsIncrementalArgs = {
  filter?: InputMaybe<SubscriptionsFilterInput>;
};


export type MutationSubscriptionsSyncOneSourcesArgs = {
  filter?: InputMaybe<SubscriptionsFilterInput>;
};


export type MutationSubscriptionsUpdateArgs = {
  data: SubscriptionsUpdateInput;
  filter?: InputMaybe<SubscriptionsFilterInput>;
};

export type OffsetInput = {
  limit: Scalars['Int']['input'];
  offset: Scalars['Int']['input'];
};

export const OrderByEnum = {
  Asc: 'ASC',
  Desc: 'DESC'
} as const;

export type OrderByEnum = typeof OrderByEnum[keyof typeof OrderByEnum];
export type PageInfo = {
  __typename?: 'PageInfo';
  endCursor?: Maybe<Scalars['String']['output']>;
  hasNextPage: Scalars['Boolean']['output'];
  hasPreviousPage: Scalars['Boolean']['output'];
  startCursor?: Maybe<Scalars['String']['output']>;
};

export type PageInput = {
  limit: Scalars['Int']['input'];
  page: Scalars['Int']['input'];
};

export type PaginationInfo = {
  __typename?: 'PaginationInfo';
  current: Scalars['Int']['output'];
  offset: Scalars['Int']['output'];
  pages: Scalars['Int']['output'];
  total: Scalars['Int']['output'];
};

export type PaginationInput = {
  cursor?: InputMaybe<CursorInput>;
  offset?: InputMaybe<OffsetInput>;
  page?: InputMaybe<PageInput>;
};

export type Query = {
  __typename?: 'Query';
  _sea_orm_entity_metadata?: Maybe<Scalars['String']['output']>;
  bangumi: BangumiConnection;
  credential3rd: Credential3rdConnection;
  credential3rdCheckAvailable: Credential3rdCheckAvailableInfo;
  downloaders: DownloadersConnection;
  downloads: DownloadsConnection;
  episodes: EpisodesConnection;
  feeds: FeedsConnection;
  subscriberTasks: SubscriberTasksConnection;
  subscribers: SubscribersConnection;
  subscriptionBangumi: SubscriptionBangumiConnection;
  subscriptionEpisode: SubscriptionEpisodeConnection;
  subscriptions: SubscriptionsConnection;
};


export type Query_Sea_Orm_Entity_MetadataArgs = {
  table_name: Scalars['String']['input'];
};


export type QueryBangumiArgs = {
  filters?: InputMaybe<BangumiFilterInput>;
  orderBy?: InputMaybe<BangumiOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type QueryCredential3rdArgs = {
  filters?: InputMaybe<Credential3rdFilterInput>;
  orderBy?: InputMaybe<Credential3rdOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type QueryCredential3rdCheckAvailableArgs = {
  filter: Credential3rdCheckAvailableInput;
};


export type QueryDownloadersArgs = {
  filters?: InputMaybe<DownloadersFilterInput>;
  orderBy?: InputMaybe<DownloadersOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type QueryDownloadsArgs = {
  filters?: InputMaybe<DownloadsFilterInput>;
  orderBy?: InputMaybe<DownloadsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type QueryEpisodesArgs = {
  filters?: InputMaybe<EpisodesFilterInput>;
  orderBy?: InputMaybe<EpisodesOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type QueryFeedsArgs = {
  filters?: InputMaybe<FeedsFilterInput>;
  orderBy?: InputMaybe<FeedsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type QuerySubscriberTasksArgs = {
  filters?: InputMaybe<SubscriberTasksFilterInput>;
  orderBy?: InputMaybe<SubscriberTasksOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type QuerySubscribersArgs = {
  filters?: InputMaybe<SubscribersFilterInput>;
  orderBy?: InputMaybe<SubscribersOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type QuerySubscriptionBangumiArgs = {
  filters?: InputMaybe<SubscriptionBangumiFilterInput>;
  orderBy?: InputMaybe<SubscriptionBangumiOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type QuerySubscriptionEpisodeArgs = {
  filters?: InputMaybe<SubscriptionEpisodeFilterInput>;
  orderBy?: InputMaybe<SubscriptionEpisodeOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type QuerySubscriptionsArgs = {
  filters?: InputMaybe<SubscriptionsFilterInput>;
  orderBy?: InputMaybe<SubscriptionsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type StringFilterInput = {
  between?: InputMaybe<Array<Scalars['String']['input']>>;
  contains?: InputMaybe<Scalars['String']['input']>;
  ends_with?: InputMaybe<Scalars['String']['input']>;
  eq?: InputMaybe<Scalars['String']['input']>;
  gt?: InputMaybe<Scalars['String']['input']>;
  gte?: InputMaybe<Scalars['String']['input']>;
  is_in?: InputMaybe<Array<Scalars['String']['input']>>;
  is_not_in?: InputMaybe<Array<Scalars['String']['input']>>;
  is_not_null?: InputMaybe<Scalars['String']['input']>;
  is_null?: InputMaybe<Scalars['String']['input']>;
  like?: InputMaybe<Scalars['String']['input']>;
  lt?: InputMaybe<Scalars['String']['input']>;
  lte?: InputMaybe<Scalars['String']['input']>;
  ne?: InputMaybe<Scalars['String']['input']>;
  not_between?: InputMaybe<Array<Scalars['String']['input']>>;
  not_like?: InputMaybe<Scalars['String']['input']>;
  starts_with?: InputMaybe<Scalars['String']['input']>;
};

export type SubscriberIdFilterInput = {
  eq?: InputMaybe<Scalars['Int']['input']>;
};

export const SubscriberTaskStatusEnum = {
  Done: 'Done',
  Failed: 'Failed',
  Killed: 'Killed',
  Pending: 'Pending',
  Running: 'Running',
  Scheduled: 'Scheduled'
} as const;

export type SubscriberTaskStatusEnum = typeof SubscriberTaskStatusEnum[keyof typeof SubscriberTaskStatusEnum];
export const SubscriberTaskTypeEnum = {
  SyncOneSubscriptionFeedsFull: 'sync_one_subscription_feeds_full',
  SyncOneSubscriptionFeedsIncremental: 'sync_one_subscription_feeds_incremental',
  SyncOneSubscriptionSources: 'sync_one_subscription_sources'
} as const;

export type SubscriberTaskTypeEnum = typeof SubscriberTaskTypeEnum[keyof typeof SubscriberTaskTypeEnum];
export type SubscriberTasks = {
  __typename?: 'SubscriberTasks';
  attempts: Scalars['Int']['output'];
  doneAt?: Maybe<Scalars['String']['output']>;
  id: Scalars['String']['output'];
  job: Scalars['Json']['output'];
  lastError?: Maybe<Scalars['String']['output']>;
  lockAt?: Maybe<Scalars['String']['output']>;
  lockBy?: Maybe<Scalars['String']['output']>;
  maxAttempts: Scalars['Int']['output'];
  priority: Scalars['Int']['output'];
  runAt: Scalars['String']['output'];
  status: SubscriberTaskStatusEnum;
  subscriber?: Maybe<Subscribers>;
  subscriberId: Scalars['Int']['output'];
  taskType: SubscriberTaskTypeEnum;
};

export type SubscriberTasksConnection = {
  __typename?: 'SubscriberTasksConnection';
  edges: Array<SubscriberTasksEdge>;
  nodes: Array<SubscriberTasks>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type SubscriberTasksEdge = {
  __typename?: 'SubscriberTasksEdge';
  cursor: Scalars['String']['output'];
  node: SubscriberTasks;
};

export type SubscriberTasksFilterInput = {
  and?: InputMaybe<Array<SubscriberTasksFilterInput>>;
  attempts?: InputMaybe<IntegerFilterInput>;
  doneAt?: InputMaybe<TextFilterInput>;
  id?: InputMaybe<StringFilterInput>;
  job?: InputMaybe<Scalars['JsonbFilterInput']['input']>;
  lastError?: InputMaybe<StringFilterInput>;
  lockAt?: InputMaybe<TextFilterInput>;
  lockBy?: InputMaybe<StringFilterInput>;
  maxAttempts?: InputMaybe<IntegerFilterInput>;
  or?: InputMaybe<Array<SubscriberTasksFilterInput>>;
  priority?: InputMaybe<IntegerFilterInput>;
  runAt?: InputMaybe<TextFilterInput>;
  status?: InputMaybe<StringFilterInput>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  taskType?: InputMaybe<StringFilterInput>;
};

export type SubscriberTasksOrderInput = {
  attempts?: InputMaybe<OrderByEnum>;
  doneAt?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  job?: InputMaybe<OrderByEnum>;
  lastError?: InputMaybe<OrderByEnum>;
  lockAt?: InputMaybe<OrderByEnum>;
  lockBy?: InputMaybe<OrderByEnum>;
  maxAttempts?: InputMaybe<OrderByEnum>;
  priority?: InputMaybe<OrderByEnum>;
  runAt?: InputMaybe<OrderByEnum>;
  status?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  taskType?: InputMaybe<OrderByEnum>;
};

export type Subscribers = {
  __typename?: 'Subscribers';
  bangumi: BangumiConnection;
  bangumiConf?: Maybe<Scalars['Json']['output']>;
  createdAt: Scalars['String']['output'];
  credential3rd: Credential3rdConnection;
  displayName: Scalars['String']['output'];
  downloader: DownloadersConnection;
  episode: EpisodesConnection;
  feed: FeedsConnection;
  id: Scalars['Int']['output'];
  subscriberTask: SubscriberTasksConnection;
  subscription: SubscriptionsConnection;
  updatedAt: Scalars['String']['output'];
};


export type SubscribersBangumiArgs = {
  filters?: InputMaybe<BangumiFilterInput>;
  orderBy?: InputMaybe<BangumiOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type SubscribersCredential3rdArgs = {
  filters?: InputMaybe<Credential3rdFilterInput>;
  orderBy?: InputMaybe<Credential3rdOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type SubscribersDownloaderArgs = {
  filters?: InputMaybe<DownloadersFilterInput>;
  orderBy?: InputMaybe<DownloadersOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type SubscribersEpisodeArgs = {
  filters?: InputMaybe<EpisodesFilterInput>;
  orderBy?: InputMaybe<EpisodesOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type SubscribersFeedArgs = {
  filters?: InputMaybe<FeedsFilterInput>;
  orderBy?: InputMaybe<FeedsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type SubscribersSubscriberTaskArgs = {
  filters?: InputMaybe<SubscriberTasksFilterInput>;
  orderBy?: InputMaybe<SubscriberTasksOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type SubscribersSubscriptionArgs = {
  filters?: InputMaybe<SubscriptionsFilterInput>;
  orderBy?: InputMaybe<SubscriptionsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type SubscribersConnection = {
  __typename?: 'SubscribersConnection';
  edges: Array<SubscribersEdge>;
  nodes: Array<Subscribers>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type SubscribersEdge = {
  __typename?: 'SubscribersEdge';
  cursor: Scalars['String']['output'];
  node: Subscribers;
};

export type SubscribersFilterInput = {
  and?: InputMaybe<Array<SubscribersFilterInput>>;
  id?: InputMaybe<SubscriberIdFilterInput>;
  or?: InputMaybe<Array<SubscribersFilterInput>>;
};

export type SubscribersOrderInput = {
  bangumiConf?: InputMaybe<OrderByEnum>;
  createdAt?: InputMaybe<OrderByEnum>;
  displayName?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  updatedAt?: InputMaybe<OrderByEnum>;
};

export type SubscriptionBangumi = {
  __typename?: 'SubscriptionBangumi';
  bangumi?: Maybe<Bangumi>;
  bangumiId: Scalars['Int']['output'];
  id: Scalars['Int']['output'];
  subscriber?: Maybe<Subscribers>;
  subscriberId: Scalars['Int']['output'];
  subscription?: Maybe<Subscriptions>;
  subscriptionId: Scalars['Int']['output'];
};

export type SubscriptionBangumiBasic = {
  __typename?: 'SubscriptionBangumiBasic';
  bangumiId: Scalars['Int']['output'];
  id: Scalars['Int']['output'];
  subscriberId: Scalars['Int']['output'];
  subscriptionId: Scalars['Int']['output'];
};

export type SubscriptionBangumiConnection = {
  __typename?: 'SubscriptionBangumiConnection';
  edges: Array<SubscriptionBangumiEdge>;
  nodes: Array<SubscriptionBangumi>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type SubscriptionBangumiEdge = {
  __typename?: 'SubscriptionBangumiEdge';
  cursor: Scalars['String']['output'];
  node: SubscriptionBangumi;
};

export type SubscriptionBangumiFilterInput = {
  and?: InputMaybe<Array<SubscriptionBangumiFilterInput>>;
  bangumiId?: InputMaybe<IntegerFilterInput>;
  id?: InputMaybe<IntegerFilterInput>;
  or?: InputMaybe<Array<SubscriptionBangumiFilterInput>>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  subscriptionId?: InputMaybe<IntegerFilterInput>;
};

export type SubscriptionBangumiInsertInput = {
  bangumiId: Scalars['Int']['input'];
  id?: InputMaybe<Scalars['Int']['input']>;
  subscriberId?: InputMaybe<Scalars['Int']['input']>;
  subscriptionId: Scalars['Int']['input'];
};

export type SubscriptionBangumiOrderInput = {
  bangumiId?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  subscriptionId?: InputMaybe<OrderByEnum>;
};

export type SubscriptionBangumiUpdateInput = {
  bangumiId?: InputMaybe<Scalars['Int']['input']>;
  id?: InputMaybe<Scalars['Int']['input']>;
  subscriptionId?: InputMaybe<Scalars['Int']['input']>;
};

export const SubscriptionCategoryEnum = {
  Manual: 'manual',
  MikanBangumi: 'mikan_bangumi',
  MikanSeason: 'mikan_season',
  MikanSubscriber: 'mikan_subscriber'
} as const;

export type SubscriptionCategoryEnum = typeof SubscriptionCategoryEnum[keyof typeof SubscriptionCategoryEnum];
export type SubscriptionCategoryEnumFilterInput = {
  eq?: InputMaybe<SubscriptionCategoryEnum>;
  gt?: InputMaybe<SubscriptionCategoryEnum>;
  gte?: InputMaybe<SubscriptionCategoryEnum>;
  is_in?: InputMaybe<Array<SubscriptionCategoryEnum>>;
  is_not_in?: InputMaybe<Array<SubscriptionCategoryEnum>>;
  is_not_null?: InputMaybe<SubscriptionCategoryEnum>;
  is_null?: InputMaybe<SubscriptionCategoryEnum>;
  lt?: InputMaybe<SubscriptionCategoryEnum>;
  lte?: InputMaybe<SubscriptionCategoryEnum>;
  ne?: InputMaybe<SubscriptionCategoryEnum>;
};

export type SubscriptionEpisode = {
  __typename?: 'SubscriptionEpisode';
  episode?: Maybe<Episodes>;
  episodeId: Scalars['Int']['output'];
  id: Scalars['Int']['output'];
  subscriber?: Maybe<Subscribers>;
  subscriberId: Scalars['Int']['output'];
  subscription?: Maybe<Subscriptions>;
  subscriptionId: Scalars['Int']['output'];
};

export type SubscriptionEpisodeBasic = {
  __typename?: 'SubscriptionEpisodeBasic';
  episodeId: Scalars['Int']['output'];
  id: Scalars['Int']['output'];
  subscriberId: Scalars['Int']['output'];
  subscriptionId: Scalars['Int']['output'];
};

export type SubscriptionEpisodeConnection = {
  __typename?: 'SubscriptionEpisodeConnection';
  edges: Array<SubscriptionEpisodeEdge>;
  nodes: Array<SubscriptionEpisode>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type SubscriptionEpisodeEdge = {
  __typename?: 'SubscriptionEpisodeEdge';
  cursor: Scalars['String']['output'];
  node: SubscriptionEpisode;
};

export type SubscriptionEpisodeFilterInput = {
  and?: InputMaybe<Array<SubscriptionEpisodeFilterInput>>;
  episodeId?: InputMaybe<IntegerFilterInput>;
  id?: InputMaybe<IntegerFilterInput>;
  or?: InputMaybe<Array<SubscriptionEpisodeFilterInput>>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  subscriptionId?: InputMaybe<IntegerFilterInput>;
};

export type SubscriptionEpisodeInsertInput = {
  episodeId: Scalars['Int']['input'];
  id?: InputMaybe<Scalars['Int']['input']>;
  subscriberId?: InputMaybe<Scalars['Int']['input']>;
  subscriptionId: Scalars['Int']['input'];
};

export type SubscriptionEpisodeOrderInput = {
  episodeId?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  subscriptionId?: InputMaybe<OrderByEnum>;
};

export type SubscriptionEpisodeUpdateInput = {
  episodeId?: InputMaybe<Scalars['Int']['input']>;
  id?: InputMaybe<Scalars['Int']['input']>;
  subscriptionId?: InputMaybe<Scalars['Int']['input']>;
};

export type Subscriptions = {
  __typename?: 'Subscriptions';
  bangumi: BangumiConnection;
  category: SubscriptionCategoryEnum;
  createdAt: Scalars['String']['output'];
  credential3rd?: Maybe<Credential3rd>;
  credentialId?: Maybe<Scalars['Int']['output']>;
  displayName: Scalars['String']['output'];
  enabled: Scalars['Boolean']['output'];
  episode: EpisodesConnection;
  feed: FeedsConnection;
  id: Scalars['Int']['output'];
  sourceUrl: Scalars['String']['output'];
  subscriber?: Maybe<Subscribers>;
  subscriberId: Scalars['Int']['output'];
  subscriptionBangumi: SubscriptionBangumiConnection;
  subscriptionEpisode: SubscriptionEpisodeConnection;
  updatedAt: Scalars['String']['output'];
};


export type SubscriptionsBangumiArgs = {
  filters?: InputMaybe<BangumiFilterInput>;
  orderBy?: InputMaybe<BangumiOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type SubscriptionsEpisodeArgs = {
  filters?: InputMaybe<EpisodesFilterInput>;
  orderBy?: InputMaybe<EpisodesOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type SubscriptionsFeedArgs = {
  filters?: InputMaybe<FeedsFilterInput>;
  orderBy?: InputMaybe<FeedsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type SubscriptionsSubscriptionBangumiArgs = {
  filters?: InputMaybe<SubscriptionBangumiFilterInput>;
  orderBy?: InputMaybe<SubscriptionBangumiOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};


export type SubscriptionsSubscriptionEpisodeArgs = {
  filters?: InputMaybe<SubscriptionEpisodeFilterInput>;
  orderBy?: InputMaybe<SubscriptionEpisodeOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type SubscriptionsBasic = {
  __typename?: 'SubscriptionsBasic';
  category: SubscriptionCategoryEnum;
  createdAt: Scalars['String']['output'];
  credentialId?: Maybe<Scalars['Int']['output']>;
  displayName: Scalars['String']['output'];
  enabled: Scalars['Boolean']['output'];
  id: Scalars['Int']['output'];
  sourceUrl: Scalars['String']['output'];
  subscriberId: Scalars['Int']['output'];
  updatedAt: Scalars['String']['output'];
};

export type SubscriptionsConnection = {
  __typename?: 'SubscriptionsConnection';
  edges: Array<SubscriptionsEdge>;
  nodes: Array<Subscriptions>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type SubscriptionsEdge = {
  __typename?: 'SubscriptionsEdge';
  cursor: Scalars['String']['output'];
  node: Subscriptions;
};

export type SubscriptionsFilterInput = {
  and?: InputMaybe<Array<SubscriptionsFilterInput>>;
  category?: InputMaybe<SubscriptionCategoryEnumFilterInput>;
  createdAt?: InputMaybe<TextFilterInput>;
  credentialId?: InputMaybe<IntegerFilterInput>;
  displayName?: InputMaybe<StringFilterInput>;
  enabled?: InputMaybe<BooleanFilterInput>;
  id?: InputMaybe<IntegerFilterInput>;
  or?: InputMaybe<Array<SubscriptionsFilterInput>>;
  sourceUrl?: InputMaybe<StringFilterInput>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  updatedAt?: InputMaybe<TextFilterInput>;
};

export type SubscriptionsInsertInput = {
  category: SubscriptionCategoryEnum;
  createdAt?: InputMaybe<Scalars['String']['input']>;
  credentialId?: InputMaybe<Scalars['Int']['input']>;
  displayName: Scalars['String']['input'];
  enabled: Scalars['Boolean']['input'];
  id?: InputMaybe<Scalars['Int']['input']>;
  sourceUrl: Scalars['String']['input'];
  subscriberId?: InputMaybe<Scalars['Int']['input']>;
  updatedAt?: InputMaybe<Scalars['String']['input']>;
};

export type SubscriptionsOrderInput = {
  category?: InputMaybe<OrderByEnum>;
  createdAt?: InputMaybe<OrderByEnum>;
  credentialId?: InputMaybe<OrderByEnum>;
  displayName?: InputMaybe<OrderByEnum>;
  enabled?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  sourceUrl?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  updatedAt?: InputMaybe<OrderByEnum>;
};

export type SubscriptionsUpdateInput = {
  category?: InputMaybe<SubscriptionCategoryEnum>;
  createdAt?: InputMaybe<Scalars['String']['input']>;
  credentialId?: InputMaybe<Scalars['Int']['input']>;
  displayName?: InputMaybe<Scalars['String']['input']>;
  enabled?: InputMaybe<Scalars['Boolean']['input']>;
  id?: InputMaybe<Scalars['Int']['input']>;
  sourceUrl?: InputMaybe<Scalars['String']['input']>;
  updatedAt?: InputMaybe<Scalars['String']['input']>;
};

export type TextFilterInput = {
  between?: InputMaybe<Array<Scalars['String']['input']>>;
  eq?: InputMaybe<Scalars['String']['input']>;
  gt?: InputMaybe<Scalars['String']['input']>;
  gte?: InputMaybe<Scalars['String']['input']>;
  is_in?: InputMaybe<Array<Scalars['String']['input']>>;
  is_not_in?: InputMaybe<Array<Scalars['String']['input']>>;
  is_not_null?: InputMaybe<Scalars['String']['input']>;
  is_null?: InputMaybe<Scalars['String']['input']>;
  lt?: InputMaybe<Scalars['String']['input']>;
  lte?: InputMaybe<Scalars['String']['input']>;
  ne?: InputMaybe<Scalars['String']['input']>;
  not_between?: InputMaybe<Array<Scalars['String']['input']>>;
};

export type GetCredential3rdQueryVariables = Exact<{
  filters: Credential3rdFilterInput;
  orderBy?: InputMaybe<Credential3rdOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
}>;


export type GetCredential3rdQuery = { __typename?: 'Query', credential3rd: { __typename?: 'Credential3rdConnection', nodes: Array<{ __typename?: 'Credential3rd', id: number, cookies?: string | null, username?: string | null, password?: string | null, userAgent?: string | null, createdAt: string, updatedAt: string, credentialType: Credential3rdTypeEnum }>, paginationInfo?: { __typename?: 'PaginationInfo', total: number, pages: number } | null } };

export type InsertCredential3rdMutationVariables = Exact<{
  data: Credential3rdInsertInput;
}>;


export type InsertCredential3rdMutation = { __typename?: 'Mutation', credential3rdCreateOne: { __typename?: 'Credential3rdBasic', id: number, cookies?: string | null, username?: string | null, password?: string | null, userAgent?: string | null, createdAt: string, updatedAt: string, credentialType: Credential3rdTypeEnum } };

export type UpdateCredential3rdMutationVariables = Exact<{
  data: Credential3rdUpdateInput;
  filters: Credential3rdFilterInput;
}>;


export type UpdateCredential3rdMutation = { __typename?: 'Mutation', credential3rdUpdate: Array<{ __typename?: 'Credential3rdBasic', id: number, cookies?: string | null, username?: string | null, password?: string | null, userAgent?: string | null, createdAt: string, updatedAt: string, credentialType: Credential3rdTypeEnum }> };

export type DeleteCredential3rdMutationVariables = Exact<{
  filters: Credential3rdFilterInput;
}>;


export type DeleteCredential3rdMutation = { __typename?: 'Mutation', credential3rdDelete: number };

export type GetCredential3rdDetailQueryVariables = Exact<{
  id: Scalars['Int']['input'];
}>;


export type GetCredential3rdDetailQuery = { __typename?: 'Query', credential3rd: { __typename?: 'Credential3rdConnection', nodes: Array<{ __typename?: 'Credential3rd', id: number, cookies?: string | null, username?: string | null, password?: string | null, userAgent?: string | null, createdAt: string, updatedAt: string, credentialType: Credential3rdTypeEnum }> } };

export type CheckCredential3rdAvailableQueryVariables = Exact<{
  id: Scalars['Int']['input'];
}>;


export type CheckCredential3rdAvailableQuery = { __typename?: 'Query', credential3rdCheckAvailable: { __typename?: 'Credential3rdCheckAvailableInfo', available: boolean } };

export type InsertFeedMutationVariables = Exact<{
  data: FeedsInsertInput;
}>;


export type InsertFeedMutation = { __typename?: 'Mutation', feedsCreateOne: { __typename?: 'FeedsBasic', id: number, createdAt: string, updatedAt: string, feedType: FeedTypeEnum, token: string } };

export type DeleteFeedMutationVariables = Exact<{
  filters: FeedsFilterInput;
}>;


export type DeleteFeedMutation = { __typename?: 'Mutation', feedsDelete: number };

export type GetSubscriptionsQueryVariables = Exact<{
  filters: SubscriptionsFilterInput;
  orderBy: SubscriptionsOrderInput;
  pagination: PaginationInput;
}>;


export type GetSubscriptionsQuery = { __typename?: 'Query', subscriptions: { __typename?: 'SubscriptionsConnection', nodes: Array<{ __typename?: 'Subscriptions', id: number, createdAt: string, updatedAt: string, displayName: string, category: SubscriptionCategoryEnum, sourceUrl: string, enabled: boolean, credentialId?: number | null }>, paginationInfo?: { __typename?: 'PaginationInfo', total: number, pages: number } | null } };

export type InsertSubscriptionMutationVariables = Exact<{
  data: SubscriptionsInsertInput;
}>;


export type InsertSubscriptionMutation = { __typename?: 'Mutation', subscriptionsCreateOne: { __typename?: 'SubscriptionsBasic', id: number, createdAt: string, updatedAt: string, displayName: string, category: SubscriptionCategoryEnum, sourceUrl: string, enabled: boolean, credentialId?: number | null } };

export type UpdateSubscriptionsMutationVariables = Exact<{
  data: SubscriptionsUpdateInput;
  filters: SubscriptionsFilterInput;
}>;


export type UpdateSubscriptionsMutation = { __typename?: 'Mutation', subscriptionsUpdate: Array<{ __typename?: 'SubscriptionsBasic', id: number, createdAt: string, updatedAt: string, displayName: string, category: SubscriptionCategoryEnum, sourceUrl: string, enabled: boolean }> };

export type DeleteSubscriptionsMutationVariables = Exact<{
  filters?: InputMaybe<SubscriptionsFilterInput>;
}>;


export type DeleteSubscriptionsMutation = { __typename?: 'Mutation', subscriptionsDelete: number };

export type GetSubscriptionDetailQueryVariables = Exact<{
  id: Scalars['Int']['input'];
}>;


export type GetSubscriptionDetailQuery = { __typename?: 'Query', subscriptions: { __typename?: 'SubscriptionsConnection', nodes: Array<{ __typename?: 'Subscriptions', id: number, displayName: string, createdAt: string, updatedAt: string, category: SubscriptionCategoryEnum, sourceUrl: string, enabled: boolean, feed: { __typename?: 'FeedsConnection', nodes: Array<{ __typename?: 'Feeds', id: number, createdAt: string, updatedAt: string, token: string, feedType: FeedTypeEnum, feedSource: FeedSourceEnum }> }, credential3rd?: { __typename?: 'Credential3rd', id: number, username?: string | null } | null, bangumi: { __typename?: 'BangumiConnection', nodes: Array<{ __typename?: 'Bangumi', createdAt: string, updatedAt: string, id: number, mikanBangumiId?: string | null, displayName: string, season: number, seasonRaw?: string | null, fansub?: string | null, mikanFansubId?: string | null, rssLink?: string | null, posterLink?: string | null, homepage?: string | null }> } }> } };

export type SyncSubscriptionFeedsIncrementalMutationVariables = Exact<{
  filter: SubscriptionsFilterInput;
}>;


export type SyncSubscriptionFeedsIncrementalMutation = { __typename?: 'Mutation', subscriptionsSyncOneFeedsIncremental: { __typename?: 'SubscriberTasks', id: string } };

export type SyncSubscriptionFeedsFullMutationVariables = Exact<{
  filter: SubscriptionsFilterInput;
}>;


export type SyncSubscriptionFeedsFullMutation = { __typename?: 'Mutation', subscriptionsSyncOneFeedsFull: { __typename?: 'SubscriberTasks', id: string } };

export type SyncSubscriptionSourcesMutationVariables = Exact<{
  filter: SubscriptionsFilterInput;
}>;


export type SyncSubscriptionSourcesMutation = { __typename?: 'Mutation', subscriptionsSyncOneSources: { __typename?: 'SubscriberTasks', id: string } };

export type GetTasksQueryVariables = Exact<{
  filters: SubscriberTasksFilterInput;
  orderBy: SubscriberTasksOrderInput;
  pagination: PaginationInput;
}>;


export type GetTasksQuery = { __typename?: 'Query', subscriberTasks: { __typename?: 'SubscriberTasksConnection', nodes: Array<{ __typename?: 'SubscriberTasks', id: string, job: any, taskType: SubscriberTaskTypeEnum, status: SubscriberTaskStatusEnum, attempts: number, maxAttempts: number, runAt: string, lastError?: string | null, lockAt?: string | null, lockBy?: string | null, doneAt?: string | null, priority: number }>, paginationInfo?: { __typename?: 'PaginationInfo', total: number, pages: number } | null } };

export type DeleteTasksMutationVariables = Exact<{
  filters: SubscriberTasksFilterInput;
}>;


export type DeleteTasksMutation = { __typename?: 'Mutation', subscriberTasksDelete: number };

export type RetryTasksMutationVariables = Exact<{
  filters: SubscriberTasksFilterInput;
}>;


export type RetryTasksMutation = { __typename?: 'Mutation', subscriberTasksRetryOne: { __typename?: 'SubscriberTasks', id: string, job: any, taskType: SubscriberTaskTypeEnum, status: SubscriberTaskStatusEnum, attempts: number, maxAttempts: number, runAt: string, lastError?: string | null, lockAt?: string | null, lockBy?: string | null, doneAt?: string | null, priority: number } };


export const GetCredential3rdDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetCredential3rd"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filters"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Credential3rdFilterInput"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"orderBy"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"Credential3rdOrderInput"}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"pagination"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"PaginationInput"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"credential3rd"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"filters"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filters"}}},{"kind":"Argument","name":{"kind":"Name","value":"orderBy"},"value":{"kind":"Variable","name":{"kind":"Name","value":"orderBy"}}},{"kind":"Argument","name":{"kind":"Name","value":"pagination"},"value":{"kind":"Variable","name":{"kind":"Name","value":"pagination"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"cookies"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"password"}},{"kind":"Field","name":{"kind":"Name","value":"userAgent"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"credentialType"}}]}},{"kind":"Field","name":{"kind":"Name","value":"paginationInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"total"}},{"kind":"Field","name":{"kind":"Name","value":"pages"}}]}}]}}]}}]} as unknown as DocumentNode<GetCredential3rdQuery, GetCredential3rdQueryVariables>;
export const InsertCredential3rdDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"InsertCredential3rd"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"data"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Credential3rdInsertInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"credential3rdCreateOne"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"data"},"value":{"kind":"Variable","name":{"kind":"Name","value":"data"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"cookies"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"password"}},{"kind":"Field","name":{"kind":"Name","value":"userAgent"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"credentialType"}}]}}]}}]} as unknown as DocumentNode<InsertCredential3rdMutation, InsertCredential3rdMutationVariables>;
export const UpdateCredential3rdDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateCredential3rd"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"data"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Credential3rdUpdateInput"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filters"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Credential3rdFilterInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"credential3rdUpdate"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"data"},"value":{"kind":"Variable","name":{"kind":"Name","value":"data"}}},{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filters"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"cookies"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"password"}},{"kind":"Field","name":{"kind":"Name","value":"userAgent"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"credentialType"}}]}}]}}]} as unknown as DocumentNode<UpdateCredential3rdMutation, UpdateCredential3rdMutationVariables>;
export const DeleteCredential3rdDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"DeleteCredential3rd"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filters"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Credential3rdFilterInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"credential3rdDelete"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filters"}}}]}]}}]} as unknown as DocumentNode<DeleteCredential3rdMutation, DeleteCredential3rdMutationVariables>;
export const GetCredential3rdDetailDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetCredential3rdDetail"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"id"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"credential3rd"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"filters"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"id"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"eq"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}]}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"cookies"}},{"kind":"Field","name":{"kind":"Name","value":"username"}},{"kind":"Field","name":{"kind":"Name","value":"password"}},{"kind":"Field","name":{"kind":"Name","value":"userAgent"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"credentialType"}}]}}]}}]}}]} as unknown as DocumentNode<GetCredential3rdDetailQuery, GetCredential3rdDetailQueryVariables>;
export const CheckCredential3rdAvailableDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"CheckCredential3rdAvailable"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"id"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"credential3rdCheckAvailable"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"id"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"available"}}]}}]}}]} as unknown as DocumentNode<CheckCredential3rdAvailableQuery, CheckCredential3rdAvailableQueryVariables>;
export const InsertFeedDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"InsertFeed"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"data"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"FeedsInsertInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"feedsCreateOne"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"data"},"value":{"kind":"Variable","name":{"kind":"Name","value":"data"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"feedType"}},{"kind":"Field","name":{"kind":"Name","value":"token"}}]}}]}}]} as unknown as DocumentNode<InsertFeedMutation, InsertFeedMutationVariables>;
export const DeleteFeedDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"DeleteFeed"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filters"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"FeedsFilterInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"feedsDelete"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filters"}}}]}]}}]} as unknown as DocumentNode<DeleteFeedMutation, DeleteFeedMutationVariables>;
export const GetSubscriptionsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetSubscriptions"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filters"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"SubscriptionsFilterInput"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"orderBy"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"SubscriptionsOrderInput"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"pagination"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"PaginationInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"subscriptions"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"pagination"},"value":{"kind":"Variable","name":{"kind":"Name","value":"pagination"}}},{"kind":"Argument","name":{"kind":"Name","value":"filters"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filters"}}},{"kind":"Argument","name":{"kind":"Name","value":"orderBy"},"value":{"kind":"Variable","name":{"kind":"Name","value":"orderBy"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"category"}},{"kind":"Field","name":{"kind":"Name","value":"sourceUrl"}},{"kind":"Field","name":{"kind":"Name","value":"enabled"}},{"kind":"Field","name":{"kind":"Name","value":"credentialId"}}]}},{"kind":"Field","name":{"kind":"Name","value":"paginationInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"total"}},{"kind":"Field","name":{"kind":"Name","value":"pages"}}]}}]}}]}}]} as unknown as DocumentNode<GetSubscriptionsQuery, GetSubscriptionsQueryVariables>;
export const InsertSubscriptionDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"InsertSubscription"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"data"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"SubscriptionsInsertInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"subscriptionsCreateOne"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"data"},"value":{"kind":"Variable","name":{"kind":"Name","value":"data"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"category"}},{"kind":"Field","name":{"kind":"Name","value":"sourceUrl"}},{"kind":"Field","name":{"kind":"Name","value":"enabled"}},{"kind":"Field","name":{"kind":"Name","value":"credentialId"}}]}}]}}]} as unknown as DocumentNode<InsertSubscriptionMutation, InsertSubscriptionMutationVariables>;
export const UpdateSubscriptionsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"UpdateSubscriptions"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"data"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"SubscriptionsUpdateInput"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filters"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"SubscriptionsFilterInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"subscriptionsUpdate"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"data"},"value":{"kind":"Variable","name":{"kind":"Name","value":"data"}}},{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filters"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"category"}},{"kind":"Field","name":{"kind":"Name","value":"sourceUrl"}},{"kind":"Field","name":{"kind":"Name","value":"enabled"}}]}}]}}]} as unknown as DocumentNode<UpdateSubscriptionsMutation, UpdateSubscriptionsMutationVariables>;
export const DeleteSubscriptionsDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"DeleteSubscriptions"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filters"}},"type":{"kind":"NamedType","name":{"kind":"Name","value":"SubscriptionsFilterInput"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"subscriptionsDelete"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filters"}}}]}]}}]} as unknown as DocumentNode<DeleteSubscriptionsMutation, DeleteSubscriptionsMutationVariables>;
export const GetSubscriptionDetailDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetSubscriptionDetail"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"id"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"Int"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"subscriptions"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"filters"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"id"},"value":{"kind":"ObjectValue","fields":[{"kind":"ObjectField","name":{"kind":"Name","value":"eq"},"value":{"kind":"Variable","name":{"kind":"Name","value":"id"}}}]}}]}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"category"}},{"kind":"Field","name":{"kind":"Name","value":"sourceUrl"}},{"kind":"Field","name":{"kind":"Name","value":"enabled"}},{"kind":"Field","name":{"kind":"Name","value":"feed"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"token"}},{"kind":"Field","name":{"kind":"Name","value":"feedType"}},{"kind":"Field","name":{"kind":"Name","value":"feedSource"}}]}}]}},{"kind":"Field","name":{"kind":"Name","value":"credential3rd"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"username"}}]}},{"kind":"Field","name":{"kind":"Name","value":"bangumi"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"createdAt"}},{"kind":"Field","name":{"kind":"Name","value":"updatedAt"}},{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"mikanBangumiId"}},{"kind":"Field","name":{"kind":"Name","value":"displayName"}},{"kind":"Field","name":{"kind":"Name","value":"season"}},{"kind":"Field","name":{"kind":"Name","value":"seasonRaw"}},{"kind":"Field","name":{"kind":"Name","value":"fansub"}},{"kind":"Field","name":{"kind":"Name","value":"mikanFansubId"}},{"kind":"Field","name":{"kind":"Name","value":"rssLink"}},{"kind":"Field","name":{"kind":"Name","value":"posterLink"}},{"kind":"Field","name":{"kind":"Name","value":"homepage"}}]}}]}}]}}]}}]}}]} as unknown as DocumentNode<GetSubscriptionDetailQuery, GetSubscriptionDetailQueryVariables>;
export const SyncSubscriptionFeedsIncrementalDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"SyncSubscriptionFeedsIncremental"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filter"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"SubscriptionsFilterInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"subscriptionsSyncOneFeedsIncremental"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filter"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<SyncSubscriptionFeedsIncrementalMutation, SyncSubscriptionFeedsIncrementalMutationVariables>;
export const SyncSubscriptionFeedsFullDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"SyncSubscriptionFeedsFull"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filter"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"SubscriptionsFilterInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"subscriptionsSyncOneFeedsFull"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filter"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<SyncSubscriptionFeedsFullMutation, SyncSubscriptionFeedsFullMutationVariables>;
export const SyncSubscriptionSourcesDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"SyncSubscriptionSources"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filter"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"SubscriptionsFilterInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"subscriptionsSyncOneSources"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filter"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}}]}}]}}]} as unknown as DocumentNode<SyncSubscriptionSourcesMutation, SyncSubscriptionSourcesMutationVariables>;
export const GetTasksDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"query","name":{"kind":"Name","value":"GetTasks"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filters"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"SubscriberTasksFilterInput"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"orderBy"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"SubscriberTasksOrderInput"}}}},{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"pagination"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"PaginationInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"subscriberTasks"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"pagination"},"value":{"kind":"Variable","name":{"kind":"Name","value":"pagination"}}},{"kind":"Argument","name":{"kind":"Name","value":"filters"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filters"}}},{"kind":"Argument","name":{"kind":"Name","value":"orderBy"},"value":{"kind":"Variable","name":{"kind":"Name","value":"orderBy"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"nodes"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"job"}},{"kind":"Field","name":{"kind":"Name","value":"taskType"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"attempts"}},{"kind":"Field","name":{"kind":"Name","value":"maxAttempts"}},{"kind":"Field","name":{"kind":"Name","value":"runAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastError"}},{"kind":"Field","name":{"kind":"Name","value":"lockAt"}},{"kind":"Field","name":{"kind":"Name","value":"lockBy"}},{"kind":"Field","name":{"kind":"Name","value":"doneAt"}},{"kind":"Field","name":{"kind":"Name","value":"priority"}}]}},{"kind":"Field","name":{"kind":"Name","value":"paginationInfo"},"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"total"}},{"kind":"Field","name":{"kind":"Name","value":"pages"}}]}}]}}]}}]} as unknown as DocumentNode<GetTasksQuery, GetTasksQueryVariables>;
export const DeleteTasksDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"DeleteTasks"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filters"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"SubscriberTasksFilterInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"subscriberTasksDelete"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filters"}}}]}]}}]} as unknown as DocumentNode<DeleteTasksMutation, DeleteTasksMutationVariables>;
export const RetryTasksDocument = {"kind":"Document","definitions":[{"kind":"OperationDefinition","operation":"mutation","name":{"kind":"Name","value":"RetryTasks"},"variableDefinitions":[{"kind":"VariableDefinition","variable":{"kind":"Variable","name":{"kind":"Name","value":"filters"}},"type":{"kind":"NonNullType","type":{"kind":"NamedType","name":{"kind":"Name","value":"SubscriberTasksFilterInput"}}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"subscriberTasksRetryOne"},"arguments":[{"kind":"Argument","name":{"kind":"Name","value":"filter"},"value":{"kind":"Variable","name":{"kind":"Name","value":"filters"}}}],"selectionSet":{"kind":"SelectionSet","selections":[{"kind":"Field","name":{"kind":"Name","value":"id"}},{"kind":"Field","name":{"kind":"Name","value":"job"}},{"kind":"Field","name":{"kind":"Name","value":"taskType"}},{"kind":"Field","name":{"kind":"Name","value":"status"}},{"kind":"Field","name":{"kind":"Name","value":"attempts"}},{"kind":"Field","name":{"kind":"Name","value":"maxAttempts"}},{"kind":"Field","name":{"kind":"Name","value":"runAt"}},{"kind":"Field","name":{"kind":"Name","value":"lastError"}},{"kind":"Field","name":{"kind":"Name","value":"lockAt"}},{"kind":"Field","name":{"kind":"Name","value":"lockBy"}},{"kind":"Field","name":{"kind":"Name","value":"doneAt"}},{"kind":"Field","name":{"kind":"Name","value":"priority"}}]}}]}}]} as unknown as DocumentNode<RetryTasksMutation, RetryTasksMutationVariables>;