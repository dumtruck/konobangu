set windows-shell := ["pwsh.exe", "-c"]
set dotenv-load := true

prepare-dev-recorder:
    cargo install sea-orm-cli
    cargo install cargo-watch

dev-webui:
    pnpm run --filter=webui dev

dev-proxy:
    pnpm run --filter=proxy dev

# crash on windows
dev-recorder:
    watchexec -r -w apps/recorder -- cargo run -p recorder --bin recorder_cli -- --environment development

dev-deps:
    docker compose -f devdeps.compose.yaml up

dev-deps-clean:
    docker compose -f devdeps.compose.yaml down -v

dev-codegen:
    pnpm run --filter=webui codegen

dev-all:
    zellij --layout dev.kdl