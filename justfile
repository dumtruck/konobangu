set windows-shell := ["pwsh.exe", "-c"]
set dotenv-load

prepare-dev-recorder:
  cargo install loco-cli
  cargo install sea-orm-cli
  cargo install cargo-watch

dev-webui:
  pnpm run dev

dev-proxy:
  pnpm run --filter=proxy dev

dev-recorder:
  cargo watch -w apps/recorder -i '**/*.{js,css,scss,tsx,ts,jsx,html}' -x 'recorder start'

dev-playground:
  pnpm run --filter=recorder dev

down-recorder:
  cargo run -p recorder --bin recorder_cli -- db down 999 --environment development

play-recorder:
  cargo recorder-playground
