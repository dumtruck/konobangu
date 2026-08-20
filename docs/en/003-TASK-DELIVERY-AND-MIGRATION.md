# 003 — Task delivery and database upgrade

## Business records and API

The application owns `task_runs` and `task_outbox`. SeaORM/migration 2.0.4 and SQLx 0.9.0 execute business transactions; Seaography 2.0.0-rc.9 supplies schema helpers. Apalis 1.0.0-rc.10 with apalis-postgres 1.0.0-rc.9 owns delivery, worker claiming, heartbeats and orphan recovery. These prereleases are pinned; their public APIs are used without a fork.

Subscriber/system task lists and details read application views over `task_runs`. Creation and retry write the run and outbox in the original GraphQL operation transaction. A later failing root or batch rolls back both; no resolver makes queue network calls. Creation returns a queued task ID, not completion of ingestion. The browser uses Apollo for entity data and native Table 9.2.4 for server pagination, sorting and filtering. Change a sort/filter/page size to return to page zero; selection uses stable business IDs.

Tasks retain Pending, Scheduled, Running, Done, Failed and Killed. Only task commands/executors change state, owner, generation or execution budget. Queue lock/priority implementation fields are removed from task API inputs and outputs. `generation` and `cancelRequestedAt` expose business lifecycle. Subscriber tasks use the authenticated owner; system task creation and management require explicit application administration, regardless of authentication method or historically shared subscribers. Foreign references must belong to the same owner.

Delete is cancellation or archive. Pending/Scheduled become Killed; Running receives a cancellation request. A terminal run is archived and hidden from normal lists. Tombstones remain durable. A cancelled Running run with an expired or absent execution lease atomically becomes Killed, with done_at and cleared token/lease, without another attempt. Cancellation, replay and the existing dispatcher reuse this rule; it does not depend on another envelope. Retry accepts Failed/Killed without a live executor, revalidates references/payload, increments generation, resets attempts and writes a new outbox record. Concurrent retries have one winner and return a conflict for the other caller. Cross-owner denial or conflicts must not display success.

Subscription deletion is an explicit command in the original IdentityOperation. It excludes new task references with a transaction-scoped subscription intent, locks task rows before deleting the parent, requests cancellation of active runs and preserves history/outbox. An already fenced business stage finishes before the delete boundary; subsequent stages reject its cancelled token. The nullable live subscription FK uses ON DELETE SET NULL; payload IDs remain historical only. Retry/replay revalidate resources, and details show deleted references. Cron creation uses a nonblocking intent check and rolls back for the next poll when deletion is in progress.

## GraphQL timestamp format upgrade

The schema context enables Seaography's official `types.timestamp_rfc3339=true`. DateTimeUtc output becomes offset-bearing RFC3339 instead of Chrono display strings, retaining database microseconds. Existing scalar/filter/input types and their parsers are unchanged; there is no custom serializer or database timestamp migration. The current Time date filter still accepts its existing `YYYY-MM-DD HH:mm:ss.f` input; the output setting does not add RFC3339 filter input support. Callers caching or comparing output strings should compare timestamp semantics. The WebUI's shared Temporal.Instant boundary accepts legacy Chrono UTC/offset values without treating offset-free strings as browser-local time. Real ordinary-role GraphQL regressions verify persisted RFC3339/microsecond output and existing date filtering.

## Delivery, retries and fencing

The task_control role claims due outbox rows with `FOR UPDATE SKIP LOCKED`, commits its short transaction, pushes via public TaskSink, then acknowledges in another short transaction. Outbox leases last 60 seconds. Failed delivery retains the row and uses 1, 2, 4, 8, 16, 32, then 60 seconds of backoff. Polling resumes expired claims after restart. The idempotency key is `task_id:generation`; the envelope contains only these two fields. Worker owner and versioned payload are loaded from the trusted run.

Business attempts count actual executions, independently of duplicate queue deliveries. New runs allow three executions including the first, with 5- and 30-second delays. Network failures, HTTP 5xx and recoverable database errors can retry; 4xx, invalid inputs and ownership failures cannot. Queue delivery has a separate high ceiling so duplicates do not consume this business budget. Application errors remain errors to the public worker middleware. Shutdown interrupts retry waits, leaves durable state, and stops dispatcher/scheduler polling.

A claim replaces the execution UUID token through generation/token CAS. The business lease is 120 seconds with 30-second renewal. An expired claim can be taken over. Every private database stage binds the trusted subscriber and checks the current, unexpired, uncancelled token before commit. Old generations, terminal replays and stale tokens cannot publish new results. File publication stages a temporary object and atomically renames it under the short fence lock. Association/source keys use uniqueness and upserts. No transaction spans external HTTP. Renewal SQL waits do not stop polling execution. Cancellation, renewal errors/lease loss and shutdown stop the losing future before terminal writes. In-flight PostgreSQL statements receive token-scoped cancellation through short-lived connections using the same ordinary business/scheduler roles; transaction-local application_name tags prevent cancellation of unrelated queries. These control connections close explicitly, add no permanent pool or privileges, and leave the identity 3+2 budget unchanged. Service shutdown signals and joins the worker and polling loops.

External reads may repeat. Previously committed valid stages or already issued requests cannot be withdrawn by later cancellation. This is durable delivery with fenced, repeat-safe business stages; it does not promise exactly-once external effects.

## Cron

Cron is application-owned and uses croner 4 plus chrono-tz. One short transaction locks a due plan, creates task/outbox and advances `next_run`; `(cron_id, scheduled_at_utc)` is unique. Polling is the correctness source, with no dependence on NOTIFY delivery. Scheduler connections cannot access private business or identity tables.

Times persist in UTC and are calculated in the configured IANA zone. A repeated DST wall time produces two UTC occurrences; a nonexistent wall time is skipped. Downtime coalesces missed occurrences into one run and moves the plan beyond the current time. If the same Cron still has an active run, that occurrence is skipped with a recorded reason, without an accumulating backlog. Disabling/editing a plan changes future eligibility. Database failure rolls back the occurrence, outbox and plan advance together.

## Owner-only migration and recovery

A new database initializes automatically at startup by default. Runtime current_user must be restricted to capability roles rather than retain ownership privileges; the shared database.url bootstrap login may hold migration privileges. Set `database.migration.auto_run=false` for operator-controlled migration and run the owner entry explicitly. See [authentication and roles](002-AUTHENTICATION-DECISION.md).

```sh
mise exec -- cargo run -p recorder --locked --bin migrate-up -- --environment production --config-file /path/to/recorder.config.toml
```

Owner migration prefers `database.migration.url`; otherwise the supplied `database.url` must be an owner connection. Stages are resumable, not one cross-component atomic commit:

| Stage | Result |
| --- | --- |
| Application transaction | Identity migration; create `task_runs`, `task_outbox`, their policies and the task views |
| Queue initialization | Public PostgresStorage migrator explicitly creates `apalis` and writes history to `apalis._sqlx_migrations` |
| Access | NOLOGIN capability roles, grants and configured login memberships |

The application never creates, imports or archives a queue schema of its own: `apalis` belongs entirely to the upstream migrator, and task history lives only in `task_runs`. A database that still carries objects from a pre-`task_runs` queue is not upgraded in place; drop those objects or recreate the database before migrating. Set the exact known `database.migration.legacy_oidc_issuer` when existing OIDC identities need attribution; no owner is inferred from names/email.

Stop writes before migrating an existing database and back it up first. If any stage fails, keep writes stopped, repair the reported issue and rerun up. app_scoped_access has subscriber-bound task/outbox access; task_control_access has cross-subscriber queue, Cron and task/outbox access; auth_access has no private application, task or queue access; auth_provider_access owns only bundled protocol state. See [role and pool contracts](002-AUTHENTICATION-DECISION.md).

Rollback is stopped-write restoration of the backup, storage and matching release/configuration. Down explicitly rejects the irreversible identity/task migrations.

## Related contracts

Configuration/interpolation/secrets and media capabilities: [media and configuration](004-MEDIA-AND-CONFIGURATION.md). Identity pools and capability roles: [authentication](002-AUTHENTICATION-DECISION.md). Toolchain, platform builds and checks: [development verification](001-DEVELOPMENT-VERIFICATION.md).

Task-delivery migration tables and columns use Rust identifiers in migrations/defs.rs. SeaQuery handles schema/query operations it can express; PostgreSQL-specific policies and partial indexes interpolate those identifiers in Rust.

[English](003-TASK-DELIVERY-AND-MIGRATION.md) | [中文](../zh/003-TASK-DELIVERY-AND-MIGRATION.md)
