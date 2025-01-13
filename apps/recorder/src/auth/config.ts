import { type OidcClientSettings, UserManager } from 'oidc-client-ts';

export const PostLoginRedirectUriKey = 'post_login_redirect_uri';

export function buildOidcConfig(): OidcClientSettings {
  const origin = window.location.origin;

  const resource = process.env.OIDC_AUDIENCE!;

  return {
    authority: process.env.OIDC_ISSUER!,
    client_id: process.env.OIDC_CLIENT_ID!,
    client_secret: process.env.OIDC_CLIENT_SECRET!,
    redirect_uri: `${origin}/api/playground/oidc/callback`,
    disablePKCE: false,
    scope: `openid profile email ${process.env.OIDC_EXTRA_SCOPES}`,
    response_type: 'code',
    resource,
    post_logout_redirect_uri: `${origin}/api/playground`,
    extraQueryParams: {
      resource,
    },
    extraTokenParams: {
      resource,
    },
  };
}

export function buildUserManager(): UserManager {
  return new UserManager(buildOidcConfig());
}
