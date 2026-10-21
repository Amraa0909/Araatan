//! ARAATAN v0.6.1 — Entry point
//! Adaptive Risk-Aware Algorithmic Trading System
//! XAUUSD & USDJPY on M5 · zsfm-rs · GGUF Q8_0 · candle

use std::env;

const VERSION: &str = "0.6.1";
const NAME: &str = "ARAATAN";

fn main() {
    // Logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("{} v{} starting", NAME, VERSION);
    tracing::info!("Symbols: XAUUSD, USDJPY | Timeframe: M5");
    tracing::info!("Model runtime: zsfm-rs | Quantization: Q8_0");

    // CLI argument parsing (minimal; will become clap-driven)
    let args: Vec<String> = env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match cmd {
        "check" => {
            tracing::info!("Running health check…");
            check_health();
        }
        "live" => {
            tracing::info!("Live mode — NOT yet wired (workspace scaffold pending)");
        }
        "backtest" => {
            tracing::info!("Backtest mode — NOT yet wired (workspace scaffold pending)");
        }
        "convert" => {
            tracing::info!("GGUF convert — delegated to zsfm-rs CLI");
        }
        "gguf-info" => {
            tracing::info!("GGUF info — delegated to zsfm-rs CLI");
        }
        "help" | "--help" | "-h" => print_help(),
        "version" | "--version" | "-V" => println!("{} v{}", NAME, VERSION),
        other => {
            tracing::warn!("Unknown command: {}", other);
            print_help();
        }
    }
}

fn print_help() {
    println!(
        "{name} v{ver}

USAGE:
    araatan <COMMAND>

COMMANDS:
    live         Start live inference runner (M5)
    backtest     Run historical backtest on M5 bars
    replay       Replay audit log
    convert      Convert HF checkpoint → GGUF Q8_0
    gguf-info    Print GGUF metadata
    check        Health check (config, GGUF, device)
    help         Show this help
    version      Show version

ENV:
    RUST_LOG     Log level (default: info)
",
        name = NAME,
        ver = VERSION
    );
}

fn check_health() {
    // Placeholder — full checks come after workspace scaffold
    let model_dir = std::path::Path::new("models");
    if model_dir.exists() {
        tracing::info!("models/ directory found ✓");
    } else {
        tracing::warn!("models/ directory missing — create it and place GGUF files");
    }

    let cfg = std::path::Path::new("araatan.toml");
    if cfg.exists() {
        tracing::info!("araatan.toml found ✓");
    } else {
        tracing::warn!("araatan.toml missing — copy the example from README");
    }

    tracing::info!("Health check complete (scaffold mode)");
}