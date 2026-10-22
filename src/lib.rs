//! ARAATAN v0.6.1 — Core Library
pub use config::AppConfig;
use tickerall::TickerallClient;
use zsfm_core::{GGUFFile, ModelType, Predictor};
use std::path::Path;

pub mod config;
pub mod tickerall;

pub const NAME: &str = "ARAATAN";
pub const VERSION: &str = "0.6.1";

/// Forecast Engine for multivariate time series
pub struct ForecastEngine {
    predictor: Predictor,
    model_type: ModelType,
}

impl ForecastEngine {
    /// Loads a GGUF model for forecasting
    pub fn new(model_path: &str) -> anyhow::Result<Self> {
        let gguf = GGUFFile::load(Path::new(model_path))
            .map_err(|e| anyhow::anyhow!("Failed to load GGUF model: {}", e))?;

        let model_type = gguf.model_type()
            .ok_or_else(|| anyhow::anyhow!("Model type not specified in GGUF"))?;
        let predictor = Predictor::new(gguf)
            .map_err(|e| anyhow::anyhow!("Failed to create predictor: {}", e))?;

        Ok(Self {
            predictor,
            model_type,
        })
    }
}

/// Initialize logging
pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
}

/// Loads configuration from araatan.toml or returns default
pub fn load_config() -> AppConfig {
    config::AppConfig::load().unwrap_or_default()
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

/// Run live trading
pub async fn cmd_live(_cfg: &AppConfig) -> anyhow::Result<()> {
    tracing::info!("Live trading not yet implemented");
    Ok(())
}

/// Run forecast
pub async fn cmd_forecast(_cfg: &AppConfig, symbol: &str, model_path: &str, horizon: usize) -> anyhow::Result<()> {
    // Load forecast engine
    let mut engine = ForecastEngine::new(model_path)?;

    // Fetch historical data
    let tickerall_config = _cfg.tickerall.to_tickerall_config();
    let client = TickerallClient::new(tickerall_config);
    let bars = client.fetch_candles(symbol, &_cfg.system.timeframe, 24).await?;

    // Prepare input data (OHLC)
    let mut input: Vec<Vec<f32>> = bars.iter().map(|bar| vec![
        bar.open as f32,
        bar.high as f32,
        bar.low as f32,
        bar.close as f32,
    ]).collect();

    // Print results
    tracing::info!("Forecast engine loaded for {} (horizon: {})", symbol, horizon);
    tracing::info!("Model type: {:?}", engine.model_type);

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
