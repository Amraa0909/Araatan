//! ARAATAN v0.6.1 — Entry point
//! Adaptive Risk-Aware Algorithmic Trading System
//! Multi-symbol M5 runner with TickerAll.com market data feed

mod config;
mod tickerall;

use std::env;
use anyhow::Result;
use config::AppConfig;
use tickerall::TickerallClient;

const VERSION: &str = "0.6.1";
const NAME: &str = "ARAATAN";

#[tokio::main]
async fn main() -> Result<()> {
    // Logging initialization
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tracing::info!("{} v{} starting", NAME, VERSION);

    let cfg = match AppConfig::load() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to load araatan.toml ({}), using default configuration", e);
            AppConfig::load().unwrap_or_default()
        }
    };

    tracing::info!(
        "Symbols: {} | Timeframe: {}",
        cfg.system.symbols.join(", "),
        cfg.system.timeframe
    );
    tracing::info!("Market Data Provider: TickerAll ({})", cfg.tickerall.base_url);

    let args: Vec<String> = env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match cmd {
        "check" => {
            check_health(&cfg).await;
        }
        "fetch" => {
            let symbol = args.get(2).map(|s| s.as_str()).unwrap_or("XAUUSD");
            let hours: u32 = args
                .get(3)
                .and_then(|s| s.parse().ok())
                .unwrap_or(24);
            cmd_fetch(&cfg, symbol, hours).await?;
        }
        "warmup" => {
            let symbol = args.get(2).map(|s| s.as_str()).unwrap_or("XAUUSD");
            let target_bars = args
                .get(3)
                .and_then(|s| s.parse().ok())
                .unwrap_or(cfg.tickerall.warmup_bars);
            cmd_warmup(&cfg, symbol, target_bars).await?;
        }
        "session" => {
            cmd_start_session(&cfg).await?;
        }
        "live" => {
            cmd_live(&cfg).await?;
        }
        "backtest" => {
            tracing::info!("Backtest mode — Scaffold ready for historical TickerAll data");
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

    Ok(())
}

fn print_help() {
    println!(
        "{name} v{ver}

USAGE:
    araatan <COMMAND> [OPTIONS]

COMMANDS:
    fetch [SYMBOL] [HOURS]   Fetch OHLC candles from TickerAll (default: XAUUSD 24)
    warmup [SYMBOL] [BARS]   Fetch warmup buffer (600 M5 bars = 50h) from TickerAll
    session                  Start a broker session on TickerAll and retrieve accountId
    check                    Health check (config, GGUF, TickerAll API connectivity)
    live                     Start live inference runner (M5)
    backtest                 Run historical backtest on M5 bars
    help                     Show this help
    version                  Show version

ENV:
    TICKERALL_API_KEY        TickerAll Bearer API key (cf_api_...)
    TICKERALL_ACCOUNT_ID     TickerAll MetaTrader account ID
    TICKERALL_BASE_URL       API endpoint (default: https://api.tickerall.com)
    RUST_LOG                 Log level (default: info)
",
        name = NAME,
        ver = VERSION
    );
}

async fn check_health(cfg: &AppConfig) {
    tracing::info!("Running system & TickerAll health check…");

    // 1. Config file check
    let cfg_path = std::path::Path::new("araatan.toml");
    if cfg_path.exists() {
        tracing::info!("araatan.toml found ✓");
    } else {
        tracing::warn!("araatan.toml missing — using defaults");
    }

    // 2. Models dir check
    let model_dir = std::path::Path::new("models");
    if model_dir.exists() {
        tracing::info!("models/ directory found ✓");
    } else {
        tracing::warn!("models/ directory missing (create models/ and place Chronos/Toto/TabICL GGUF)");
    }

    // 3. TickerAll API credentials check
    let client = TickerallClient::new(cfg.tickerall.to_tickerall_config());
    if client.has_api_key() {
        tracing::info!("TickerAll API Key configured ✓");
    } else {
        tracing::warn!("TickerAll API Key not configured (set TICKERALL_API_KEY or araatan.toml)");
    }

    if client.has_account_id() {
        tracing::info!("TickerAll Account ID configured ✓");
    } else {
        tracing::info!("TickerAll Account ID not set (can run 'araatan session' to create or use public feed)");
    }

    // 4. Test connectivity
    tracing::info!("Testing connectivity to TickerAll ({})…", cfg.tickerall.base_url);
    match client.fetch_candles("XAUUSD", "M5", 1).await {
        Ok(bars) => {
            tracing::info!(
                "TickerAll connectivity OK ✓ — Retrieved {} recent M5 bar(s) for XAUUSD",
                bars.len()
            );
        }
        Err(e) => {
            tracing::warn!("TickerAll test request returned: {}", e);
        }
    }

    tracing::info!("Health check complete");
}

async fn cmd_fetch(cfg: &AppConfig, symbol: &str, hours: u32) -> Result<()> {
    tracing::info!("Fetching {} M5 candles (lookback: {} hours) from TickerAll…", symbol, hours);
    let client = TickerallClient::new(cfg.tickerall.to_tickerall_config());

    let bars = client.fetch_candles(symbol, "M5", hours).await?;
    tracing::info!("Successfully received {} bars for {}", bars.len(), symbol);

    if let Some(first) = bars.first() {
        tracing::info!(
            "Earliest bar: ts={} open={:.2} close={:.2}",
            first.ts_utc, first.open, first.close
        );
    }
    if let Some(last) = bars.last() {
        tracing::info!(
            "Latest bar:   ts={} open={:.2} close={:.2}",
            last.ts_utc, last.open, last.close
        );
    }

    Ok(())
}

async fn cmd_warmup(cfg: &AppConfig, symbol: &str, target_bars: usize) -> Result<()> {
    tracing::info!(
        "ARAATAN Warmup: Fetching {} M5 bars (~{:.1} hours) for {} from TickerAll…",
        target_bars,
        (target_bars as f64) / 12.0,
        symbol
    );
    let client = TickerallClient::new(cfg.tickerall.to_tickerall_config());

    let bars = client.fetch_warmup_bars(symbol, target_bars).await?;
    let count = bars.len();

    tracing::info!("Warmup buffer populated with {} / {} bars", count, target_bars);
    if count >= target_bars {
        tracing::info!("Warmup status: [WARM_OK] ✓ — Model context ready for inference");
    } else {
        tracing::warn!(
            "Warmup status: [WARM_PARTIAL] — Got {} bars, model context may need more history",
            count
        );
    }

    Ok(())
}

async fn cmd_start_session(cfg: &AppConfig) -> Result<()> {
    let broker = cfg.tickerall.broker.as_deref().unwrap_or("mt5");
    let server = cfg.tickerall.server.as_deref().unwrap_or("");
    let account = cfg.tickerall.account.unwrap_or(0);
    let password = cfg.tickerall.password.as_deref().unwrap_or("");

    if server.is_empty() || account == 0 || password.is_empty() {
        tracing::error!("Broker details missing in araatan.toml [tickerall] (server, account, password required)");
        return Ok(());
    }

    tracing::info!("Initiating broker session on TickerAll: {} server={}", broker, server);
    let mut client = TickerallClient::new(cfg.tickerall.to_tickerall_config());
    let account_id = client.start_session(broker, server, account, password).await?;

    tracing::info!("Broker session established ✓ AccountId: {}", account_id);
    tracing::info!("Save this accountId to araatan.toml or export TICKERALL_ACCOUNT_ID={}", account_id);

    Ok(())
}

async fn cmd_live(cfg: &AppConfig) -> Result<()> {
    tracing::info!("Starting ARAATAN Live Runner (TickerAll feed)");
    tracing::info!("Symbols: {}", cfg.system.symbols.join(", "));
    tracing::info!("Timeframe: M5 (300,000 ms)");

    let client = TickerallClient::new(cfg.tickerall.to_tickerall_config());

    for symbol in &cfg.system.symbols {
        tracing::info!("Warming up {} buffer…", symbol);
        match client.fetch_warmup_bars(symbol, cfg.tickerall.warmup_bars).await {
            Ok(bars) => tracing::info!("{} warmed up with {} bars ✓", symbol, bars.len()),
            Err(e) => tracing::warn!("Warmup failed for {}: {}", symbol, e),
        }
    }

    tracing::info!("Entering M5 bar monitoring loop (Press Ctrl+C to stop)…");
    // Scaffold loop: In full live execution, sleeps until the next M5 boundary
    Ok(())
}
