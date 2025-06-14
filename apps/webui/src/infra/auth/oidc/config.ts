import { LogLevel, type OpenIdConfiguration } from 'oidc-client-rx';

export function buildOidcConfig(): OpenIdConfiguration {
  const origin = window.location.origin;

  const resource = process.env.OIDC_AUDIENCE!;

  return {
    authority: process.env.OIDC_ISSUER!,
    redirectUrl: `${origin}/auth/oidc/callback`,
    postLogoutRedirectUri: `${origin}/`,
    clientId: process.env.OIDC_CLIENT_ID!,
    clientSecret: process.env.OIDC_CLIENT_SECRET,
    scope: process.env.OIDC_EXTRA_SCOPES
      ? `openid profile email offline_access ${process.env.OIDC_EXTRA_SCOPES}`
      : 'openid profile email offline_access',
    triggerAuthorizationResultEvent: true,
    responseType: 'code',
    silentRenew: true,
    useRefreshToken: true,
    logLevel: LogLevel.Warn,
    autoUserInfo: !resource,
    renewUserInfoAfterTokenRenew: !resource,
    ignoreNonceAfterRefresh: !!resource,
    renewTimeBeforeTokenExpiresInSeconds: 30,
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
