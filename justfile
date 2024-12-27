set windows-shell := ["pwsh.exe", "-c"]
set dotenv-load

prepare-dev-recorder:
  cargo install loco-cli
  cargo install sea-orm-cli
  cargo install cargo-watch

dev-recorder:
  cargo watch -w crates/recorder -w config -x 'recorder start'

down-recorder:
  cargo run -p recorder --bin recorder_cli -- db down 999 --environment recorder.development

play-recorder:
  cargo recorder-playground

dev-webui:
  npm run dev:webui

dev-proxy:
  npm run dev:proxy