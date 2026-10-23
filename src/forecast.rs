use anyhow::{Context, Result};
use std::path::Path;
use zsfm_chronos::{BurnChronosModel, Chronos2Config, InferConfig};
use zsfm_toto::{TotoModel, TotoConfig, TotoInferConfig};

pub struct ForecastEngine {
    chronos: BurnChronosModel,
    toto: TotoModel,
}

impl ForecastEngine {
    pub fn new(chronos_path: &str, toto_path: &str) -> Result<Self> {
        // 1. Chronos-2 ачаалах
        let c2_json = r#"{
            "d_model": 512, "d_kv": 64, "d_ff": 2048,
            "num_layers": 6, "num_heads": 8,
            "chronos_config": {
                "context_length": 512, "input_patch_size": 16,
                "output_patch_size": 16, "input_patch_stride": 16,
                "quantiles": [0.1, 0.5, 0.9], "use_reg_token": false,
                "use_arcsinh": false, "max_output_patches": 64
            }
        }"#;
        let c2 = Chronos2Config::from_json(c2_json)?;
        let chronos = BurnChronosModel::load(Path::new(chronos_path), InferConfig::from(&c2))?;

        // 2. Toto-2 ачаалах
        let t2 = TotoConfig::default(); // Toto-ийн тохиргоо (шаардлагатай бол өөрчилнө)
        let toto = TotoModel::load(Path::new(toto_path), TotoInferConfig::from(&t2))?;

        Ok(Self { chronos, toto })
    }

    /// Хоёр загварын таамаглалыг нэгтгэж, шалтгаантай хамт буцаана
    pub fn predict_ensemble(&self, context: &[Vec<f64>], horizon: usize) -> Result<(Vec<Vec<f64>>, String)> {
        // Chronos таамаглал
        let chronos_flat = flatten_context(context);
        let chronos_f32 = self.chronos.forecast(&chronos_flat, horizon)?;
        let chronos_pred: Vec<Vec<f64>> = chronos_f32.into_iter()
            .map(|v| v.into_iter().map(|x| x as f64).collect()).collect();

        // Toto таамаглал
        let toto_flat = flatten_context(context);
        let toto_f32 = self.toto.forecast(&toto_flat, horizon)?;
        let toto_pred: Vec<Vec<f64>> = toto_f32.into_iter()
            .map(|v| v.into_iter().map(|x| x as f64).collect()).collect();

        // Нэгтгэх (Дундаж)
        let mut ensemble_pred = vec![vec![0.0; horizon]; 4]; // OHLC
        for t in 0..horizon {
            for v in 0..4 {
                ensemble_pred[v][t] = (chronos_pred[v][t] + toto_pred[v][t]) / 2.0;
            }
        }

        // OHLC засвар (High >= Close, Low <= Close)
        for t in 0..horizon {
            let o = ensemble_pred[0][t];
            let c = ensemble_pred[3][t];
            ensemble_pred[1][t] = ensemble_pred[1][t].max(o).max(c);
            ensemble_pred[2][t] = ensemble_pred[2][t].min(o).min(c);
        }

        // Шалтгаан тайлбарлах (Одоогоор энгийн текст, дараа нь LLM-ээр баяжуулна)
        let last_close = context[3].last().copied().unwrap_or(0.0);
        let final_close = ensemble_pred[3][horizon - 1];
        let trend = if final_close > last_close { "ӨСӨХ (BUY)" } else { "БУУРАХ (SELL)" };
        
        let explanation = format!(
            "Загваруудын нэгдсэн дүгнэлт:\n\
            - Одоогийн хаалтын үнэ: {:.2}\n\
            - Таамагласан хаалтын үнэ ({} бар): {:.2}\n\
            - Хандлага: {}\n\
            - Тайлбар: Chronos-2 болон Toto-2 хоёулаа {} чиглэлийг баталж байна.",
            last_close, horizon, final_close, trend, trend
        );

        Ok((ensemble_pred, explanation))
    }
}

// Туслах функц: 2D массивыг 1D болгох
fn flatten_context(context: &[Vec<f64>]) -> Vec<f32> {
    let mut flat = Vec::new();
    for variate in context {
        for &val in variate {
            flat.push(val as f32);
        }
    }
    flat
}
