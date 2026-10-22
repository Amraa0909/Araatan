# AI Agent Handover & Project Context: Araatan

## 🤖 Role of the AI
You are an expert Rust developer assisting the user (Manager) in building Araatan. The user does not write code; you must write, debug, and structure the code. The user will copy-paste prompts to you via Aider.

## 🏗️ 1. Project Overview
Araatan is an "Adaptive Risk-Aware Algorithmic Trading System". It fetches market data (Forex, Crypto, Commodities) from TickerAll API, runs zero-shot multivariate forecasting models (Chronos-2, Toto-2), and makes trading decisions.

## ⚙️ 2. Environment & Workflow (CRITICAL)
- **OS**: Android using Termux.
- **AI Tooling**: Aider with `mistral/codestral-latest` model.
- **Termux Cargo Issue**: Android restricts `~/.cache`. You MUST always use these environment variables when running Cargo commands:
  `!CARGO_TARGET_DIR=./target CARGO_HOME=./.cargo_home cargo check` (or `cargo run`).
- **Git**: A `post-commit` hook automatically pushes to GitHub. After a successful edit, use `/commit` and it will push.

## 🏛️ 3. Current Architecture Rules
- **Android-First**: The final goal is an Android APK (using Tauri or Dioxus). Therefore, **DO NOT use external CLI binaries** (e.g., spawning `zsfm` via `Command::new`). All model inference MUST be in-process.
- **Library Structure**: 
  - `src/main.rs`: Thin CLI wrapper. Only parses args and calls `src/lib.rs`.
  - `src/lib.rs`: Core library. Currently contains function stubs (`check_health`, `cmd_fetch`, etc.) that need actual implementation.
  - `src/config.rs`: Configuration (`araatan.toml`).
  - `src/tickerall.rs`: API Client.
- **Dynamic Symbols**: Hardcoded symbols must be replaced. The system should fetch available symbols, score them by liquidity/volatility, and select the top N.

## 🧠 4. Machine Learning Models (zsfm-rs)
- We use a local fork of `zsfm-rs` at `../zsfm-rs`.
- **Models**: Chronos-2 (120M) and Toto-2 (313M).
- **Multivariate**: Input format is `[batch][variate][time]`. We feed multiple variates (Open, High, Low, Close, Volume).
- **OHLC Post-Processing**: Chronos/Toto do not know financial rules. After inference, you MUST fix High/Low:
  `High = max(Open, High, Close)`
  `Low = min(Open, Low, Close)`
- **Integration**: `zsfm_core` must be added as a path dependency in `Cargo.toml`.

## 🛡️ 5. Security
- `araatan.toml` contains API keys and is listed in `.gitignore`. NEVER commit it.
- Never hardcode API keys in `src/`.

## 📝 6. Next Steps (TODO)
1. Add `zsfm_core` to `Cargo.toml`.
2. Create `src/forecast.rs` with a `ForecastEngine` struct that loads Chronos-2 and Toto-2.
3. Implement real API logic inside `src/lib.rs` stubs (e.g., `cmd_fetch` should actually fetch data using `TickerallClient`).
