# AI Agent Handover & Project Context: Araatan

## 🤖 Role of the AI
You are an expert Rust developer assisting the user (Manager) in building Araatan. The user does not write code; you must write, debug, and structure the code. The user will copy-paste prompts to you via Aider.

## 🏗️ 1. Project Overview
Araatan is an "Adaptive Risk-Aware Algorithmic Trading System". It fetches market data (Forex, Crypto, Commodities) from TickerAll API, runs zero-shot forecasting models (Chronos-2, Toto-2), and makes trading decisions.

## ⚙️ 2. Environment & Workflow (CRITICAL)
- **OS**: Android using Termux.
- **AI Tooling**: Aider with `mistral/codestral-latest` model.
- **Termux Cargo Issue**: Android restricts `~/.cache`. You MUST always use these environment variables when running Cargo commands:
  `!CARGO_TARGET_DIR=./target CARGO_HOME=./.cargo_home cargo check` (or `cargo run`).
- **ARM FP16 Issue**: We enabled `+fp16` in `.cargo/config.toml` to fix `gemm-f16` compilation on Helio G99.
- **Git**: A `post-commit` hook automatically pushes to GitHub. After a successful edit, use `/commit` and it will push.
- **Reading External Files**: Aider cannot read files outside `~/Araatan`. If you need to reference `~/zsfm-rs`, use `cp` to copy them into `temp_refs/` inside the project, read them, and then `rm -rf temp_refs`.

## 🏛️ 3. Current Architecture & Status
- **Android-First**: The final goal is an Android APK (using Tauri or Dioxus). **DO NOT use external CLI binaries**. All model inference MUST be in-process.
- **Library Structure**: 
  - `src/main.rs`: Thin CLI wrapper. Only parses args and calls `src/lib.rs`.
  - `src/lib.rs`: Core library. Contains `match cmd` block and functions like `cmd_fetch`, `cmd_session`, `cmd_symbols`.
  - `src/tickerall.rs`: TickerAll API Client. Handles session, fetching candles, and fetching symbols. It is robust against missing fields (uses `serde_json::Value` where necessary).
  - `src/forecast.rs`: Contains `ForecastEngine`. Currently, it only loads and runs **Chronos-2**.
  - `src/config.rs`: Configuration (`araatan.toml`).
- **Broker**: XM (MetaTrader 5). The `GOLD` symbol is used for Gold (not XAUUSD). Broker server format: `XMGlobal-MT5 5`.
- **Models**: Chronos-2 (120M Q8) and Toto-2 (313M Q8) are downloaded locally (`chronos-2-q8.gguf` and `toto-2-313m-q8.gguf`).

## 🧠 4. Machine Learning Models (zsfm-rs)
- We use a local fork of `zsfm-rs` at `~/zsfm-rs`. It is integrated as a path dependency in `Cargo.toml`.
- **Multivariate**: Input format to the models is `[batch][variate][time]`, where variates are Open, High, Low, Close.
- **OHLC Post-Processing**: After inference, you MUST fix High/Low:
  `High = max(Open, High, Close)`
  `Low = min(Open, Low, Close)`

## 🛡️ 5. Security
- `araatan.toml` contains API keys and passwords. It is listed in `.gitignore`. NEVER commit it.
- Never hardcode API keys in `src/`.
- The user leaked their MT5 password and API key in chat. Remind them to change it!

## 📝 6. Next Steps (TODO)
1. **Integrate Toto-2**: Update `src/forecast.rs` to load `toto-2-313m-q8.gguf` alongside Chronos-2.
2. **Ensemble Logic**: Average the predictions of Chronos-2 and Toto-2 in `ForecastEngine::predict_ensemble()`. Include the OHLC post-processing.
3. **Add Indicators & Meta-Reasoning**: Integrate technical indicators (RSI, MACD) and use TabICL or an LLM to explain *why* the ensemble made a specific prediction.
4. **Android UI**: Prepare the project for Tauri/Dioxus.
