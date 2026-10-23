//! Forecasting Engine for ARAATAN using zsfm_core
use anyhow::Result;
use zsfm_chronos::{BurnChronosModel, Chronos2Config, InferConfig};
use std::path::Path;

/// Forecast Engine for multivariate time series
pub struct ForecastEngine {
    predictor: BurnChronosModel,
    model_type: String,
}

impl ForecastEngine {
    /// Creates a new ForecastEngine
    pub fn new(model_path: &str) -> Result<Self> {
        let model_type = "Chronos".to_string();
        let c2 = Chronos2Config::from_json(r#"{
            "d_model": 512,
            "d_kv": 64,
            "d_ff": 2048,
            "num_layers": 6,
            "num_heads": 8,
            "max_context": 512,
            "max_horizon": 128
        }"#)?;
        let config = InferConfig::from(&c2);
        let predictor = BurnChronosModel::load(Path::new(model_path), config)?;

        Ok(Self {
            predictor,
            model_type,
        })
    }

    /// Predicts multivariate time series
    pub fn predict_multivariate(&self, context: &[Vec<f64>], horizon: usize) -> Result<Vec<Vec<f64>>> {
        self.predictor.predict(context, horizon)
    }
}
