# Konobangu Testing Torrents

Internal HTTP tracker, web seed and torrent fixture for downloader tests. Its isolated pnpm workspace owns its lockfile and native dependency build policy.

From the repository root, `mise exec -- just setup` installs frozen dependencies; `mise exec -- just dev-torrents` runs the local service. The test framework owns service startup and cleanup during normal verification.

For a local container preview:

```sh
docker buildx build --platform linux/amd64 --tag konobangu-testing-torrents:local --load packages/testing-torrents
docker run --rm -p 6080:6080 -p 6081:6081 -p 6082:6082 konobangu-testing-torrents:local
```

Publication is selected by `release.artifacts.testing_torrents_image` in the root metadata and performed by the application Release workflow. That orchestrator verifies the candidate, records its digest and promotes stable aliases. See the [release contract](../../docs/en/005-RELEASE-ACCEPTANCE.md) · [中文](../../docs/zh/005-RELEASE-ACCEPTANCE.md) and [development rules](../../docs/en/001-DEVELOPMENT-VERIFICATION.md) · [中文](../../docs/zh/001-DEVELOPMENT-VERIFICATION.md).
