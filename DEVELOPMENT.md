# Araatan Development Context & Workflow

## 1. Environment Setup (Termux)
This project is being developed on Android using Termux. Due to Android's strict file permissions (specifically for `~/.cache`), standard Cargo commands will fail.
Always use the following environment variables when running Cargo:
`CARGO_TARGET_DIR=./target CARGO_HOME=./.cargo_home cargo check` (or cargo run).

## 2. AI Development Workflow
- We use **Aider** with the **Mistral** model (`mistral/codestral-latest`) for AI-assisted coding.
- A `post-commit` hook is installed to automatically push to GitHub after every commit.
- Start Aider using: `aider --model mistral/codestral-latest`

## 3. Current Architecture
- `src/main.rs`: Just a CLI wrapper. It parses arguments and calls functions from `lib.rs`.
- `src/lib.rs`: The core library. **WARNING:** Currently, functions like `check_health`, `cmd_fetch`, `cmd_warmup`, etc., are just stubs (placeholders using `tracing::info!`) and DO NOT contain the actual TickerAll API logic yet.
- `src/config.rs`: Configuration handling.
- `src/tickerall.rs`: TickerAll API client definitions.

## 4. Next Steps (TODO)
- Implement the real TickerAll API logic inside `src/lib.rs` (replacing the stubs).
- Prepare the project for an Android UI (using Tauri or Dioxus).
