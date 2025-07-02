set windows-shell := ["pwsh.exe", "-c"]
set dotenv-load := true

clean-cargo-incremental:
    # https://github.com/rust-lang/rust/issues/141540
    rm -r target/debug/incremental

prepare-dev:
    cargo install cargo-binstall
    cargo binstall sea-orm-cli cargo-llvm-cov cargo-nextest
    # <package-manager> install watchexec just zellij nasm libjxl netcat

prepare-dev-testcontainers:
    docker pull linuxserver/qbittorrent:latest
    docker pull ghcr.io/dumtruck/konobangu-testing-torrents:latest
    docker pull postgres:17-alpine

export-recorder-ts-bindings:
    cargo test export_bindings -p recorder

dev-webui:
    pnpm run --filter=webui dev

prod-webui:
    pnpm run --filter=webui build
    mkdir -p apps/recorder/webui
    cp -r apps/webui/dist/* apps/recorder/webui/

dev-proxy:
    npx --yes kill-port --port 8899,5005
    pnpm run --parallel --filter=proxy dev

dev-recorder:
    watchexec -r -e rs,toml,yaml,json -- cargo run -p recorder --bin  recorder_cli -- --environment=development --graceful-shutdown=false

prod-recorder: prod-webui
    cargo run --release -p recorder --bin recorder_cli -- --environment=production --working-dir=apps/recorder --graceful-shutdown=false

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
