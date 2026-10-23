#![allow(clippy::needless_range_loop)]
use anyhow::Result;
use std::path::Path;
use zsfm_chronos::{BurnChronosModel, Chronos2Config, InferConfig as ChronosInferConfig};
use zsfm_toto::{BurnTotoModel, InferConfig as TotoInferConfig};

pub struct ForecastEngine {
    chronos: BurnChronosModel,
    toto: BurnTotoModel,
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
        let chronos =
            BurnChronosModel::load(Path::new(chronos_path), ChronosInferConfig::from(&c2))?;

        // 2. Toto-2 ачаалах — InferConfig::default() нь upstream
        //    Datadog/Toto-2.0 config.json-той тохирно
        let toto = BurnTotoModel::load(Path::new(toto_path), TotoInferConfig::default())?;

        Ok(Self { chronos, toto })
    }

    /// Хоёр загварын таамаглалыг нэгтгэж, шалтгаантай хамт буцаана
    pub fn predict_ensemble(
        &self,
        context: &[Vec<f64>],
        horizon: usize,
    ) -> Result<(Vec<Vec<f64>>, String)> {
        let n_var = context.len();

        // --- Chronos: univariate, variate бүрд тусдаа таамаглал ---
        // context[v] = тухайн variate-ийн бар утгууд (жишээ: O, H, L, C)
        // forecast → [n_quantiles][horizon]; дунд quantile-г авна
        let mut chronos_pred: Vec<Vec<f64>> = Vec::with_capacity(n_var);
        for v in 0..n_var {
            let var_f32: Vec<f32> = context[v].iter().map(|&x| x as f32).collect();
            let fcst = self.chronos.forecast(&var_f32, horizon)?;
            let mid = fcst.len() / 2; // 3 quantile (0.1, 0.5, 0.9) → index 1
            let median = fcst.into_iter().nth(mid).unwrap_or_default();
            chronos_pred.push(median.into_iter().map(|x| x as f64).collect());
        }

        // --- Toto: multivariate, нэг дуудалт ---
        let ctx_f32: Vec<Vec<f32>> = context
            .iter()
            .map(|v| v.iter().map(|&x| x as f32).collect())
            .collect();
        let mask: Vec<Vec<bool>> = context.iter().map(|v| vec![true; v.len()]).collect();
        // toto_raw: [n_quantiles=9][n_var][horizon]
        let toto_raw = self.toto.forecast(&ctx_f32, &mask, horizon)?;
        let median_q = 4; // 9 quantile-ийн дунд (0.5)
        let toto_pred: Vec<Vec<f64>> = toto_raw[median_q]
            .iter()
            .map(|v| v.iter().map(|&x| x as f64).collect())
            .collect();

        // --- Нэгтгэх (дундаж) ---
        let mut ensemble_pred = vec![vec![0.0_f64; horizon]; n_var];
        for v in 0..n_var {
            for t in 0..horizon {
                ensemble_pred[v][t] = (chronos_pred[v][t] + toto_pred[v][t]) / 2.0;
            }
        }

        // --- OHLC засвар (context нь [O, H, L, C] гэж үзнэ) ---
        if n_var >= 4 {
            for t in 0..horizon {
                let o = ensemble_pred[0][t];
                let c = ensemble_pred[3][t];
                ensemble_pred[1][t] = ensemble_pred[1][t].max(o).max(c);
                ensemble_pred[2][t] = ensemble_pred[2][t].min(o).min(c);
            }
        }

        // --- Тайлбар ---
        let last_close = context[3].last().copied().unwrap_or(0.0);
        let final_close = ensemble_pred[3][horizon - 1];
        let trend = if final_close > last_close {
            "ӨСӨХ (BUY)"
        } else {
            "БУУРАХ (SELL)"
        };

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
