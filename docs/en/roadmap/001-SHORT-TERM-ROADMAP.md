# 001 — Short-term roadmap

This page owns future decisions and acceptance conditions. Current contracts live in [authentication](../002-AUTHENTICATION-DECISION.md), [tasks and upgrades](../003-TASK-DELIVERY-AND-MIGRATION.md), and [media/configuration](../004-MEDIA-AND-CONFIGURATION.md). Delivered changes belong in [CHANGELOG](../../../CHANGELOG.md).

## Current delivery and future scope

Bundled OIDC and unified authentication passed current-host behavioral revalidation; runtime-image acceptance and public distribution remain current commitments in [release delivery and acceptance](../005-RELEASE-ACCEPTANCE.md), not future roadmap work. The maintainer must publish the exact committed and verified candidate to close delivery.

## Remaining work

| Item | Direction | Acceptance |
| --- | --- | --- |
| R6 Model tooling | Exercise actual Python/Animeta model loading, inference, datasets and resources; see the [Animeta proposal](002-ANIMETA-MODEL.md) | Lockfile updates do not establish working inference; require reproducible samples, quality and resource budgets |
| R7 Awa replacement | Awa is the chosen candidate; wait for its runtime and official SeaORM adapter to support SQLx 0.9 and the project ORM together | A successful experiment must lead to an independently accepted production migration |
| R8 Progressive JXL | Retain static JXL defaults; demonstrate recognizable previews and subsequent refinements while the response is unfinished | Capture the actual production HTTP/page path; complete-file decoding is insufficient |
| Stable Rust | Replace the pinned nightly once upstream quirks_path no longer requires it | Pass default, all valid feature combinations and trimmed branches on Windows/macOS/Linux |

## R7 decisions

The Awa experiment preserves the accepted Apalis contracts. Verify published compatibility and licensing before starting; do not integrate speculative future versions.

1. Use public transaction APIs to prove enqueue/rollback, Cron occurrences and business publication in the same real SeaORM/SQLx transaction. Another connection or private storage table is not an acceptable substitute.
2. Preserve app-scoped, auth and task-control capabilities and the owner-only migration phase. Cover rollback after later mutation roots, cross-owner denial, and retry/cancel/delete transactions.
3. Exercise real workers with duplicate delivery, renewal, executor loss, cancellation recovery, attempt budgets, late results, resource deletion, restart and multi-instance Cron. Preserve timezone/DST, downtime coalescing and no-overlap behavior.
4. Identify lease/retry/recovery code that can be deleted and business fences, history and recovery that must remain. Reject the experiment if it needs a duplicate full executor or weakens correctness.
5. After success, complete runtime/lifecycle/roles, APIs/pages, codegen, CI, documentation and old-code removal in the same delivery. Migrate nonempty databases through stopped-write/drain/import and rehearsed backup restoration, retaining IDs, owners, history and terminal states. Never reset data or consume both queues concurrently.

## R8 decisions

Honor client Accept/q/exclusions and prefer eligible progressive JXL → ordinary JXL → WebP → original. Features take priority over small size differences; retain one fixed encode per profile without size-driven retries or downgrade. Do not enable automatic AVIF. Preserve originals, authorization and publication fences. Static defaults do not depend on the progressive gate, and static acceptance does not close R8.

Controlled native Linux or SSH verification is valid evidence. CI configuration and actual CI execution are recorded separately. Organize future work as complete delivery themes rather than numerous small experiment/implementation/cleanup rounds.

[English](001-SHORT-TERM-ROADMAP.md) | [中文](../../zh/roadmap/001-SHORT-TERM-ROADMAP.md)
