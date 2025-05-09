set windows-shell := ["pwsh.exe", "-c"]
set dotenv-load := true

prepare-dev-recorder:
    cargo install sea-orm-cli cargo-llvm-cov cargo-nextest
    # install watchexec

prepare-dev-testcontainers:
    docker pull linuxserver/qbittorrent:latest
    docker pull ghcr.io/dumtruck/konobangu-testing-torrents:latest
    docker pull postgres:17-alpine

dev-webui:
    pnpm run --filter=webui dev

dev-proxy:
    pnpm run --filter=proxy dev

dev-recorder:
    watchexec -r -w apps/recorder -- cargo run -p recorder --bin recorder_cli -- --environment development

dev-recorder-migrate-down:
    cargo run -p recorder --bin migrate_down -- --environment development

dev-deps:
    docker compose -f devdeps.compose.yaml up

dev-deps-clean:
    docker compose -f devdeps.compose.yaml down -v

dev-codegen:
    pnpm run --filter=webui codegen

dev-all:
    zellij --layout dev.kdl

dev-codegen-wait:
    @until nc -z localhost 5001; do echo "Waiting for Recorder..."; sleep 1; done
    pnpm run --filter=webui codegen-watch

dev-coverage:
    cargo llvm-cov test --html

