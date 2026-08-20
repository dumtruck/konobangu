import { randomBytes } from "node:crypto";
import { createServer } from "node:http";
import { createServer as createSecureServer } from "node:https";
import { exportJWK, generateKeyPair } from "jose";
import Provider, { type Configuration } from "oidc-provider";

/** Test accounts and consent only; OIDC protocol handling remains in the provider. */
export async function startOidcProvider({
  redirectUris,
  tls,
  issuerHostname = "localhost",
}: {
  redirectUris: string[];
  /** Serves the issuer over HTTPS; WebKit blocks an HTTP issuer from HTTPS pages. */
  tls?: { cert: string; key: string };
  issuerHostname?: string;
}) {
  const { privateKey } = await generateKeyPair("RS256", { extractable: true });
  const key = await exportJWK(privateKey);
  Object.assign(key, { kid: "test-rsa", alg: "RS256", use: "sig" });
  let provider: Provider;
  await using ownership = new AsyncDisposableStack();
  const handler: import("node:http").RequestListener = async (
    request,
    response,
  ) => {
    try {
      const uri = new URL(request.url ?? "/", "http://127.0.0.1");
      if (uri.pathname === "/health") {
        response.end("ready");
        return;
      }
      if (uri.pathname.startsWith("/interaction/")) {
        const interaction = await provider.interactionDetails(
          request,
          response,
        );
        if (request.method === "GET") {
          response.setHeader("content-type", "text/html; charset=utf-8");
          response.end(
            `<form method="post"><button name="account" value="A">Sign in as A</button><button name="account" value="B">Sign in as B</button></form>`,
          );
          return;
        }
        if (request.method !== "POST") {
          response.writeHead(405).end();
          return;
        }
        const parts: Buffer[] = [];
        for await (const part of request) parts.push(part);
        const account = new URLSearchParams(
          Buffer.concat(parts).toString(),
        ).get("account");
        if (account !== "A" && account !== "B") {
          response.writeHead(400).end();
          return;
        }
        if (interaction.prompt.name === "login") {
          await provider.interactionFinished(
            request,
            response,
            { login: { accountId: account } },
            { mergeWithLastSubmission: false },
          );
          return;
        }
        await consent(provider, request, response, interaction);
        return;
      }
      provider.callback()(request, response);
    } catch {
      response.writeHead(500).end("Test identity provider interaction failed");
    }
  };
  const server = ownership.use(
    tls ? createSecureServer(tls, handler) : createServer(handler),
  );
  await new Promise<void>((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const address = server.address();
  if (address === null || typeof address === "string")
    throw new Error("Identity provider did not bind a TCP listener");
  const port = address.port;
  const issuer = `${tls ? "https" : "http"}://${issuerHostname}:${port}`;
  const configuration: Configuration = {
    clients: [
      {
        client_id: "konobangu-test",
        application_type: "web",
        redirect_uris: redirectUris,
        response_types: ["code"],
        grant_types: ["authorization_code", "refresh_token"],
        token_endpoint_auth_method: "none",
        id_token_signed_response_alg: "RS256",
      },
    ],
    jwks: { keys: [key] },
    cookies: { keys: [randomBytes(32).toString("hex")] },
    features: {
      devInteractions: { enabled: false },
      resourceIndicators: {
        enabled: true,
        defaultResource: () => "urn:konobangu:api",
        useGrantedResource: () => true,
        getResourceServerInfo: (_context, resource) => {
          if (resource !== "urn:konobangu:api")
            throw new Error("Unknown fixture resource");
          return {
            scope: "konobangu:api",
            audience: resource,
            accessTokenFormat: "jwt",
            jwt: { sign: { alg: "RS256" } },
          };
        },
      },
    },
    clientBasedCORS: (_context, origin, client) =>
      client.redirectUris?.some((uri) => new URL(uri).origin === origin) ===
      true,
    interactions: {
      url: (_context, interaction) => `/interaction/${interaction.uid}`,
    },
    pkce: { required: () => true },
    claims: { openid: ["sub"], profile: ["name"], email: ["email"] },
    scopes: ["openid", "profile", "email", "offline_access"],
    // This registered public API client exercises SDK renewal. The provider
    // still owns refresh-grant issuance, resource binding and rotation.
    issueRefreshToken: (_context, client) =>
      client.clientId === "konobangu-test" &&
      client.grantTypeAllowed("refresh_token"),
    ttl: { AccessToken: 30, RefreshToken: 600 },
    findAccount: async (_context, accountId) => ({
      accountId,
      claims: async () => ({
        sub: accountId,
        name: `User ${accountId}`,
        email: `${accountId.toLowerCase()}@example.test`,
      }),
    }),
  };
  provider = new Provider(issuer, configuration);
  const signingKeys = [key];
  let generation = 0;
  const lifetime = ownership.move();
  const close = () => lifetime.disposeAsync();
  return {
    [Symbol.asyncDispose]: close,
    issuer,
    port,
    async rotateSigningKey() {
      const { privateKey } = await generateKeyPair("RS256", {
        extractable: true,
      });
      const next = await exportJWK(privateKey);
      Object.assign(next, {
        kid: `test-rsa-${++generation}`,
        alg: "RS256",
        use: "sig",
      });
      signingKeys.unshift(next);
      configuration.jwks = { keys: signingKeys };
      // Published-key rotation follows the provider's documented ordering:
      // new first, old retained. Restarting this fixture provider also resets
      // its in-memory grants, so renewal is tested before rotation/re-login.
      provider = new Provider(issuer, configuration);
    },
    /** API/container smoke uses the provider's RFC9068 token model. The browser
     * suite separately exercises its complete public-client PKCE flow. */
    async issueAccessToken(accountId: "A" | "B") {
      const client = await provider.Client.find("konobangu-test");
      if (!client) throw new Error("Fixture public client was not registered");
      const grant = new provider.Grant({
        accountId,
        clientId: client.clientId,
      });
      grant.addResourceScope("urn:konobangu:api", "konobangu:api");
      const resourceServer = new provider.ResourceServer("urn:konobangu:api", {
        audience: "urn:konobangu:api",
        scope: "konobangu:api",
        accessTokenFormat: "jwt",
        jwt: { sign: { alg: "RS256" } },
      });
      const token = new provider.AccessToken({
        client,
        accountId,
        grantId: await grant.save(),
        gty: "authorization_code",
        resourceServer,
        scope: "konobangu:api",
      });
      return token.save();
    },
  };
}

async function consent(
  provider: Provider,
  request: import("node:http").IncomingMessage,
  response: import("node:http").ServerResponse,
  interaction: Awaited<ReturnType<Provider["interactionDetails"]>>,
) {
  const clientId = interaction.params.client_id;
  if (!interaction.session || typeof clientId !== "string")
    throw new Error("Consent requires an authenticated client");
  const grant = interaction.grantId
    ? await provider.Grant.find(interaction.grantId)
    : new provider.Grant({
        accountId: interaction.session.accountId,
        clientId,
      });
  if (!grant) throw new Error("Consent grant was not found");
  const details = interaction.prompt.details;
  if (Array.isArray(details.missingOIDCScope))
    grant.addOIDCScope(details.missingOIDCScope.join(" "));
  if (Array.isArray(details.missingOIDCClaims))
    grant.addOIDCClaims(details.missingOIDCClaims);
  for (const [resource, scopes] of Object.entries(
    details.missingResourceScopes ?? {},
  ))
    grant.addResourceScope(resource, scopes.join(" "));
  const grantId = await grant.save();
  await provider.interactionFinished(
    request,
    response,
    { consent: { grantId } },
    { mergeWithLastSubmission: true },
  );
}
