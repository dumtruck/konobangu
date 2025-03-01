set windows-shell := ["pwsh.exe", "-c"]
set dotenv-load

prepare-dev-recorder:
  cargo install sea-orm-cli
  cargo install cargo-watch

dev-webui:
  pnpm run dev

dev-proxy:
  pnpm run --filter=proxy dev

dev-recorder:
  bacon recorder

dev-playground:
  pnpm run --filter=recorder dev

play-recorder:
  cargo recorder-playground
