# Changelog

## 0.1.0

### Added

- Optional `auth.provider.trusted_ca_file` for OIDC issuers behind a private certificate authority.
- Bundled OIDC provider with native Basic login, code/PKCE, persistent signing and rotating refresh grants; one TokenSet/Bearer application model for bundled and external issuers.
- Independent auth-provider and shared database/configuration Rust crates, capability-restricted pools and owner-scoped GraphQL transactions.
- Bearer-authenticated native image/RSS grants with local macaroon attenuation, bounded renewal/retry and LAN HTTP Crypto bootstrap.
- Durable task runs/outbox, fenced execution, cancellation recovery and timezone-aware Cron scheduling.
- Native configuration interpolation and file secrets with redacted diagnostics.
- Versioned image derivatives, bounded media execution and provenance-aware backfill.
- Cross-platform development/release tooling and application, migration and platform verification.
- README-backed bilingual VitePress documentation with local search and GitHub Pages deployment at konobangu.apeiraco.com.

### Changed

- Populate CI caches from default-branch owners, reuse tooling through stage dependencies, cache compact third-party Cargo artifacts and share container download caches.
- Upgrade dependencies; adopt injection-js, SecurityDept RxSignal, Apollo and TanStack Table 9.
- Centralize HTTP middleware, one SDK-extending WebUI environment (native web and test hosts) and unique resource ownership; use SDK callback routing.
- Name database privileges by application scope, authentication and task control; bootstrap NOLOGIN capabilities from one database URL, derive grants from Rust migration identifiers and consolidate unreleased migrations.
- Use RFC3339 timestamps and Temporal, loading missing capabilities before startup; target Chrome/Edge 140+, Firefox 140+ and Safari/iOS 18.6+.
- Encode covers with pinned JPXL Balanced 77 and static WebP80 through one bounded executor on an owned Rayon pool. Preserve originals and composite cover transparency on white.
- Preserve generated binding formatting and use native Node ESM path metadata in tooling.
- Use Vite email previews/exports, Playwright-owned fixtures and unit/integration/e2e test directories; share task migration identifiers in Rust.
- Share resource-based verification stages between Just and CI; isolate browser builds and run TypeScript checks/watch through explicit commands over one reference graph.
- Restrict verification to default/release branch pushes and their PRs; expose one aggregate branch-protection check.
- Publish metadata-selected bundles and runtime/fixture images through one tested-source workflow, binding run/attempt, metadata and immutable artifact receipts.
- Consolidate CLI/Just tooling and generated output; restore Zellij sessions, use kebab-case executables, and group configuration by service with explicit units.

### Fixed

- Correct release readiness and automatic-role deployment guidance; retain unfinished delivery as current release obligations.
- Support independent-site WebUI/API with SDK TokenSet boundaries, provider-only native browser credentials, local JWT verification and local sign-out/reconnection; select allowed WebUI origins per request.
- Separate Cron plans from execution fields and retries; remove unused notification machinery.
- Load GraphiQL workers through the official Vite integration and keep worker dependencies in the dev source graph.
- Resolve workspace source during Vite builds so standalone browser fixtures work without prebuilt recorder declarations.
- Preserve endpoint security/cache policies under header presets, and authorization, validators and Range behavior during image negotiation.
- Keep task publication, cancellation, retries and deletion behind transaction and lease fences.
- Install locked CLI dependencies before invoking tools in clean CI checkouts; apply file logging thresholds.
- Preserve manifest line endings in release metadata updates and accept CRLF release notes.
- Handle qBittorrent duplicate-add conflicts through server hash verification; pin the integration image by digest.
- Release media admission budgets before delivering completion, preventing spurious backpressure on subsequent jobs.

### Removed

- Direct Basic API authentication, WebUI Basic contexts/iframe adapters, cookie-session runtime, browser token persistence, private-resource Service Worker/PWA, Tera, Jotai/DI forks and unused date adapters.
- Codec subprocesses, libjxl build machinery and tracked compiler declaration artifacts.

### Upgrade notes

Back up the database and storage, stop writers/workers, and apply owner-only migrations. Tasks queued in the earlier Apalis 0.7 schema are not migrated; drop that schema or recreate the database first. Database rollback requires restoring the backup. Default startup initializes capability-restricted pools from one database URL; optional separate runtime/owner credentials remain available; replace configuration templates, flat auth/task/media fields and legacy JXL options. Automatic covers remain JXL + WebP; JPXL currently produces ordinary single-pass images.

See [authentication](docs/en/002-AUTHENTICATION-DECISION.md), [upgrades](docs/en/003-TASK-DELIVERY-AND-MIGRATION.md), and [media/configuration](docs/en/004-MEDIA-AND-CONFIGURATION.md).

[English](CHANGELOG.md) | [中文](docs/zh/CHANGELOG.md)
