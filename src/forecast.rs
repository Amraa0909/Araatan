//! Forecasting Engine for ARAATAN using zsfm-rs
use anyhow::{Context, Result};
use zsfm::{GGUFFile, ModelType, Predictor};
use std::path::Path;

/// Forecast Engine for multivariate time series
pub struct ForecastEngine {
    predictor: Predictor,
    model_type: ModelType,
}

impl ForecastEngine {
    /// Loads a GGUF model for forecasting
    pub fn new(model_path: &str) -> Result<Self> {
        let gguf = GGUFFile::load(Path::new(model_path))
            .context("Failed to load GGUF model")?;

        let model_type = gguf.model_type().context("Model type not specified in GGUF")?;
        let predictor = Predictor::new(gguf).context("Failed to create predictor")?;

        Ok(Self {
            predictor,
            model_type,
        })
    }

    /// Predicts multivariate time series
    pub fn predict_multivariate(&mut self, input: &[Vec<f32>], horizon: usize) -> Result<Vec<Vec<f32>>> {
        let input = input.to_vec();
        let output = self.predictor.predict_multivariate(&input, horizon)
            .context("Failed to make multivariate prediction")?;

        Ok(output)
    }

    /// Post-process OHLC data to ensure High >= max(Open, Close) and Low <= min(Open, Close)
    pub fn post_process_ohlc(&self, ohlc: &mut [Vec<f32>]) {
        for bar in ohlc.iter_mut() {
            if bar.len() < 4 {
                continue;
            }

            let open = bar[0];
            let close = bar[3];

            // Ensure High >= max(Open, Close)
            if bar[1] < open.max(close) {
                bar[1] = open.max(close);
            }

            // Ensure Low <= min(Open, Close)
            if bar[2] > open.min(close) {
                bar[2] = open.min(close);
            }
        }
    }
}
