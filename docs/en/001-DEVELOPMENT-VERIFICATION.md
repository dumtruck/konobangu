# 001 — Development, builds and verification

## Tools and entry points

Use mise at the repository root to manage Node, pnpm, Rust, just, CMake and other tools; `rust-toolchain.toml` pins Rust. Just loads `.env` and coordinates pnpm, Cargo, uv and the sole custom utility `scripts/dev-cli.mts`. Recipe comments own the command reference: use `mise exec -- just --list` and `mise exec -- just dev-cli --help`.

Install native prerequisites and managed tools, then frozen Node workspace dependencies, before invoking dev-cli. `just setup` installs managed tools, dependencies and hooks; CI shares `.github/actions/setup` for its prerequisites. Verification does not modify or upgrade workspace dependencies; browser fixtures use fixed supported LTS Node and browser versions. Chromium owns the complete protocol suite; Firefox and WebKit cover focused cross-site authentication and native-image behavior. Every platform and CI use one browser fixture path that needs no privilege and changes no operating-system trust store or hosts file: the fixture generates a short-lived CA and one leaf for its test names under the ignored temp directory with a project dev dependency, needing no system command, and discards the CA key after signing; Chromium trusts only the fixture leaf key and maps the test names through launch arguments; Firefox and WebKit run Playwright's bundled builds, reach the test names through a fixture-owned loopback proxy passed as Playwright's proxy option, and skip certificate errors for that fixture certificate; the fixture's own request client trusts the CA and resolves the names in-process. The test identity provider serves HTTPS on `localhost` with the same certificate, because WebKit blocks an HTTP issuer from an HTTPS page, and the API server verifies it through `auth.provider.trusted_ca_file`.

WebKit requires the runtime libraries supported by its pinned Playwright bundle. The browser runner defaults to one worker: Docker bridge changes can interrupt another active browser, and a shared worker reuses the real server/WebUI build. Explicit runner arguments can override that resource limit.

| Platform | Native build prerequisites |
| --- | --- |
| Windows 11 / Server 2022+ x64 | Visual Studio C++ Build Tools, Windows SDK, CMake, NASM, PowerShell 7.5+; run in the MSVC developer environment |
| macOS arm64/x64 | Xcode Command Line Tools, CMake, NASM |
| Linux x64 | C compiler, CMake, NASM, pkg-config, OpenSSL development package |
| Docker | Linux host or Windows/macOS Docker Desktop; container targets use the pinned linux/amd64 image |

Use pnpm for Node, Cargo for Rust and uv workspace for Python. Configure development dependencies and ordinary business, identity and scheduler database connections through [authentication](002-AUTHENTICATION-DECISION.md). Tests own isolated resources rather than sharing development databases.

## Browser support

The WebUI supports Chrome/Edge 140+, Firefox 140+ and Safari 18.6+, including iOS/iPadOS 18.6+. Use the latest browser or OS release available for the device. Frozen application-embedded browsers and older engines are outside the support contract. Vite's JavaScript and CSS targets reflect this baseline; compatibility is checked at the environment's capability boundary rather than through a browser-version blocklist.

Keep only the missing-capability bootstrap adapters needed by this baseline: Temporal, explicit resource management and Crypto computation on configured LAN HTTP origins. Native random generation remains required. Upgrading a browser does not enable secure-context-only APIs on arbitrary HTTP origins; images use native resource tokens and never depend on Service Workers. Native capabilities bypass their fallback chunks. The same browser runner exercises current engines and Chromium 140; `just setup-browser` installs the pinned fixtures, with the baseline Playwright version defined only in Just.

## Execution model

| Layer | Responsibility |
| --- | --- |
| Native entry points | `lint/check/test/build-*` invoke ecosystem tools; runner arguments select individual tests |
| Gates | `verify` resource stages compose native entries; the checklist is defined only in `justfiles/test.just` |
| Platform artifacts | `platform-check` owns target builds, dependency checks and actual artifact smoke |
| Release | `release prepare` runs the complete gate and platform verification before creating a local preview; the release workflow publishes verified CI inputs |

`verify static` checks metadata, tooling behavior, documentation links, formatting and diffs. `verify rust` uses Cargo/Docker for compilation, complete workspace tests and browser integration. `verify ts` checks declarations, bundler outputs, frontend unit tests, lint and output boundaries. `verify` runs static → rust → ts so Rust bindings and GraphQL generators precede TypeScript consumers. CI invokes the same stages in separate checkouts; the Rust stage rejects binding drift. Browser fixtures build one WebUI into an isolated directory and supply runtime configuration for bundled/external OIDC, same-origin and independently hosted WebUI/API. Bundled native Basic challenges occur only on the HTTPS issuer login endpoint; GraphQL codegen acquires its Bearer through a real code + S256 exchange.

Chromium and Firefox answer bundled login challenges natively. The WebKit driver answers only the exact top-level provider login: it verifies the real 401 and realm, sends Basic to that same URL and forwards the real 302 to the browser; SDK code/PKCE, token and API requests remain unchanged. This avoids WebKit automation credentials being attached to unrelated fetches with `credentials: "omit"`; Safari's native credential-cache behavior requires Safari validation.

Stages follow resources and must not overlap. New tests belong in existing Cargo, Vitest, Node or Playwright runners, without new recipes, CI jobs or gate steps. Only new toolchains, feature combinations or external dependencies justify additional steps; merge overlapping coverage. Do not write tests mirroring recipe literals, job counts or step order; test argument forwarding, exit codes, cleanup and artifact behavior.

`test rust` defaults to workspace libraries, binaries, integrations and playground examples, then runs doctests excluded by Cargo's all-targets mode. Explicit Cargo arguments replace the default selection and can target packages, features, test targets or name filters. Vitest/Node accept native filters. `test browser` uses the production bundled provider, a real external oidc-provider and Chromium to check authentication, polyfills and lifecycle behavior. Runner failures stop verification and preserve the original status; Just owns the default Rust sequence. Missing Docker or build prerequisites fail directly.

On macOS, Recorder debug integration-test executables use DWARF unwinding to avoid the linker's 16 MiB compact-unwind offset limit; release executables and other targets retain their normal linker settings.

Development recorder startup exports its current GraphQL SDL atomically to `apps/recorder/.cache/graphql/schema.graphql` before listening. `just dev-codegen` and `just dev-codegen watch` use that local file by default; watch tracks schema/documents without an access token or expiry. Start the development recorder first. `KONOBANGU_GRAPHQL_SCHEMA` explicitly selects another SDL file or HTTP endpoint; only HTTP uses `KONOBANGU_CODEGEN_AUTHORIZATION`. Production does not export by default; `--graphql-schema-output` is an explicit CLI output, never a public schema route. The browser fixture keeps authenticated HTTP introspection to verify the real Bearer boundary.

The qBittorrent lifecycle fixture pins a multi-platform image tag/digest in `packages/downloader/tests/integration/qbit-lifecycle.test.rs`. Upgrade that reference deliberately and run the downloader suite on Docker Desktop and native Linux. It covers authentication, repeated addition, sync, pause/resume, actual transfer, keep/delete files, timeout and shutdown. WebAPI 2.14 can report duplicate additions as HTTP 409; the adapter queries the server and accepts the conflict only when every hash in that submission exists. Mock tests cover file/magnet conflicts, stale local cache, partially missing batches and other API failures. [Upstream API changes](https://github.com/qbittorrent/qBittorrent/blob/master/WebAPI_Changelog.md#2140)

`lint rust` checks the workspace and the trimmed recorder with all-targets and -D warnings; the codec is the only compile-time choice, so those two configurations cover everything. Media encoding owns one Rayon pool, so no parallel backend is selected at build time. `test media` builds the trimmed WebP-only configuration and checks its library, test binaries and real codecs, covering every `not(feature = "jxl")` branch; `test rust` covers the shipped JXL + WebP build. Both are Cargo work in the same stage, because no media source is OS-conditional. Per-target jobs therefore verify only what actually differs per platform — native toolchain and linking, real codec execution and artifact shape — and never repeat that compilation coverage. Windows/macOS/Linux and Docker all require actual build and runtime evidence.

Lint exceptions are scoped to specific OIDC handlers and shared fixtures; other effective rustc/Clippy and Cargo manifest warnings fail. Inspect raw proc-macro/linker logs because -D warnings does not cover every diagnostic source. Preserve test IdP memory-adapter/TTL messages, Docker and license-tool diagnostics instead of filtering logs to claim zero warnings. Frontend bundle budgets and initial/polyfill/editor loading are checked through configuration and browser assertions.

### GitHub workflows

The default branch is `master`. Verification runs for pull requests targeting `master` or `release`, and pushes to those branches. A push to `dev` alone does not run verification; opening or updating its pull request to either target does. PR branch filters select the destination branch, as described in [GitHub's event reference](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#pull_request).

Only verification pushes to `master` write shared tooling and Cargo caches; PRs, release runs and documentation jobs restore them read-only. `static` installs the Linux CI tool union from `mise.ci.toml` and saves the mise/pnpm caches before the other stages start. The versioned CI profile participates in mise's cache key; tool versions stay in the root configuration. Consumers activate only `just`, Node, pnpm and their requested extra tools; local development tool selection stays unchanged. Each additional native OS has one platform job owning its mise/pnpm cache. The Rust stage independently owns the browser cache and the debug dependency cache; each platform job owns its target's release dependency cache. Keep incompatible targets and profiles isolated. Every cache remains optional for a cold build.

CI disables Cargo incremental compilation and dev/test debug information. Rust caches retain third-party dependencies, excluding workspace artifacts and Cargo-installed binaries; save only successful default-branch jobs. Docker builds mount the same portable Cargo registry/git sources as host tooling, honoring `CARGO_HOME`, and forward the CI profile settings. Testing-torrents Docker layers use their own `testing-torrents` scope with `mode=min`. Do not add duplicate compile jobs for cache priming. GitHub's repository quota covers current and historical cache entries; inspect compressed sizes and hit rates after workflow runs before expanding cached content. [GitHub cache limits](https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching#usage-limits-and-eviction-policy)

`Verification report` is the single required status check, aggregating the static, TypeScript, Rust and platform jobs. Failed, cancelled or skipped stages cannot pass it. New PR/mainline pushes supersede obsolete verification runs; release pushes retain their runs so publication can bind to an exact candidate.

Documentation deploys from `master` when its inputs change, with manual build support; deployment is restricted to `master` through `github-pages`. Application publication uses the Release workflow and the committed metadata selection. The testing-torrents publisher is called only by that orchestrator with a verified candidate and exports its digest. Its standalone Docker build uses pinned tools and the same frozen lockfile/build policy as development. Local preview and maintainer retry entries are described below and in [release delivery and acceptance](005-RELEASE-ACCEPTANCE.md).

The manager owns repository protection, approval environments and Pages settings. Workflows enforce source refs and minimum permissions and validate direct-push candidates. Project metadata owns versions and release selection. Implementers maintain this repository without administering external settings.

## Ownership and organization

The root `justfile` owns global settings, argv adaptation and imports. `justfiles/` separates setup/dev/build/quality/test/release/tools. Just expresses native commands and fixed sequences, using its own if expressions for fixed scopes; complex operations belong in the typechecked and linted CLI. Pure forwarding uses `recipe_args`: native positional argv on Unix, PowerShell 7.5+ CommandWithArgs and argv arrays on Windows, preserving LASTEXITCODE explicitly. cross-spawn handles Windows command shims without reconstructing arguments as shell strings.

The CLI owns cohesive filesystem, network, metadata and artifact operations, including cleanup, TCP readiness, argument adaptation, Docker mounts and release transactions. Just continues to coordinate fixed gates and matrices. Avoid generic quality wrappers, public intermediate commands and manual verified switches. Keep pnpm scripts only for necessary ecosystem lifecycles; removing nonredundant commands requires working replacements.

`just dev-all` and `just dev-proxy` preserve Zellij pane/session controls; process fan-out cannot replace window management. Development codegen supports independent watch and backend-readiness watch. Shared tooling handles platform differences without Bash/GNU utilities for maintained orchestration. Do not add standalone one-off sh/py/mjs entries under root scripts; experiment materials remain locally ignored.

## TypeScript outputs and application lifecycle

| Directory | Purpose |
| --- | --- |
| `dist-tsc/{{project}}/` | tsc reference declarations and declaration maps, never package entry points |
| `dist/` | Vite applications and tsdown JS/public declarations |
| `dist-tsc/{{project}}.tsbuildinfo` | Incremental metadata |

recorder bindings retain ts-rs generator formatting and are excluded from Biome lint, formatting and import organization. File-set/content hashes check generation stability; tsc/tsdown still validate types and declarations. Do not format bindings after Rust tests or exports.

Metadata identifies the root TypeScript configuration; its references graph defines project membership, shared by cleanup/layout checks. The monorepo-tsc condition resolves workspace source directly for the editor, Vite, Vitest and React Email. External consumers read bundler outputs. Public exports provide explicit annotations for declaration generation.

Type checking uses explicit commands: `just check ts` incrementally builds the reference graph, `just types rebuild` owns the clean verification build, and `just dev-types` runs native `tsc -b --watch`. The WebUI Zellij tab separates Vite and type watching into independently controlled panes. Vite owns transpilation and bundling; builds and browser fixtures consume source through monorepo-tsc and can bundle from clean declaration outputs. The TypeScript verification stage owns the type gate, including generated bindings and GraphQL inputs.

Incremental builds can retain orphan outputs after source deletion. Release verification cleans declarations and metadata before a forced rebuild, cleaning partial outputs again on failure. Daily `check ts` is incremental; the complete gate does not repeat an empty incremental build after rebuilding. Cleanup removes declared directories only. Legacy cleanup requires map/source evidence and preserves handwritten declarations, secrets, data and receipts; never use git clean -xdf.

Business time uses Temporal/Intl. Bootstrap concurrently detects and loads missing capabilities before application modules; before that barrier it cannot implement Disposable or access resource primitives. Post-barrier owners use using/await using and standard disposable symbols; resource stacks compose cleanup, transferring long-lived ownership with move. Main owns manual HMR/pagehide; entry creates and returns the resource stack. BFCache persisted pages retain the application. Cancelled initialization never mounts; cleanup is LIFO and releases remaining resources even if one throws.

Capability loading and runtime configuration run concurrently with separate default 10-second deadlines. Failure aborts unfinished sibling requests and clears stage subscriptions/timers; retries use fresh request signals. Unabortable imports may finish later, but their results cannot mount a cancelled application. Disposal settles pending stages quietly while real failures reach the startup error page. Completing the startup stream retains application ownership until explicit disposal.

Node configurations, tests and utilities use native `import.meta.dirname` / `import.meta.filename` for local paths, without building and decoding file URLs. Browser application modules do not depend on these Node properties. [Node ESM](https://nodejs.org/api/esm.html#importmetadirname)

The shared tsdown configuration builds email package JS/declarations. The email playground uses Vite for development, static preview builds and HTML/plain-text exports. React Email components and its renderer retain email-specific markup and CSS inlining; the React Email CLI/Next preview application is not part of the toolchain. `dev-email` serves the template index on loopback:5003 with reloads; `build-email` writes the static preview to dist, and `build-email export` writes to out. Preview modules can attach PreviewProps to their default component. [React Email render](https://react.email/docs/utilities/render)

Node executable utilities and fixtures are erasable TypeScript, run directly by managed Node. Modules shared with tests export functions and guard executable side effects with import.meta.main.

### Runtime dependency and type contracts

The WebUI has exactly one platform boundary: `KonobanguEnvironment`, which extends the SecurityDept SDK environment instead of copying it. Platform differences are expressed only as `CreateKonobanguEnvironmentFor{Host}Options` passed to `createKonobanguEnvironmentFor{Host}` (`NativeWeb` in the browser, `Test` in unit tests); each host resolves the Konobangu traits and selects the SDK host stack, then delegates to the host-independent `createKonobanguEnvironment`, which owns provider composition, the protocol transport policy, the HttpClient and the SDK router. Konobangu traits follow the SDK trait convention: one interface and one short injection token (such as `PLATFORM_INFO`, registered under the `"PLATFORM_INFO_TRAIT"` description) for document, fetch, locale, platform info, media queries, UI storage, page visibility, viewport and resource cryptography; time comes from the SDK `TimeTrait`. Services inject trait tokens and never read `window`, `document`, `navigator` or a storage global themselves. Do not introduce a parallel host, capability or ports abstraction, a second options entry beside the environment creator, or a provider list assembled outside it. The environment is also the unique owner: its creator registers the DestroyRef and application lifetime, rolls back a failed construction, and returns a Disposable whose disposal ends the application; there is no separate runtime object.

`KonobanguBootstrapEnvironment` is the narrower environment that exists before polyfills: document origin, a credential-free `BootstrapConfigClient`, the polyfill barrier, the stage deadline and the loopback policy, created by `createKonobanguBootstrapEnvironmentFor{Host}` without importing the SDK. Native traits provide UI preference/session storage using SDK SyncStorageTrait, including namespaced storage events. A denied store binds one SDK memory store for both data and events; UI storage never enables authentication token persistence. An environment that lacks something a caller needs is extended through creator options, `providers` (which override any unit) or a wrapping creator, never by rebuilding the composition. Authentication integration is defined in the [authentication contract](002-AUTHENTICATION-DECISION.md#dependency-and-type-boundaries).

External values stay unknown until their runtime schema returns a valid result. Form drafts, validated domain data and mutation inputs are distinct types. Contexts without a provider start at null and require a checked consumer hook; error properties require their declared runtime types. Callback wrappers promise their parameter/result signature, not arbitrary properties or overloads of the original function. Avoid double assertions that disguise missing fields, unresolved capabilities or incompatible mocks; checked narrowing, schema-verified adapters and literal inference remain valid.

### Application HTTP middleware

Business services use one schema-validated HttpClient. Ky 2.1.0 supplies the internal HTTP engine; project middleware owns destination bounds, caller/root/identity cancellation, explicit Bearer authorization, credential omission, status handling and permitted recovery. Basic belongs only to the bundled provider login. SDK protocol transport remains separate so discovery/token exchange cannot recurse through application authentication. Native fetch, Ky and SDK transport are implementation details, not domain dependencies. [Ky](https://github.com/sindresorhus/ky/tree/v2.1.0)

Retries are disabled by default. An explicitly replay-safe operation may retry once within the same identity scope; credential-check commands and mutations are never replayed automatically. Profile probes do not recursively refresh themselves on 401. JSON ownership lasts through parsing; streaming ownership lasts through body completion or cancellation. Apollo/cache ownership remains with its identity scope, while HTTP policy is shared.

### Disposable ownership

Disposal consumes unique ownership. Successful acquisition registers a resource exactly once with a private owner; consumers receive borrowed interfaces without release methods. Providers register lazy resources at creation, not during cleanup. Disposal never resolves an unused provider. Local construction stacks roll back failures and move ownership explicitly after success.

The bootstrap coordinator may deduplicate HMR/pagehide/stop requests; it invokes the underlying application lifetime once. Business resources do not silently absorb repeated dispose/setup through started/disposed guards. Detect illegal registration or disposal at the owner when necessary, and fail fast. Epochs, single-flight and cancel/stop remain separate business or concurrency protocols.

The root stack owns one SDK DestroyRef, which owns the TokenSet registry; the registry alone owns OIDC records/clients. Application resources have one separate root-stack registration. Acquiring dependencies before their consumers makes LIFO tear down UI, queries and requests before dependent auth state and SDK resources. React providers borrow the injector and do not acquire root disposal authority. Identity changes replace private scopes, not the entire RouterProvider. Shutdown cancels in-flight work, and pending logout/initialization cannot register clients after the root ends.

### Domain objects and the React bridge

Domain state lives in framework-free services and objects; React only bridges to them.

- **A domain owns its lifecycle.** What starts, renews, retries, resets or cancels something is decided by a domain service or object that works without React. A component never runs that logic in effects, nor keeps the domain's state in `useState`/`useRef`.
- **A service drives React only through its own objects.** It may react to an upstream domain's signal to reset itself, but it publishes its own signal and never restates the upstream domain's rules. The authentication service publishes identity changes that exclude same-identity renewals; the GraphQL service answers one by retiring its Apollo client and publishing a new one, leaving authorization to the HTTP client.
- **View-scoped domain objects connect, they are not disposed.** An object a view uses for a while exposes a state signal with `connect()`/`disconnect()`, counted and reconnectable, so Strict Mode or Suspense mounting a view repeatedly never ends it. The owning service creates it and stays the only owner of disposal. Give such an object an identity or registry only when sharing it buys something the service does not already share.
- **The bridge is thin.** A provider or component reads signals, connects and disconnects, forwards user intent and reports element-level facts (near the viewport, loaded, failed to load). `AuthProvider` and `GraphqlProvider` bridge their domains at the root; `PrivateIdentityView` remounts component-local state that no service can reset; `ResourceLink` and `Img` connect the resource domain's `ResourceBinding` and `ImageBinding`; the signed-out page connects the authentication domain's `SignInFlow`.
- **Route-bound CRUD pages are exempt.** A page whose data and form live exactly as long as its route uses TanStack Query/Form and Apollo hooks directly; a domain layer there would only mirror the router's lifecycle.
- **Each domain object has framework-free tests** beside its service; component tests cover only the bridge.

### Test organization

Within each apps/packages project, use directories to identify the test layer:

| Layer | Location | Discovery |
| --- | --- | --- |
| JS/Node/TS/Python unit | Source-local `**/__test__/**` or project-root `tests/**`, excluding integration/e2e | `*.test.*` or `*.spec.*`, with the language's actual extension |
| Rust unit | Inline `#[cfg(test)]` modules or separate test modules | Native Cargo library/binary test discovery |
| Integration | `tests/integration/**` | `*.test.{ts,rs,py}` or `*.spec.{ts,rs,py}` and applicable JS/TS extensions |
| End-to-end | `tests/e2e/**` | The same filename convention, with complete product journeys |

JS/TS extensions include js/ts, jsx/tsx and the c/m module variants. Use glob braces rather than regex alternation for runner patterns. The directory declares the layer; using a browser alone does not make a test end-to-end. Unit fixtures can use controlled temporary files or subprocesses. Integration tests combine production components or external services; end-to-end tests exercise complete user journeys. Helpers, fixtures and snapshots stay alongside their owning layer and are not executable test entries. Shared fixtures may serve multiple suites.

WebUI unit tests obtain the application only through the test host: `createKonobanguEnvironmentForTest` composes the SDK test environment with deterministic Konobangu traits, and the shared support fixtures wrap it with a request handler, a controllable principal, provider rendering and deferred replies, releasing everything when the test finishes. A behavior is asserted by the suite that owns it; another suite that needs that behavior uses the shared fixture or overrides one unit through `providers` instead of re-creating mocks, provider lists, injectors or cleanup registries. Spy, global, timer and React cleanup is global setup, not per-file code.

Repository utility modules follow the same layout under scripts: unit contracts live in scripts/tests, real Just/CLI composition lives in scripts/tests/integration. The existing tooling runner collects both recursively. Cargo explicitly registers nested integration entries with stable target names; helper modules and e2e support binaries are not discovered as test targets. Vitest collects only source-local __test__ directories; the WebUI's root tests directory belongs entirely to Playwright. Build-time modules of an application, such as its Vite plugins, live in that application's `tooling/` directory with their own `__test__`, type-checked by its scripts TypeScript project rather than the application project. Playwright has only integration/e2e projects, each collecting its corresponding directory without overlap; spec files sit directly in their layer and shared launchers live under support, so no feature directory creates a separate project. Deployment-backed e2e specs import the automatic deployment fixture and declares in-file execution in order, sharing that worker's backend. Other suites remain parallel. Listing tests does not acquire services. Native filters continue to select cases through the existing runners, without additional recipes or CI gate steps.

The Python package has no tests yet. Its pytest configuration recognizes *.test.py/*.spec.py and uses importlib mode, allowing dotted filenames and duplicate basenames in different suites; no empty test gate or new dependency is introduced. [pytest discovery](https://docs.pytest.org/en/stable/example/pythoncollection.html#changing-naming-conventions), [pytest import modes](https://docs.pytest.org/en/stable/explanation/pythonpath.html#import-modes)

Before the first release after this refactor, edit existing migrations directly. Recreate isolated test databases when definitions change; intermediate implementations do not need compatibility migrations.

## Platform artifacts and release

`release.artifacts` selects bundles, the runtime image and testing-torrents independently. Use `just release plan` to inspect the source/config hash and selection; `--format github-output` exports the same data to CI. An empty selection runs verification without publication. These switches do not change test coverage or grant permissions. Bundle publication checks the recorded metadata hash/selection; image publication and final manifests follow the [release delivery contract](005-RELEASE-ACCEPTANCE.md#metadata-selection-and-publication-history).

JPXL, static WebP and aws-lc retain their actual native prerequisites. Codec source stays upstream with pinned Git commits, without source trees in this repository; do not set target-cpu=native. Windows uses static CRT; musl artifacts have no ELF interpreter/NEEDED; GNU records its glibc ABI boundary; macOS permits system dylibs/frameworks only. Targets share build-release/platform-check, and existing artifacts can be checked without rebuilding. See [media and configuration](004-MEDIA-AND-CONFIGURATION.md) for media and parallel execution semantics.

`konobangu-metadata.toml` owns versions, package inventory and release paths. Version updates validate all manifests before synchronizing Rust/Node/Python and lockfiles, restoring files on failure without committing, pushing or tagging. This project is an application: Node packages are private and Rust packages use publish=false.

```text
mise exec -- just release plan
mise exec -- just release version set 0.2.0
mise exec -- just release prepare --target native
```

Local preparation produces a preview archive with local provenance, which cannot be submitted as a publishable CI artifact. Before the first automatic publication, land `release.yaml` on the default branch so GitHub can dispatch it. Push the committed version and changelog to `release` to run verification and metadata-selected publication. For a retry, dispatch the Release workflow with `source_ref=refs/heads/release`, the full tested `source_sha`, `tests_run_id` and exact `tests_run_attempt`; it consumes that candidate rather than the current branch tip. Stable publication requires the configured environment approval. The release orchestrator owns tags, draft assets, downloaded verification, finalization and stable aliases; no separate local tag/upload command bypasses it. See the [release contract](005-RELEASE-ACCEPTANCE.md). Configuration, data and WebUI remain runtime inputs; codecs/TLS have no extra runtime shared libraries.

cargo-about generates native THIRD-PARTY-NOTICES from locked dependencies, supplemented by native attributions; Vite generates WebUI notices from bundled dependencies. Failures stop packaging. Do not track versioned dependency license copies. musl runtime supplements accompany that target only. Platform receipts record platforms, source/dependency commits, commands and exit codes; old receipts do not validate new source. Actual execution evidence is separate from CI configuration. Unfinished-response progressive rendering and model evaluation remain on the [roadmap](roadmap/001-SHORT-TERM-ROADMAP.md).

## Documentation and contributor rules

### Documentation site

`apps/docs` uses stable VitePress and Vue. English convention documents (`README.md`, `CHANGELOG.md`, and CONTRIBUTING/SECURITY/PRIVACY when present) have suffixless repository-root sources; `docs/en` exposes relative file symlinks to them. Other languages keep regular source files with the same names under `docs/{lang}`. Numbered topic documents remain in their language directories. The site's `en` and `zh` directory symlinks project those directories; VitePress maps each README to the locale homepage and each CHANGELOG to its changelog route. Edit the canonical source, not a separate homepage. GitHub edit links resolve to the actual source, and `public/assets` projects repository assets.

Modern operating systems with working symlinks are required for development. On Windows, enable Developer Mode (or grant symlink creation privileges) and clone with `git -c core.symlinks=true clone ...`. Existing checkouts must enable `core.symlinks` and restore the tracked links before building. Do not replace them with copies or junctions. `check docs` deduplicates projections and resolves relative links from the canonical source; the site build also checks projection targets and rendered routes.

Just provides `dev-docs` (loopback:5004), `build-docs` (strict link validation, output in `apps/docs/dist`) and `docs-preview`. `verify ts` includes the site build. VitePress configuration participates in the root TypeScript reference graph; declarations go to `apps/docs/dist-tsc`, and temporary bundler data goes to `apps/docs/.cache`. English routes start at `/`, Chinese routes at `/zh/`; repository-relative links and configuration examples remain usable in both GitHub Markdown and the site.

The Documentation workflow builds and deploys relevant changes on `master`, or can be dispatched manually from `master`. In repository Settings → Pages, select **GitHub Actions**, set the custom domain to `konobangu.apeiraco.com`, and enable HTTPS after DNS verification. The subdomain's DNS CNAME points to `apeiraco.github.io`, without a repository path. `apps/docs/public/CNAME`, sitemap and canonical URLs use the same domain. Pull requests validate through the existing TypeScript verification stage; they do not deploy.

### Contributor rules

AGENTS.md owns core requirements and topic links. Current contracts belong in docs, future plans in the roadmap, history in CHANGELOG and working handoffs/receipts remain outside documentation dependencies. English/Chinese user-facing docs keep bidirectional language links and prefer same-language topics. YAML uses two-space indentation; code/comments use English. Follow AGENTS.md for GraphQL generated fields and skip keys, rerunning codegen and checking generated input types. See [media and configuration](004-MEDIA-AND-CONFIGURATION.md) for configuration naming, units and interpolation, and [tasks and migrations](003-TASK-DELIVERY-AND-MIGRATION.md) for migration/task delivery contracts.

[English](001-DEVELOPMENT-VERIFICATION.md) | [中文](../zh/001-DEVELOPMENT-VERIFICATION.md)
