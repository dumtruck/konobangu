# 005 — 0.1.0 release delivery and acceptance

## Current status

**Bundled OIDC and the unified authentication model passed current-host behavioral revalidation; runtime-image acceptance (RC-003) and public distribution (RC-007) remain open.** This page owns required delivery and open acceptance items for the current version, rather than future roadmap work. Publication must bind the final committed source to its successful CI run and artifacts.

| ID | Status | Required delivery |
| --- | --- | --- |
| RC-001 | Implementation accepted; public execution belongs to RC-007 | Tested-source/attempt checks, metadata selections, reusable publication, conflict rejection and same-candidate recovery passed independent review and behavior tests |
| RC-002 | Local artifact acceptance passed; CI provenance belongs to RC-007 | Four native targets built and executed; real archives assembled/extracted with hashes, configuration, WebUI and full notices through the shared utility |
| RC-003 | Authentication revalidation required | The runtime image must pass bundled/external OIDC empty-volume initialization, real task/media work, persistence, non-root/read-only operation and graceful shutdown; anonymous registry checks gate publication |
| RC-004 | Current-host revalidation passed | The full OIDC application passed actual GraphiQL editor/JSON/GraphQL operations with matched successful worker replies, font loading, component/root HMR, preserved drafts and Vite stop/restart; local SDL and document changes regenerate types without a token |
| RC-005 | Closed | The user cancelled restoration of the parallel abstraction; retain the existing owned Rayon pool |
| RC-006 | Current-host revalidation passed | Explicit administrators edited schedules and retried associated runs through real API/pages; isolated PostgreSQL tests passed revised migrations and all four capability boundaries |
| RC-007 | Manager-owned | Existing external settings were verified on 2026-10-07; publication/recovery evidence for the same candidate remains open and belongs to the manager |
| RC-008 | Current-host revalidation passed | Real Chromium, Firefox and WebKit passed bundled/external OIDC, provider-only Basic login, Bearer-only API, LAN HTTP WebUI/API + HTTPS issuer, native image/RSS and bounded retry, retaining native randomness and the Chromium 140 baseline; WebKit uses the scoped login driver described in development verification |

GraphiQL uses `graphiql/setup-workers/vite` inside its lazy route. Worker adapters stay in Vite's source graph; editor runtime entries and CommonJS leaves use explicit optimization. Development scans bootstrap and lazy route sources before serving, and GraphQL/React resolve to single package instances so schema objects remain compatible with the language service. These development optimizations do not eagerly load the production editor. Acceptance exercises actual editor/JSON/GraphQL operations and the full application's Vite configuration, entry and HMR path; setup console labels alone are insufficient.

Cron owns schedules; task_runs owns execution outcomes and retries. Plans retain enable/disable, expressions, timezones, next/last scheduled occurrences and scheduling diagnostics. The API no longer exposes priority/timeout/maxAttempts or execution-status inputs. Failed executions link to associated task runs and the existing retry command; the executor retains its fixed three-attempt policy. Polling owns scheduling correctness; unused legacy NOTIFY functions are removed. A skipped occurrence in last_error is a scheduling diagnostic, not a task failure.

Distribute one WebUI and resolve its runtime API address and public authentication configuration before creating authentication services. Bind the validated configuration to DI; missing or invalid configuration fails explicitly. Public configuration always projects OIDC; bundled/external selection is private server configuration. A preauthenticated test does not establish the real code/PKCE flow or independent deployment.

Local preparation creates a non-publishable preview. The single release orchestrator validates sources and CI artifacts, reserves a draft, verifies downloaded outputs, finalizes the release and promotes stable aliases. See [development verification](001-DEVELOPMENT-VERIFICATION.md) for local previews and maintainer retries.

## Acceptance boundary

Actual native Windows/MSVC, macOS and Linux GNU/musl build and codec evidence can close platform implementation requirements. Platform-independent tests belong on an appropriate single runner; arbitrary changes do not require repeating every platform test. Compilation, Clippy, production WebUI builds and isolated HMR tests establish their respective scopes. Development startup, final packaging and publication require their real entry points.

Release completion requires the implementations above, the selection/publication contracts below and execution evidence for the actual candidate. Exercise empty, individual and combined selections, invalid sources, tag/version conflicts, failed uploads and same-candidate retry through tests and scripts; do not manufacture conflicts with production tags during live acceptance. Never overwrite a public version with different source or selection. Reducing claims, substituting fixtures or moving obligations into the roadmap cannot close missing delivery.

Assess candidate implementation acceptance separately from public distribution. The former requires complete implementations and real-entry verification; the maintainer closes RC-007 external-state and publication evidence during distribution. An absent public Release alone is not a code failure or a prerequisite to changing a version number. Local archives and runtime receipts retain local/dirty provenance; the orchestrator accepts publishable artifacts only from the exact successful verification run for committed source.

[Awa, model tooling, Rust stable and progressive JXL](roadmap/001-SHORT-TERM-ROADMAP.md) retain their agreed future conditions. They do not expand this delivery. Ordinary JPXL remains enabled by default independently of progressive work.

## Independent WebUI and API

WebUI and API may use unrelated HTTPS sites or a static WebUI host. The WebUI stays at its origin root; API base URLs may carry a proxy prefix. Both issuer sources support explicit LAN HTTP WebUI/API through the Crypto capability barrier while retaining a fixed HTTPS issuer. One executable/WebUI uses the same TokenSet/Bearer model for bundled and external issuers. Origin/Referer policy chooses a permitted WebUI callback; the issuer separately registers it exactly. Host/Forwarded/XFF never select the issuer or callback.

The bundled provider is part of the recorder executable, backed by its own protocol schema and restricted pool. Acceptance must cover discovery/JWKS, native Basic only at `/auth-provider/login`, exact client/callback/scope binding, S256, nonce/ID token, expired/replayed/concurrent authorization codes, refresh rotation/family reuse and persistence across restart. A persistent RSA key is required. Reject Basic on every business API and remove `/api/login`. External issuers remain direct, with real SDK expiry/refresh and signing-key rotation coverage.

The WebUI uses the SecurityDept frontend-oidc SDK and its callback infrastructure; the API uses SecurityDept resource verification. Validate signature, issuer, audience, time, required scopes, asymmetric algorithm, `at+jwt` purpose and nonempty subject. ID/refresh tokens cannot authorize APIs. Cache external discovery/JWKS, and supply bundled public keys directly before startup; no per-request IdP or application-session lookup is introduced. Preserve subscriber mapping/RLS, explicit application administration and identity-owned Apollo caches. Historical shared subscribers never establish administration.

Sign-out clears local SDK material/principal, cancels private requests and refresh work, stops the old Apollo client and advances its epoch to reject late results. It does not revoke already issued JWT/resource credentials or guarantee IdP-wide SSO logout. Browser-cached credentials for the bundled login may remain. Public clients use credentials omitted; no API cookie, Basic fallback, bespoke CSRF header or browser credential store exists. Only short-lived SDK redirect state uses sessionStorage. Protocol grant state belongs to the provider, not a resource-server session.

The environment owns unique SDK/application resources; React borrows them. Verify callback cleanup, missing/error states, root shutdown/logout races, principal cancellation, safe replay and HMR using the existing runners. Development codegen/watch consumes the recorder-exported local SDL without a token. The browser fixture explicitly selects HTTP introspection and obtains a real bundled code+S256 Bearer instead of bypassing authentication. Protected commands and mutations are not automatically replayed.

Native image/RSS URLs carry only attenuated resource credentials; never access tokens or parent signatures. Cover parent/child expiry, owner/path isolation, authorization before 304 and bounded renewal without replacing loaded images. Image retry clears the old source and creates exactly one replacement request even for an identical URL. RSS remains native navigation with owner RLS, locally renewed URLs and no query credentials in XML self-links. Run independent-site browser cases with third-party cookies blocked, plus LAN HTTP, Firefox/WebKit and the configured Chromium baseline.

## Automatic release delivery

Use Android-Credential-Provider-Fixer's tested-source dispatch model, shared with SecurityDept; Outposts' image-build/CD model does not cover this application's portable release bundles. Implement this as one delivery, without new feature-domain verification groups.

### Metadata selection and publication history

```toml
[release.artifacts]
bundles = true
runtime_image = true
testing_torrents_image = false
```

These independent booleans live in `konobangu-metadata.toml`; missing, mistyped or unknown fields are rejected. They select publication, not test coverage. All false means verification only, without dispatch, approval, a tag or a GitHub Release. Image-only releases still produce the version tag and a GitHub Release with the manifest/digests; bundle archives are not required. Do not duplicate these flags in repository variables or dispatch inputs, or expose flags that override committed intent.

`just release plan` reports the selection, source SHA, metadata SHA-256 (with checkout newlines normalized) and candidate image tags. `--format github-output` exports the same plan for job conditions. Local previews report a dirty checkout; automatic publication must resolve the plan from the exact clean, tested source. Read metadata from that SHA on retry, rather than from the latest branch tip. Default bundles and runtime images are selected; testing-torrents is opt-in. The release orchestrator implements the source, artifact and publication checks below.

Git history proves the intended selection, not successful publication. Persist the metadata snapshot/hash and source SHA in the release manifest, together with actual successful archive hashes, image digests and workflow run IDs. Bundle receipts already carry the selection/hash, and local publication rejects changed metadata. The reusable fixture publisher requires the authenticated release caller, verified candidate plan and committed opt-in flag; it exports its digest and source/config hashes. Candidate image tags exclude `latest`: promote that alias only after every selected output succeeds and the stable release is finalized. Reject retries that change the selection or source under an already published version; publish a new version instead of rewriting its history.

Before making a draft public, verify every selected image through an anonymous, read-only GHCR client as well as the authenticated candidate checks. Validate the immutable references, content digests and runnable platform configurations. A package that remains private blocks finalization and leaves the draft recoverable. The maintainer enables package visibility on first publication; the workflow does not invent a visibility API or require an administrative PAT. Public retries retain anonymous verification, and successful stable alias promotion is checked against the same image digest.

### Release contents

The candidate version is `0.1.0`, matching `konobangu-metadata.toml` and both changelogs; its tag will be `v0.1.0`. A versioned changelog does not establish that publication has happened. Maintain one application release, not a registry release for every workspace member.

| Deliverable | Release decision |
| --- | --- |
| Application bundles | Four target-named archives: `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `x86_64-pc-windows-msvc`, `aarch64-apple-darwin`. Each includes `recorder-cli`/`.exe`, `webui/`, LICENSE, CHANGELOG, notices and a source/hash receipt. Add a secret-free configuration example in automated bundling; never package user data or credentials. |
| Build profile | Ship default JXL + WebP with the existing owned Rayon pool. The trimmed WebP-only variant is a build/verification option, not a separate release product. |
| Main runtime image | Publish public `ghcr.io/apeiraco/konobangu` for `linux/amd64`, using the already-verified musl executable and the same WebUI. Package it once in a minimal, non-root runtime with CA certificates; no Cargo, Node or codec build tools. Mount configuration, secrets and writable application data separately. PostgreSQL and torrent clients remain external services; the OIDC provider is embedded unless an external issuer is explicitly selected. |
| Image tags and metadata | Publish the full version and `sha-<source SHA>`; move `latest` only after a stable release completes. Record the image digest in the release manifest. Include OCI source/revision/version/license labels, associate the package with the repository and explicitly enable public visibility on initial publication. Add Linux arm64 only after native artifact validation is available. |
| Workspace libraries | Keep Node packages private and Rust crates unpublished. `email`, `testing`, recorder bindings, `util`, `util-derive`, `fetch`, `downloader` and Rust `animeta`, `konobangu-auth-provider`, `konobangu-database` and `konobangu-configuration` are internal build dependencies; distribute the application rather than npm/crates.io packages. Python `konobangu-animeta` and model demos are development/research tooling, with no PyPI/model release in this delivery. |
| Development binaries | Do not ship `mikan-doppel` (mock source), `auth-test-server`, `testcontainers-prune`, examples or `animeta-agent-cli` (currently empty). Email playground previews/exports are development outputs. |
| Test image | Select `ghcr.io/apeiraco/konobangu-testing-torrents` with `release.artifacts.testing_torrents_image`. The release orchestrator calls its reusable publisher only when selected; it is false by default and remains distinct from the runtime image. |
| Build images and docs | Keep the musl builder local to CI, without a user-facing image release. Publish docs through Pages from `master`; do not put the documentation site or compiler caches/declarations in application bundles. |

The main runtime image uses the checked musl executable and production WebUI in a non-root distroless runtime. Its service acceptance checks real database initialization, authentication, persistent data, read-only root filesystem, restart and graceful shutdown.

1. After `Verification report` succeeds for a push to `release`, resolve the committed metadata selection and dispatch `release.yaml` with the exact `source_sha`, `source_ref`, `tests_run_id` and `tests_run_attempt` only when at least one artifact is selected. Add `actions: write` only to the dispatch job. Support manual retry of those same inputs; never auto-publish from PRs, `dev`, the default branch or an unrelated tag push.
2. Validate the source repository, verification workflow identity, completed successful run, push event, release branch and matching commit through the GitHub API. Require synchronized metadata and an exact versioned changelog. Checkout the tested SHA, not the mutable branch tip. Deploy the workflow on the default branch before depending on dispatch.
3. Upload the checked recorder executable, notices and target receipt from each existing platform job, and the production WebUI from the existing TypeScript job. Upload the executable and notices as well as the passing smoke receipt. Store the source SHA, version, target and hashes with each artifact. Consume artifacts from that validated run, without repeating builds or verification. Refactor local and CI bundling to share source/artifact validation; do not expose a manual `--verified` switch.
4. When `bundles` is selected, assemble target-named archives for Linux GNU/musl x64, Windows MSVC x64 and macOS arm64, including WebUI, LICENSE, CHANGELOG and third-party notices. Check candidate provenance and file hashes before generating a release manifest and SHA256SUMS.
5. Gate stable publication through `stable-release` on the `release` ref; deployment environments allow that branch only, with no tag sources. Determine prerelease status from SemVer metadata. Give `contents: write` only to tagging/publication jobs and `packages: write` only to selected image publication jobs. Use `GITHUB_TOKEN`, with no npm/crates/PyPI publishing or signing secrets. Retain read-only repository defaults. Package and smoke-test the verified runtime image, publish its version/source tags without replacing a different existing candidate, and include its digest in the release manifest. Move `latest` after stable publication succeeds. [GitHub registry authentication](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry#authenticating-to-the-container-registry)
6. Create a missing version tag on the tested SHA, and reject a tag pointing elsewhere. Publish a draft, verify its downloaded artifacts, then expose the release. A retry must accept an identical published release and reject different source/artifacts; it must not overwrite a public release. Record the tested run ID and source SHA in the release manifest.
7. Accept a real release-branch push through verification, approval, downloadable bundles and a public pullable runtime image, plus failure/retry and tag-collision cases. Verify image startup, WebUI and API routing, secret-file configuration and persistent data mounts; codec smoke alone is insufficient. The manager verifies external settings and publication results; configuring protection is not evidence of a successful publication. Exercise bundle-only, runtime-image-only, fixture-image-only, combined and empty selections; do not require unselected outputs. Keep the fixture publisher reusable under the same release orchestrator.

[English](005-RELEASE-ACCEPTANCE.md) | [中文](../zh/005-RELEASE-ACCEPTANCE.md)
