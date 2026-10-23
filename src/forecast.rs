use anyhow::Result;
use std::path::Path;
use zsfm_chronos::{BurnChronosModel, Chronos2Config, InferConfig};

pub struct ForecastEngine {
    predictor: BurnChronosModel,
}

impl ForecastEngine {
    pub fn new(model_path: &str) -> Result<Self> {
        let c2 = Chronos2Config::from_json(r#"{
            "d_model": 512,
            "d_kv": 64,
            "d_ff": 2048,
            "num_layers": 6,
            "num_heads": 8,
            "chronos_config": {
                "context_length": 512,
                "input_patch_size": 16,
                "output_patch_size": 16,
                "input_patch_stride": 16,
                "quantiles": [0.1, 0.5, 0.9],
                "use_reg_token": false,
                "use_arcsinh": false,
                "max_output_patches": 64
            }
        }"#)?;
        let config = InferConfig::from(&c2);
        let predictor = BurnChronosModel::load(Path::new(model_path), config)?;
        Ok(Self { predictor })
    }

    pub fn predict_multivariate(&self, context: &[Vec<f64>], horizon: usize) -> Result<Vec<Vec<f64>>> {
        let mut flat_context: Vec<f32> = Vec::new();
        for variate in context {
            for &val in variate {
                flat_context.push(val as f32);
            }
        }

        let f32_forecast = self.predictor.forecast(&flat_context, horizon)?;

        let f64_forecast = f32_forecast
            .into_iter()
            .map(|v| v.into_iter().map(|x| x as f64).collect())
            .collect();

        Ok(f64_forecast)
    }
}
