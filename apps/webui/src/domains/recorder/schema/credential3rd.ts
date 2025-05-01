import {
  Credential3rdTypeEnum,
  type GetCredential3rdDetailQuery,
  type GetCredential3rdQuery,
} from '@/infra/graphql/gql/graphql';
import { gql } from '@apollo/client';
import { type } from 'arktype';

export const GET_CREDENTIAL_3RD = gql`
  query GetCredential3rd($filters: Credential3rdFilterInput!, $orderBy: Credential3rdOrderInput, $pagination: PaginationInput) {
    credential3rd(filters: $filters, orderBy: $orderBy, pagination: $pagination) {
      nodes {
        id
        cookies
        username
        password
        userAgent
        createdAt
        updatedAt
        credentialType
      }
      paginationInfo {
        total
        pages
      }
    }
  }
`;

export const INSERT_CREDENTIAL_3RD = gql`
  mutation InsertCredential3rd($data: Credential3rdInsertInput!) {
    credential3rdCreateOne(data: $data) {
      id
      cookies
      username
      password
      userAgent
      createdAt
      updatedAt
      credentialType
    }
  }
`;

export const UPDATE_CREDENTIAL_3RD = gql`
  mutation UpdateCredential3rd($data: Credential3rdUpdateInput!, $filters: Credential3rdFilterInput!) {
    credential3rdUpdate(data: $data, filter: $filters) {
      id
      cookies
      username
      password
      userAgent
      createdAt
      updatedAt
      credentialType
    }
  }
`;

export const DELETE_CREDENTIAL_3RD = gql`
  mutation DeleteCredential3rd($filters: Credential3rdFilterInput!) {
    credential3rdDelete(filter: $filters)
  }
`;

export const GET_CREDENTIAL_3RD_DETAIL = gql`
  query GetCredential3rdDetail($id: Int!) {
    credential3rd(filters: { id: { eq: $id } }) {
      nodes {
        id
        cookies
        username
        password
        userAgent
        createdAt
        updatedAt
        credentialType
      }
    }
  }
`;

export const Credential3rdTypedMikanSchema = type({
  credentialType: `'${Credential3rdTypeEnum.Mikan}'`,
  username: 'string > 0',
  password: 'string > 0',
});

export type Credential3rdTypedMikan =
  typeof Credential3rdTypedMikanSchema.infer;

const Credential3rdTypedSchema = Credential3rdTypedMikanSchema;

export const Credential3rdInsertSchema = type({
  userAgent: 'string?',
}).and(Credential3rdTypedSchema);

export type Credential3rdInsertDto = typeof Credential3rdInsertSchema.infer;

export type Credential3rdQueryDto =
  GetCredential3rdQuery['credential3rd']['nodes'][number];

export const Credential3rdUpdateSchema = Credential3rdInsertSchema.partial();

export type Credential3rdUpdateDto = typeof Credential3rdUpdateSchema.infer;

export type Credential3rdDetailDto =
  GetCredential3rdDetailQuery['credential3rd']['nodes'][number];
