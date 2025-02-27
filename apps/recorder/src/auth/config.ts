import { LogLevel, type OpenIdConfiguration } from 'oidc-client-rx';

export const isBasicAuth = process.env.AUTH_TYPE === 'basic';

export function buildOidcConfig(): OpenIdConfiguration {
  const origin = window.location.origin;

  const resource = process.env.OIDC_AUDIENCE!;

  return {
    authority: process.env.OIDC_ISSUER!,
    redirectUrl: `${origin}/api/playground/oidc/callback`,
    postLogoutRedirectUri: `${origin}/api/playground`,
    clientId: process.env.OIDC_CLIENT_ID!,
    clientSecret: process.env.OIDC_CLIENT_SECRET,
    scope: process.env.OIDC_EXTRA_SCOPES
      ? `openid profile email offline_access ${process.env.OIDC_EXTRA_SCOPES}`
      : 'openid profile email offline_access',
    triggerAuthorizationResultEvent: true,
    responseType: 'code',
    silentRenew: true,
    useRefreshToken: true,
    logLevel: LogLevel.Debug,
    autoUserInfo: !resource,
    renewUserInfoAfterTokenRenew: !resource,
    customParamsAuthRequest: {
      prompt: 'consent',
      resource,
    },
    customParamsRefreshTokenRequest: {
      resource,
    },
    customParamsCodeRequest: {
      resource,
    },
  };
}
