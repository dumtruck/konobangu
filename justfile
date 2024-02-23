set windows-shell := ["pwsh.exe", "-c"]
set dotenv-load

prepare-dev-recorder:
  cargo install loco-cli
  cargo install sea-orm-cli

dev-recorder:
  cargo watch -w crates/recorder -w config -x 'recorder start'

play-recorder:
  cargo recorder-playground

dev-webui:
  npm run dev:webui

dev-proxy:
  npm run dev:proxy