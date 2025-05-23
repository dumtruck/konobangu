import { gql } from '@apollo/client';

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
