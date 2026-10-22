//! ARAATAN v0.6.1 — CLI Entry Point
use std::env;
use anyhow::Result;
use araatan::self;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    araatan::init_logging();

    tracing::info!("{} v{} starting", araatan::NAME, araatan::VERSION);

    // Load configuration
    let cfg = araatan::load_config();
    tracing::info!(
        "Symbols: {} | Timeframe: {}",
        cfg.system.symbols.join(", "),
        cfg.system.timeframe
    );
    tracing::info!("Market Data Provider: TickerAll ({})", cfg.tickerall.base_url);

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    // Execute command
    match cmd {
        "check" => {
            araatan::check_health(&cfg).await;
        }
        "fetch" => {
            let symbol = args.get(2).map(|s| s.as_str()).unwrap_or("XAUUSD");
            let hours: u32 = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(24);
            araatan::cmd_fetch(&cfg, symbol, hours).await?;
        }
        "warmup" => {
            let symbol = args.get(2).map(|s| s.as_str()).unwrap_or("XAUUSD");
            let target_bars = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(cfg.tickerall.warmup_bars);
            araatan::cmd_warmup(&cfg, symbol, target_bars).await?;
        }
        "session" => {
            araatan::cmd_start_session(&cfg).await?;
        }
        "live" => {
            araatan::cmd_live(&cfg).await?;
        }
        "forecast" => {
            let symbol = args.get(2).map(|s| s.as_str()).unwrap_or("XAUUSD");
            let model_path = args.get(3).map(|s| s.as_str()).unwrap_or("chronos-2.gguf");
            let horizon: usize = args.get(4).and_then(|s| s.parse().ok()).unwrap_or(12);
            araatan::cmd_forecast(&cfg, symbol, model_path, horizon).await?;
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
        "help" | "--help" | "-h" => araatan::print_help(),
        "version" | "--version" | "-V" => println!("{} v{}", araatan::NAME, araatan::VERSION),
        other => {
            tracing::warn!("Unknown command: {}", other);
            araatan::print_help();
        }
    }

    Ok(())
}
