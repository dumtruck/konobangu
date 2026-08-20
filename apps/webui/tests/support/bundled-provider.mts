import { createHash, randomBytes } from "node:crypto";
import { type } from "arktype";

const Discovery = type({
  issuer: "string",
  authorization_endpoint: "string",
  token_endpoint: "string",
});
const TokenResponse = type({
  access_token: "string",
  token_type: "string",
  expires_in: "number",
  "refresh_token?": "string",
  "id_token?": "string",
});

/** Codegen uses the same real public-client code + S256 exchange as the browser. */
export async function acquireBundledAccessToken(options: {
  issuer: string;
  redirectUri: string;
  signal: AbortSignal;
}) {
  const { issuer, redirectUri, signal } = options;
  const metadataResponse = await fetch(
    `${issuer}/.well-known/openid-configuration`,
    {
      credentials: "omit",
      signal,
      redirect: "error",
    },
  );
  if (!metadataResponse.ok) throw new Error("Bundled discovery failed");
  const metadata = Discovery.assert(await metadataResponse.json());
  if (metadata.issuer !== issuer) throw new Error("Bundled issuer mismatch");
  const verifier = randomBytes(32).toString("base64url");
  const state = randomBytes(24).toString("base64url");
  const authorization = new URL(metadata.authorization_endpoint);
  authorization.search = new URLSearchParams({
    client_id: "konobangu-test",
    response_type: "code",
    redirect_uri: redirectUri,
    scope: "openid profile konobangu:api offline_access",
    state,
    nonce: randomBytes(24).toString("base64url"),
    code_challenge: createHash("sha256").update(verifier).digest("base64url"),
    code_challenge_method: "S256",
  }).toString();
  const begin = await fetch(authorization, {
    redirect: "manual",
    credentials: "omit",
    signal,
  });
  const loginLocation = begin.headers.get("location");
  await begin.body?.cancel();
  if (begin.status !== 302 || !loginLocation)
    throw new Error("Bundled authorization did not request login");
  const login = new URL(loginLocation, issuer);
  if (
    login.origin !== new URL(issuer).origin ||
    login.pathname !== `${new URL(issuer).pathname}/login`
  )
    throw new Error("Bundled login escaped its dedicated endpoint");
  const challenge = await fetch(login, {
    redirect: "manual",
    credentials: "omit",
    signal,
  });
  await challenge.body?.cancel();
  if (
    challenge.status !== 401 ||
    !challenge.headers.get("www-authenticate")?.startsWith("Basic ")
  )
    throw new Error("Bundled login did not issue its Basic challenge");
  const accepted = await fetch(login, {
    redirect: "manual",
    credentials: "omit",
    signal,
    headers: {
      Authorization: `Basic ${Buffer.from("fixture-admin:fixture-admin-password").toString("base64")}`,
    },
  });
  const callbackLocation = accepted.headers.get("location");
  await accepted.body?.cancel();
  if (accepted.status !== 302 || !callbackLocation)
    throw new Error("Bundled login did not issue an authorization code");
  const callback = new URL(callbackLocation);
  const expected = new URL(redirectUri);
  const code = callback.searchParams.get("code");
  if (
    callback.origin !== expected.origin ||
    callback.pathname !== expected.pathname ||
    callback.searchParams.get("state") !== state ||
    !code
  )
    throw new Error("Bundled callback is not bound to this authorization");
  const token = await fetch(metadata.token_endpoint, {
    method: "POST",
    credentials: "omit",
    redirect: "error",
    signal,
    body: new URLSearchParams({
      grant_type: "authorization_code",
      client_id: "konobangu-test",
      code,
      redirect_uri: redirectUri,
      code_verifier: verifier,
    }),
  });
  if (!token.ok)
    throw new Error(`Bundled code exchange failed (${token.status})`);
  const result = TokenResponse.assert(await token.json());
  if (result.token_type.toLowerCase() !== "bearer")
    throw new Error("Bundled issuer returned a non-Bearer token");
  return result;
}

/** The runtime-image restart case proves opaque refresh state survives process exit. */
export async function refreshBundledAccessToken(options: {
  issuer: string;
  refreshToken: string;
  signal: AbortSignal;
}) {
  const response = await fetch(`${options.issuer}/token`, {
    method: "POST",
    credentials: "omit",
    redirect: "error",
    signal: options.signal,
    body: new URLSearchParams({
      grant_type: "refresh_token",
      client_id: "konobangu-test",
      refresh_token: options.refreshToken,
    }),
  });
  if (!response.ok)
    throw new Error(`Bundled refresh exchange failed (${response.status})`);
  return TokenResponse.assert(await response.json());
}
