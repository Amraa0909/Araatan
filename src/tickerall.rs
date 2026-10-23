//! TickerAll API Client for ARAATAN
//! Provides historical OHLC M5 bars, warmup buffers, and live session management via TickerAll.com (MetaTrader bridge).

use std::env;
use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Internal RawBar format as specified in ARAATAN architecture_json.md
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawBar {
    pub symbol: String,
    pub ts_utc: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub timeframe: String,
}

impl RawBar {
    /// Returns UTC DateTime of the bar
    #[allow(dead_code)]
    pub fn datetime(&self) -> Option<DateTime<Utc>> {
        DateTime::from_timestamp(self.ts_utc, 0)
    }
}

/// A single candle record returned from TickerAll API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerallCandle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    #[serde(default)]
    pub bid: Option<f64>,
    #[serde(rename = "tickVolume", default)]
    pub tick_volume: Option<f64>,
    #[serde(default)]
    pub spread: Option<f64>,
}

impl TickerallCandle {
    /// Converts a TickerAll candle into ARAATAN's RawBar
    pub fn to_raw_bar(&self, symbol: &str, timeframe: &str) -> RawBar {
        RawBar {
            symbol: symbol.to_string(),
            ts_utc: self.timestamp,
            open: self.open,
            high: self.high,
            low: self.low,
            close: self.close,
            volume: self.tick_volume.unwrap_or(0.0),
            timeframe: timeframe.to_string(),
        }
    }
}

/// Response returned by TickerAll historical candles endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerallCandlesResponse {
    #[serde(default)]
    pub candles: Vec<TickerallCandle>,
    #[serde(default)]
    pub count: Option<usize>,
    #[serde(default)]
    pub coverage: Option<String>,
    #[serde(default)]
    pub truncated: Option<bool>,
    #[serde(rename = "stopReason", default)]
    pub stop_reason: Option<String>,
}

/// Session request payload for POST /v1/sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRequest {
    pub broker: String,
    pub server: String,
    pub account: u64,
    pub password: String,
}

/// Session response payload returned by TickerAll
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionResponse {
    #[serde(rename = "accountId")]
    pub account_id: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

/// Symbol response payload returned by TickerAll
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolResponse {
    pub symbol: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub contract_size: Option<f64>,
    #[serde(default)]
    pub pip_size: Option<f64>,
    #[serde(default)]
    pub spread: Option<f64>,
    #[serde(default)]
    pub commission: Option<f64>,
    #[serde(default)]
    pub swap_long: Option<f64>,
    #[serde(default)]
    pub swap_short: Option<f64>,
}

/// Configuration for TickerAll API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerallConfig {
    pub api_key: Option<String>,
    pub account_id: Option<String>,
    pub base_url: String,
}

impl Default for TickerallConfig {
    fn default() -> Self {
        Self {
            api_key: env::var("TICKERALL_API_KEY").ok(),
            account_id: env::var("TICKERALL_ACCOUNT_ID").ok(),
            base_url: env::var("TICKERALL_BASE_URL")
                .unwrap_or_else(|_| "https://api.tickerall.com".to_string()),
        }
    }
}

/// Client to communicate with TickerAll REST API
pub struct TickerallClient {
    config: TickerallConfig,
    client: reqwest::Client,
}

impl TickerallClient {
    /// Creates a new TickerAll client from config
    pub fn new(config: TickerallConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self { config, client }
    }

    /// Creates a client using environment variables
    #[allow(dead_code)]
    pub fn from_env() -> Self {
        Self::new(TickerallConfig::default())
    }

    /// Check if API key is set
    pub fn has_api_key(&self) -> bool {
        self.config.api_key.as_ref().map_or(false, |k| !k.trim().is_empty())
    }

    /// Check if account ID is set
    pub fn has_account_id(&self) -> bool {
        self.config.account_id.as_ref().map_or(false, |a| !a.trim().is_empty())
    }

    /// Authenticate and start a broker session to obtain an `accountId`
    pub async fn start_session(
        &mut self,
        broker: &str,
        server: &str,
        account: u64,
        password: &str,
    ) -> Result<String> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .context("TICKERALL_API_KEY is required to start a broker session")?;

        let url = format!("{}/v1/sessions", self.config.base_url.trim_end_matches('/'));
        let req_body = SessionRequest {
            broker: broker.to_string(),
            server: server.to_string(),
            account,
            password: password.to_string(),
        };

        let resp = self
            .client
            .post(&url)
            .bearer_auth(api_key)
            .json(&req_body)
            .send()
            .await
            .context("Failed to send session request to TickerAll")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("TickerAll session error (status {}): {}", status, text);
        }

        let session_data: SessionResponse = resp
            .json()
            .await
            .context("Failed to parse TickerAll session response")?;

        let account_id = session_data
            .account_id
            .or(session_data.id)
            .context("TickerAll session response did not return an accountId")?;

        self.config.account_id = Some(account_id.clone());
        Ok(account_id)
    }

    /// Fetch available symbols for the current account
    pub async fn fetch_symbols(&self, account_id: &str) -> Result<Vec<String>> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .context("TICKERALL_API_KEY is required to fetch symbols")?;

        let url = format!(
            "{}/v1/accounts/{}/symbols",
            self.config.base_url.trim_end_matches('/'),
            account_id
        );

        let resp = self
            .client
            .get(&url)
            .bearer_auth(api_key)
            .send()
            .await
            .context("Failed to send symbols request to TickerAll")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("TickerAll symbols error (status {}): {}", status, text);
        }

        let symbols_data: Vec<SymbolResponse> = resp
            .json()
            .await
            .context("Failed to parse TickerAll symbols response")?;

        let symbols: Vec<String> = symbols_data
            .into_iter()
            .map(|s| s.symbol)
            .collect();

        Ok(symbols)
    }

    /// Fetch historical OHLC candles
    /// Uses authenticated `/v1/accounts/:id/candles` if `account_id` and `api_key` are present;
    /// otherwise falls back to public `/v1/public/candles`.
    pub async fn fetch_candles(
        &self,
        symbol: &str,
        timeframe: &str,
        hours: u32,
    ) -> Result<Vec<RawBar>> {
        let base_url = self.config.base_url.trim_end_matches('/');

        let has_acc = self.config.account_id.as_ref().map_or(false, |a| !a.trim().is_empty());
        let has_key = self.config.api_key.as_ref().map_or(false, |k| !k.trim().is_empty());

        let resp = if has_acc && has_key {
            let account_id = self.config.account_id.as_ref().unwrap().trim();
            let api_key = self.config.api_key.as_ref().unwrap().trim();
            let url = format!("{}/v1/accounts/{}/candles", base_url, account_id);
            self.client
                .get(&url)
                .bearer_auth(api_key)
                .query(&[
                    ("symbol", symbol),
                    ("timeframe", timeframe),
                    ("hours", &hours.to_string()),
                ])
                .send()
                .await
                .with_context(|| format!("Failed to request candles from {}", url))?
        } else {
            let url = format!("{}/v1/public/candles", base_url);
            self.client
                .get(&url)
                .header("Origin", "https://tickerall.com")
                .header("Referer", "https://tickerall.com/")
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
                .query(&[
                    ("symbol", symbol),
                    ("timeframe", timeframe),
                    ("hours", &hours.to_string()),
                ])
                .send()
                .await
                .with_context(|| format!("Failed to request public candles from {}", url))?
        };

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!(
                "TickerAll candle request failed ({}) for {}: {}",
                status,
                symbol,
                text
            );
        }

        let body: TickerallCandlesResponse = resp
            .json()
            .await
            .context("Failed to parse TickerAll candles JSON response")?;

        let raw_bars: Vec<RawBar> = body
            .candles
            .into_iter()
            .map(|c| c.to_raw_bar(symbol, timeframe))
            .collect();

        Ok(raw_bars)
    }

    /// Warmup fetching: gets at least `target_bars` (default: 600 bars for ARAATAN M5 warmup)
    pub async fn fetch_warmup_bars(&self, symbol: &str, target_bars: usize) -> Result<Vec<RawBar>> {
        // M5 bars: 12 bars per hour. 600 bars = 50 hours. Add buffer (e.g. 60 hours)
        let hours_needed = ((target_bars as f64 / 12.0) * 1.2).ceil() as u32;
        let mut bars = self.fetch_candles(symbol, "M5", hours_needed.max(1)).await?;

        // Sort ascending by timestamp
        bars.sort_by_key(|b| b.ts_utc);

        // Keep the latest `target_bars` if we have more
        if bars.len() > target_bars {
            let skip_count = bars.len() - target_bars;
            bars.drain(0..skip_count);
        }

        Ok(bars)
    }
}
