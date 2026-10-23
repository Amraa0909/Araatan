//! ARAATAN v0.6.1 — Core Library
pub use config::AppConfig;
use tickerall::TickerallClient;
use anyhow::Context;

pub mod config;
pub mod tickerall;
pub mod forecast;

pub const NAME: &str = "ARAATAN";
pub const VERSION: &str = "0.6.1";

/// Initialize logging
pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
}

/// Loads configuration from araatan.toml or returns default




pub async fn cmd_symbols(cfg: &AppConfig) -> anyhow::Result<()> {
    let tickerall_config = cfg.tickerall.to_tickerall_config();
    let mut client = TickerallClient::new(tickerall_config);

    // 1. Эхлээд брокерт холбогдож session үүсгэх
    let broker = cfg.tickerall.broker.as_deref().unwrap_or("mt5");
    let server = cfg.tickerall.server.as_deref().unwrap_or("Demo");
    let account = cfg.tickerall.account.unwrap_or(0);
    let password = cfg.tickerall.password.as_deref().unwrap_or("");

    tracing::info!("Connecting to broker to get a hot session...");
    let account_id = client.start_session(broker, server, account, password).await?;
    tracing::info!("Session active. Account ID: {}", account_id);

    // 2. Session амьд байхад нь симболуудыг татах
    let symbols = client.fetch_symbols(&account_id).await?;
    
    println!("--- AVAILABLE SYMBOLS ({}) ---", symbols.len());
    for (i, sym) in symbols.iter().enumerate() {
        println!("{}: {}", i + 1, sym);
    }
    
    Ok(())
}

pub fn load_config() -> AppConfig {
    match config::AppConfig::load() {
        Ok(c) => {
            println!("DEBUG: Config loaded successfully. api_key is: {:?}", c.tickerall.api_key);
            c
        }
        Err(e) => {
            println!("DEBUG: Failed to parse araatan.toml: {}", e);
            AppConfig::default()
        }
    }
}

/// Check health of the system
pub async fn check_health(_cfg: &AppConfig) {
    let tickerall_config = _cfg.tickerall.to_tickerall_config();
    let client = TickerallClient::new(tickerall_config);

    if client.has_api_key() && client.has_account_id() {
        tracing::info!("TickerAll client initialized with API key and account ID");
    } else if client.has_api_key() {
        tracing::info!("TickerAll client initialized with API key (no account ID)");
    } else {
        tracing::info!("TickerAll client initialized (no API key or account ID)");
    }

    tracing::info!("Health check passed");
}

/// Fetch market data
pub async fn cmd_fetch(_cfg: &AppConfig, symbol: &str, hours: u32) -> anyhow::Result<()> {
    let tickerall_config = _cfg.tickerall.to_tickerall_config();
    let client = TickerallClient::new(tickerall_config);

    let bars = client.fetch_candles(symbol, &_cfg.system.timeframe, hours).await?;

    tracing::info!(
        "Fetched {} bars for {} in the last {} hours",
        bars.len(),
        symbol,
        hours
    );

    Ok(())
}

/// Warmup market data
pub async fn cmd_warmup(_cfg: &AppConfig, symbol: &str, target_bars: usize) -> anyhow::Result<()> {
    let tickerall_config = _cfg.tickerall.to_tickerall_config();
    let client = TickerallClient::new(tickerall_config);

    let bars = client.fetch_warmup_bars(symbol, target_bars).await?;

    tracing::info!(
        "Warmup complete: {} bars for {} (target: {})",
        bars.len(),
        symbol,
        target_bars
    );

    Ok(())
}

/// Start a broker session
pub async fn cmd_start_session(_cfg: &AppConfig) -> anyhow::Result<()> {
    let tickerall_config = _cfg.tickerall.to_tickerall_config();
    let mut client = TickerallClient::new(tickerall_config);

    let broker = _cfg.tickerall.broker.as_deref().unwrap_or("mt5");
    let server = _cfg.tickerall.server.as_deref().unwrap_or("Demo");
    let account = _cfg.tickerall.account.unwrap_or(123456);
    let password = _cfg.tickerall.password.as_deref().unwrap_or("password");

    let account_id = client.start_session(broker, server, account, password).await?;

    tracing::info!("Broker session started with account ID: {}", account_id);

    Ok(())
}

/// List available symbols

/// Run live trading
pub async fn cmd_live(_cfg: &AppConfig) -> anyhow::Result<()> {
    tracing::info!("Live trading not yet implemented");
    Ok(())
}

/// Run forecast
pub async fn cmd_forecast(_cfg: &AppConfig, symbol: &str, model_path: &str, horizon: usize) -> anyhow::Result<()> {
    // Load forecast engine
    let engine = forecast::ForecastEngine::new(model_path, "toto-2-313m-q8.gguf")?;

    // Fetch historical data
    let tickerall_config = _cfg.tickerall.to_tickerall_config();
    let client = TickerallClient::new(tickerall_config);
    let bars = client.fetch_candles(symbol, &_cfg.system.timeframe, 24).await?;

    // Prepare input data (OHLC)
    let mut input: Vec<Vec<f64>> = vec![Vec::new(); 4]; // 4 variates: Open, High, Low, Close
    for bar in bars.iter() {
        input[0].push(bar.open);
        input[1].push(bar.high);
        input[2].push(bar.low);
        input[3].push(bar.close);
    }

    // Run forecast
    let (forecast, explanation) = engine.predict_ensemble(&input, horizon)?;
    tracing::info!("{}", explanation);

    // Print results
    tracing::info!("Forecast engine loaded for {} (horizon: {})", symbol, horizon);
    // tracing::info!("Model type: {:?}", engine.model_type);
    tracing::info!("Forecast: {:?}", forecast);

    Ok(())
}

/// Print help message
pub fn print_help() {
    println!(
        "ARAATAN v{} — Market Data & Trading System
Usage:
  araatan [command] [options]

Commands:
  check       Check system health
  fetch       Fetch market data
  warmup      Warmup market data
  session     Start a broker session
  symbols     List available symbols
  live        Run live trading
  forecast    Run forecast using GGUF model
  backtest    Run backtest
  convert     Convert GGUF model
  gguf-info   Show GGUF model info
  help        Show this help message
  version     Show version information
",
        VERSION
    );
}
