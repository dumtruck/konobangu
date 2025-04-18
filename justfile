set windows-shell := ["pwsh.exe", "-c"]
set dotenv-load := true

prepare-dev-recorder:
    cargo install sea-orm-cli
    cargo install cargo-watch

dev-webui:
    pnpm run --filter=webui dev

dev-proxy:
    pnpm run --filter=proxy dev

# bacon recorder # crash on windows
dev-recorder:
    cargo watch -w "apps/recorder" -x "run -p recorder --bin recorder_cli -- --environment development"
