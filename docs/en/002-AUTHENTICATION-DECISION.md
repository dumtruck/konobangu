# 002 — Authentication and data access

## Authentication contract and deployment boundary

The WebUI and API are independently deployable, including unrelated registrable domains. Both bundled and external issuers use one model: SecurityDept 0.3.3 frontend-oidc TokenSet clients run public-client Authorization Code with PKCE S256, and Recorder APIs verify JWT Bearer access tokens. Static assets are public; application APIs never authenticate session cookies or Basic credentials. Images and RSS additionally accept the bounded native resource credentials below. See the [release contract](005-RELEASE-ACCEPTANCE.md).

The bundled issuer lives in the independent Rust `packages/auth-provider` crate. SecurityDept Basic authentication is confined to its dedicated `/auth-provider/login` endpoint. Browser-owned credentials authenticate the user there; the WebUI has no credential form, credential store, Basic SDK context or iframe login adapter. After login, the provider returns an authorization code to an exactly registered WebUI callback. API authentication is Bearer regardless of the issuer source. External issuers remain direct integrations, without an additional broker.

The resource audience `urn:konobangu:api` and required scope `konobangu:api` are separate from the browser client ID. Resource verification checks signature, issuer, audience, expiry/not-before, required scopes and nonempty subject, and requires `typ=at+jwt` or `application/at+jwt` with an asymmetric signing algorithm. ID tokens, refresh tokens and Basic credentials cannot authorize an API call. External issuers use SecurityDept discovery/JWKS caches; the bundled runtime supplies its public key directly, avoiding HTTP self-discovery before the listener starts. No per-request IdP call or application session lookup is needed.

Application identity comes only from the server-validated `GET /api/auth/user-info` projection. Browser ID claims and provider userinfo do not replace subscriber mapping. Principals have a nonempty issuer and subject with `authType: "oidc"`. Their `(issuer, subject)` identity owns private requests and caches. Restoring different SDK material cancels old confirmation; late results cannot authorize the new material. Routine renewal preserves the same identity's view, drafts and Apollo cache, while protected requests wait for the renewed material's server confirmation. Borrowed authorization is checked again immediately before use; the principal probe bypasses that gate to avoid recursion. No second token store is maintained.

### Runtime deployment configuration

One prebuilt bundle reads `konobangu.config.json` at its origin root, for example `{"version":1,"apiBaseUrl":"https://api.example.net/api/"}`; the default API path is `/api/`. The API may include a reverse-proxy prefix. User information, query, fragment, protocol-relative addresses and non-HTTP(S) URLs are rejected. An HTTP WebUI may explicitly select an HTTP API; HTTPS pages remain subject to mixed-content restrictions. Missing or invalid configuration fails startup rather than guessing a credential destination. The sidecar contains no token, secret or authentication mode.

`GET <apiBaseUrl>auth/config` is public, `no-store`, creates no cookie/session and does not query identity storage. It always publishes `provider: "oidc"`, the resource envelope and the flattened SecurityDept `FrontendOidcModeConfigProjection`. The browser never learns the private `bundled`/`oidc` selector. SDK schemas validate the projection; the host additionally rejects client secrets, executable claims scripts, disabled PKCE, empty identifiers and a callback other than this WebUI's `/auth/callback`.

`auth.webui.allowed_origins` uses strict origin roots or full-match regex rules. Empty/unrestricted policies fail startup. Public config selects the WebUI from `Origin`, or `Referer` only when Origin is absent; invalid, duplicate or `null` Origin cannot fall back. Host, Forwarded, XFF and real IP never choose the WebUI or issuer. Responses are `no-store` and vary by Origin/Referer. Register every callback separately with the issuer; a WebUI regex does not register OAuth callbacks.

### Dependency and type boundaries

`createKonobanguEnvironmentForNativeWeb` creates the only application environment and DI root; see [development rules](001-DEVELOPMENT-VERIFICATION.md#runtime-dependency-and-type-contracts). Before polyfills, `KonobanguBootstrapEnvironment` reads public JSON without importing the SDK. Domains receive the shared HttpClient and concrete environment traits rather than raw host inputs or SDK transport. External JSON stays unknown until schema validation. SDK protocol schemas remain SDK-owned; verified boundary adaptations do not justify double casts or partial service mocks.

## Bundled provider

The issuer is a stable configured HTTPS URL ending in `/auth-provider`, optionally preceded by a deployment prefix. Loopback HTTP is permitted only for development/testing. All advertised endpoints derive from that URL, never request Host/Referer/XFF. A reverse proxy must forward the corresponding path. LAN HTTP WebUI/API can use the same HTTPS issuer. The provider supports one preconfigured public client, `response_type=code`, PKCE S256, authorization-code/refresh grants and `token_endpoint_auth_method=none`.

| Endpoint under the issuer | Responsibility |
| --- | --- |
| `/.well-known/openid-configuration` | OIDC discovery metadata |
| `/authorize` | Validate client, exact callback, scopes, response type and PKCE before starting login |
| `/login` | The only Basic challenge/credential endpoint; resume a short-lived validated pending request |
| `/token` | Public-client authorization-code exchange and rotating opaque refresh tokens |
| `/jwks` | Public RSA signing keys only |
| `/userinfo` | Bearer-protected OIDC subject information |

OAuth2 transitions use `oxide-auth` and `oxide-auth-async`; `openidconnect` constructs the typed OIDC metadata and ID token. Basic validation and real-IP handling use SecurityDept. Real-IP defaults trust the transport peer; configuring trusted proxies does not affect issuer/callback selection. The provider does not offer password grants, implicit flow, dynamic registration, generic account/consent UI, login sessions, token introspection or access-token revocation.

Pending requests, authorization codes and refresh grants belong to the `auth_provider` schema under its own restricted capability. Secrets are stored as hashes. Authorization codes are time-limited and atomically consumed; grants bind client, callback, subject, scope, PKCE and OIDC nonce/authentication time. Refresh rotation consumes and replaces a token transactionally, and family reuse ends subsequent refresh eligibility. Already signed JWTs remain valid to expiry. Expired protocol records are pruned in bounded batches; no separate scheduling subsystem is required.

Production requires a persistent RSA private key, at least 2048 bits, supplied through a file secret. The public JWKS contains no private material and exposes a stable key ID. Access tokens use RS256 and RFC9068 claims; ID tokens use the browser client audience and retain nonce/at_hash semantics. A missing/invalid key fails startup rather than generating an ephemeral identity.

## Native resource credentials

`POST /api/auth/resource-grants` requires Bearer authentication and JSON `{}`. It returns `grant` (Macaroon V2JSON), RFC3339 `issuedAt`/`expiresAt` and `childTtlSeconds: 300`. The in-memory parent binds the verified subscriber, `native-resource:read`, the recorder audience and a 3600-second lifetime. It cannot authorize APIs, renew itself or directly read resources. Before native loading, ResourceService locally attenuates it to an exact image/feed path and at most 300 seconds inside the parent window using HMAC-SHA256.

Only enrolled GET/HEAD image and RSS endpoints accept the `resource_token` query parameter. The URL contains a child signature, never the parent signature or an OIDC access token. Authorization and every restriction are checked before database/storage access, validators and 304 handling. Resource IDs use decoded logical paths independent of hostnames/proxy prefixes. Bearer plus a query credential is rejected; no Basic compatibility path exists. Native images use anonymous CORS. No service worker, PWA registration or manual image/RSS fetch is involved.

One owned service merges parent exchanges and renews five minutes before expiry while visible resources remain active. Hidden/inactive views stop timers. Network/timeout/5xx acquisition failures retry at most three times after 5/10/20 seconds; 401/403/schema failures do not retry on timers. Loaded image URLs survive routine token/parent renewal. Native image failures permit one renewed-grant retry, clearing the old source before remounting so an unchanged URL still creates exactly one replacement request. Identity changes cancel acquisition and clear parent material. Issued children remain valid until expiry after local logout.

RSS children bind `feeds/rss/<locator>`; the locator itself grants no access. Owner RLS still rejects another subscriber's feed. Responses are XML, `no-store`, `Referrer-Policy: no-referrer`, and self-links omit query credentials. ResourceLink owns a native link through its connectable binding; visible-page renewal is local and stale URLs clear on hiding/identity changes. These short URLs are for browser opening, not permanent external-reader credentials. External readers need their own Bearer credential; machine credentials remain future work.

`auth.resource_grants.signing_key` accepts 32-byte unpadded base64url, including through `AUTH__RESOURCE_GRANTS__SIGNING_KEY_FILE`. Omission generates an ephemeral resource key; replicas need the same explicit secret for continuity. This key is independent of the provider RSA key. Keys, credentials and query tokens are redacted, including sensitive macaroon TRACE metadata.

### Crypto bootstrap and LAN HTTP

The concurrent capability barrier preserves native Web Crypto and dynamically supplies missing SHA-256/HMAC capabilities through `@noble/hashes` before SDK construction. Native `getRandomValues` remains mandatory; PKCE is never disabled and secure-context status is not spoofed. A LAN HTTP WebUI/API still uses an HTTPS issuer and exact registered LAN callback. Computation polyfills do not provide transport confidentiality or override mixed-content, CORS or issuer policy. Avoid retaining query credentials in reverse-proxy logs.

## Credential checks and system schedules

`POST /api/credential3rd/{id}/check-available` accepts JSON `{}` under Bearer authentication and returns `{ "available": true }` without caching. A short owner transaction reads the credential and PostgreSQL `xmin`, closes before remote work, then a new transaction conditionally commits encrypted cookies for the same owner/ID/version. Concurrent edits/deletion/checks yield 409 without stale writes; cross-owner access returns 403 and remote failure 502. Retrying rereads current data. Database rollback cannot undo a remote request already received.

System tasks and system Cron require an explicit application administrator capability, independent of authentication type. The bundled configured issuer and fixed subject identify its administrator; changing the username does not change subject. External administrators require explicit trusted issuer/subject configuration and default to none. Shared historical subscribers or untrusted token claims do not grant administration. Non-administrators retain subscriber scope and `system_task_cron IS NULL` on root/relation/update/delete/batch paths. See [task delivery](003-TASK-DELIVERY-AND-MIGRATION.md).

## Configuration and roles

Figment loads defaults < selected TOML/JSON/YAML < canonical double-underscore overrides, with process environment ahead of dotenv. Structured values use Figment TOML-like syntax, for example in Just/dotenvy files:

```dotenv
AUTH__WEBUI__ALLOWED_ORIGINS='[{type="strict",value="https://ui.example"}]'
AUTH__PROVIDER__CREDENTIALS__PASSWORD_FILE=/run/secrets/provider-password
AUTH__PROVIDER__SIGNING__PRIVATE_KEY_PEM_FILE=/run/secrets/provider-key
```

Outer quotes belong to the dotenv loader. Docker `--env-file` treats them literally: omit outer quotes for that loader. Larger lists belong in the configuration file. See [configuration sources and secrets](004-MEDIA-AND-CONFIGURATION.md#configuration-sources).

```toml
[database]
url = "postgres://konobangu:PASSWORD@localhost/konobangu"

[auth]
# External administrators only; use exact verified issuer and subject.
# administrators = [{ issuer = "https://idp.example/", subject = "operator-id" }]

[auth.webui]
allowed_origins = [{ type = "strict", value = "https://ui.example" }]

[auth.provider]
type = "bundled"
issuer = "https://api.example/auth-provider"

[auth.provider.client]
client_id = "konobangu-webui"
redirect_uris = ["https://ui.example/auth/callback"]
scopes = ["openid", "profile", "konobangu:api", "offline_access"]

[auth.provider.credentials]
username = "admin"
# subject defaults to stable "local-admin".

[auth.provider.tokens]
authorization_code_ttl_seconds = 60
access_token_ttl_seconds = 300
refresh_token_ttl_seconds = 2592000
```

Generate a private key without an external OpenSSL executable, keeping it outside committed configuration:

```text
node -e "const fs=require('node:fs'); const c=require('node:crypto'); fs.mkdirSync('secrets',{recursive:true}); fs.writeFileSync('secrets/provider-key.pem',c.generateKeyPairSync('rsa',{modulusLength:2048,privateKeyEncoding:{type:'pkcs8',format:'pem'},publicKeyEncoding:{type:'spki',format:'pem'}}).privateKey,{mode:0o600,flag:'wx'})"
```

For an external issuer, replace the provider block and its nested fields:

```toml
[auth.provider]
type = "oidc"
issuer = "https://idp.example/"
# trusted_ca_file = "/etc/konobangu/idp-ca.pem"

[auth.provider.browser]
client_id = "konobangu-webui"
scopes = ["openid", "profile", "konobangu:api", "offline_access"]
```

Register a public code client with auth method `none`, PKCE S256 and every exact callback. Configure its JWT access tokens for `urn:konobangu:api` and `konobangu:api`; SDK 0.3.3 does not send an RFC8707 resource parameter. Both sources use `auth.provider.resource.audience/required_scopes`. Private-CA issuers require valid API/browser trust; external mode supports `trusted_ca_file`. Discovery/configuration failures fail explicitly, without Basic/development fallback.

Runtime privileges use fixed NOLOGIN, NOINHERIT, NOSUPERUSER, NOBYPASSRLS capabilities:

| Role | Responsibility |
| --- | --- |
| `konobangu_app_scoped_access` | Subscriber-bound application data and worker business stages |
| `konobangu_auth_access` | Verified identity mapping and subscriber initialization |
| `konobangu_task_control_access` | Cross-subscriber task delivery/runs/outbox/Cron and queue storage |
| `konobangu_auth_provider_access` | Bundled provider protocol records, without application data access |

Default deployment uses one existing PostgreSQL login in `database.url`. Identity and scheduler URLs inherit it; bundled `auth.provider.database.url` inherits identity, then database. Startup owner migrations initialize schemas/roles/grants before restricted runtime pools open. Membership is `INHERIT FALSE, SET TRUE`; PostgreSQL 16+ and actual CREATEROLE plus ownership/grant rights are required. No manual grant SQL, generated login or replacement password is needed. Failure never disables RLS or falls back to owner queries.

Pools activate their fixed capability on creation and reset it on checkout. Business transactions additionally use SET LOCAL ROLE and bind subscriber identity. Commit/rollback/cancellation clear transaction scope. Apalis requires its own restricted pool because its queries do not inherit an unrelated business transaction. Use direct connections or session pooling; transaction-pooling proxies cannot preserve this baseline. The shared-login model reduces privileges per operation, not credentials against arbitrary SQL: session_user can switch granted roles. Optional separate runtime URLs and owner `database.migration.url` provide credential isolation, all targeting the same database.

`konobangu-database` owns shared pool/role mechanisms and migration helpers. Recorder owns domain identifiers and explicit application grants; auth-provider owns its schema identifiers/migrations/grants. The owner entry composes migration definitions without duplicate migration histories. Runtime pools cannot perform owner DDL. AuthService and ProviderRuntime each own their respective bounded pool and close it once, including failed initialization.

## Existing database migration and recovery

Stop writers/workers and preserve a tested database/storage backup. Unknown legacy OIDC issuer attribution and conflicting identities fail migration rather than guessing ownership from email or display name. Set `database.migration.legacy_oidc_issuer` only when known exactly. Default startup applies the same owner migration/grant path; explicit maintenance can use:

```sh
mise exec -- cargo run -p recorder --locked --bin migrate-up -- --environment production --config-file /path/to/recorder.config.toml
```

Unreleased intermediate migrations are edited directly; isolated test databases can be recreated. Existing business data is preserved. Historical Basic identity rows may remain as storage records but cannot authenticate APIs. Only the trusted bundled issuer/fixed subject may map to the historical administrator seed; new external identities get independent subscribers. Shared subscribers do not imply administrator status. Component initialization is not a single atomic DDL commit; keep writes stopped on failure, repair permissions and rerun the idempotent owner entry. Recovery uses the prior backup with matching old code/configuration; do not erase production data.

## Endpoints, CSRF and signing out

Application APIs require explicit Bearer credentials; writes also require JSON. OPTIONS precedes authentication. CORS permits configured WebUI origins without credentialed cookies and does not replace authentication. `/api/login` is absent, and API failures never emit a Basic challenge. Only the provider's login endpoint may accept Basic. Provider `/token` CORS permits origins of registered callbacks; it uses public-client form requests without cookies or client secrets.

The callback uses SDK `useTokenSetFrontendCallback` and its configured router for initialization, state/PKCE exchange and CallbackCleanup+Replace parameter removal. A handled response navigates once to the SDK-stored in-app return path; missing/invalid callbacks never fabricate login. The public signed-out page starts the same SDK redirect. Its connectable domain flow owns pending/error state; unmounting does not dispose an application-owned client.

Logout clears local SDK material/principal, cancels private requests and ends the old Apollo identity. It does not initialize a lazy client just to clear it; ready clients clearState, unregister for registry disposal and register a fresh lazy entry. No revocation/introspection call occurs. Issued access/resource tokens remain valid to expiry. External IdP SSO and browser-cached bundled Basic credentials may persist, allowing later login without another prompt. Only short-lived redirect state uses sessionStorage; tokens and projection stay in memory.

The shared HttpClient normalizes requests, enforces destination/epoch and sends credentials omitted. Protected requests require the registry Bearer header; public config never initializes auth. One bounded replay is allowed only for explicitly safe operations; principal probes and mutation/credential commands do not recursively refresh or automatically replay. SDK discovery/token transport remains separate from application HTTP recovery. Only a provider-confirmed invalid grant clears refresh state; network/403 failures remain errors.

## GraphQL, tasks and frontend ownership

Online execution uses explicit async-graphql dynamic resolvers; Seaography supplies schema/naming/filter helpers. Each operation owns the subscriber transaction, shared by roots/relations/loaders/CRUD/batches/task mutations. Resolvers cannot commit, loaders never cache across requests, and mutation success follows commit. Execution errors roll back with data=null; nullable query boundaries retain valid partial data. Cancellation drops uncommitted transactions into rollback. Empty/missing subscriber scope denies private rows.

Owners come from verified mapping, never frontend subscriber_id. Insert injects owner, updates cannot change it, and references are checked in the same owner transaction because PostgreSQL FK checks can bypass RLS. Introspection excludes business execution even in mixed documents. System operations use explicit administration, not authentication method. Tasks close short business transactions before remote/image work, then commit upserts or outbox/fenced stages under the [task contract](003-TASK-DELIVERY-AND-MIGRATION.md).

The SDK registry owns clients/factories; AuthService coordinates verified application identity; ResourceService owns native grants. The environment owns their lifetime, while React borrows it. Each identity owns one ApolloClient/cache; identity/logout ends the old client and rejects late results, while ordinary refresh preserves it. Apollo query/watchQuery permits partial data plus errors; mutation errors never count as successful writes. HTTP 403/network/5xx/parser failures do not start login loops.

[English](002-AUTHENTICATION-DECISION.md) | [中文](../zh/002-AUTHENTICATION-DECISION.md)
