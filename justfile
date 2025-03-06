set windows-shell := ["pwsh.exe", "-c"]
set dotenv-load

prepare-dev-recorder:
  cargo install sea-orm-cli
  cargo install cargo-watch

dev-webui:
  pnpm run --filter=webui dev

dev-proxy:
  pnpm run --filter=proxy dev

dev-recorder:
  bacon recorder

