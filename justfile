set windows-shell := ["pwsh.exe", "-c"]
set dotenv-load := true

prepare-dev:
    cargo install cargo-binstall
    cargo binstall sea-orm-cli cargo-llvm-cov cargo-nextest
    # <package-manager> install watchexec just zellij

prepare-dev-testcontainers:
    docker pull linuxserver/qbittorrent:latest
    docker pull ghcr.io/dumtruck/konobangu-testing-torrents:latest
    docker pull postgres:17-alpine

dev-optimize-images:
    npx -y zx apps/recorder/examples/optimize_image.mjs

dev-webui:
    pnpm run --filter=webui dev

dev-proxy:
    npx --yes kill-port --port 8899,5005
    pnpm run --parallel --filter=proxy dev

dev-recorder:
    watchexec -r -e rs,toml,yaml,json,env -- cargo run -p recorder --bin recorder_cli -- --environment development

dev-recorder-migrate-down:
    cargo run -p recorder --bin migrate_down -- --environment development

dev-deps:
    docker compose -f devdeps.compose.yaml up

dev-deps-clean:
    docker compose -f devdeps.compose.yaml down -v

dev-codegen:
    pnpm run --filter=webui codegen

dev-codegen-wait:
    @until nc -z localhost 5001; do echo "Waiting for Recorder..."; sleep 1; done
    pnpm run --filter=webui codegen-watch

dev-coverage:
    cargo llvm-cov test --html

[unix]
dev-all:
    zellij --layout dev.kdl

[windows]
dev-all:
    @echo "zellij is not supported on Windows, please use vscode tasks 'dev-all'"
