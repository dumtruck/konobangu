set windows-shell := ["pwsh.exe", "-c"]
set dotenv-load

prepare-dev-recorder:
  cargo install loco-cli
  cargo install sea-orm-cli
  cargo install cargo-watch

dev-webui:
  pnpm run dev

dev-recorder:
  cargo watch -w apps/recorder -x 'recorder start'

down-recorder:
  cargo run -p recorder --bin recorder_cli -- db down 999 --environment development

play-recorder:
  cargo recorder-playground