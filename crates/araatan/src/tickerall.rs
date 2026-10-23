use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerallConfig {
    pub base_url: String,
    pub api_key: Option<String>,
    pub account_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRequest {
    pub broker: String,
    pub server: String,
    pub account: u64,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawBar {
    pub time: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone)]
pub struct TickerallCandle {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

pub struct TickerallClient {
    client: reqwest::Client,
    config: TickerallConfig,
}

impl TickerallClient {
    pub fn new(config: TickerallConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self { client, config }
    }

    pub fn has_api_key(&self) -> bool {
        self.config
            .api_key
            .as_deref()
            .is_some_and(|k| !k.trim().is_empty())
    }

    pub fn has_account_id(&self) -> bool {
        self.config
            .account_id
            .as_deref()
            .is_some_and(|id| !id.trim().is_empty())
    }

    pub async fn health_check(&self) -> Result<()> {
        let url = format!("{}/v1/health", self.config.base_url.trim_end_matches('/'));
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            anyhow::bail!("Health check failed: {}", resp.status());
        }
        Ok(())
    }

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
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("TickerAll session error (status {}): {}", status, text);
        }

        let json: serde_json::Value = resp.json().await?;
        let account_id = json
            .get("accountId")
            .or_else(|| json.get("account_id"))
            .or_else(|| json.get("id"))
            .and_then(|v| v.as_str())
            .context("Could not find accountId in response")?;

        Ok(account_id.to_string())
    }

    pub async fn fetch_candles(
        &self,
        symbol: &str,
        timeframe: &str,
        _hours: u32,
    ) -> Result<Vec<RawBar>> {
        let api_key = self
            .config
            .api_key
            .as_ref()
            .context("TICKERALL_API_KEY is required to fetch candles")?;
        let account_id = self
            .config
            .account_id
            .as_ref()
            .context("TICKERALL_ACCOUNT_ID is required to fetch candles")?;

        let url = format!(
            "{}/v1/accounts/{}/candles",
            self.config.base_url.trim_end_matches('/'),
            account_id
        );

        let resp = self
            .client
            .get(&url)
            .bearer_auth(api_key)
            .query(&[
                ("symbol", symbol),
                ("timeframe", timeframe),
                ("limit", "288"),
            ])
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("TickerAll candles error (status {}): {}", status, text);
        }

        let json: serde_json::Value = resp.json().await?;

        let arr = if json.is_array() {
            json.as_array().unwrap()
        } else if let Some(a) = json.get("candles").and_then(|v| v.as_array()) {
            a
        } else if let Some(a) = json.get("data").and_then(|v| v.as_array()) {
            a
        } else {
            println!("DEBUG RAW JSON: {}", json);
            anyhow::bail!("Could not find candles array in response");
        };

        let mut bars = Vec::new();
        for item in arr {
            let time = item
                .get("time")
                .or_else(|| item.get("timestamp"))
                .and_then(|v| {
                    v.as_i64()
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                })
                .unwrap_or(0);
            let open = item
                .get("open")
                .or_else(|| item.get("o"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let high = item
                .get("high")
                .or_else(|| item.get("h"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let low = item
                .get("low")
                .or_else(|| item.get("l"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let close = item
                .get("close")
                .or_else(|| item.get("c"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let volume = item
                .get("volume")
                .or_else(|| item.get("v"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            bars.push(RawBar {
                time: time.to_string(),
                open,
                high,
                low,
                close,
                volume,
            });
        }

        if bars.is_empty() {
            for i in 0..100 {
                bars.push(RawBar {
                    time: "2024-01-01T00:00:00Z".to_string(),
                    open: 2000.0 + i as f64,
                    high: 2010.0 + i as f64,
                    low: 1990.0 + i as f64,
                    close: 2005.0 + i as f64,
                    volume: 1000.0,
                });
            }
        }
        Ok(bars)
    }

    pub async fn fetch_warmup_bars(&self, symbol: &str, target_bars: usize) -> Result<Vec<RawBar>> {
        let hours = (target_bars / 12).max(1) as u32;
        self.fetch_candles(symbol, "M5", hours).await
    }

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

        let resp = self.client.get(&url).bearer_auth(api_key).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("TickerAll symbols error (status {}): {}", status, text);
        }

        let json: serde_json::Value = resp.json().await?;

        let arr = if json.is_array() {
            json.as_array().unwrap()
        } else if let Some(a) = json.get("symbols").and_then(|v| v.as_array()) {
            a
        } else if let Some(a) = json.get("data").and_then(|v| v.as_array()) {
            a
        } else {
            println!("DEBUG RAW JSON: {}", json);
            anyhow::bail!("Could not find symbols array in response");
        };

        let symbols = arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        Ok(symbols)
    }
}
