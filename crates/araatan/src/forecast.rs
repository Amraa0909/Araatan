#![allow(clippy::needless_range_loop)]
use anyhow::{Context, Result};
use std::path::Path;
use zsfm_chronos::{BurnChronosModel, Chronos2Config, InferConfig as ChronosInferConfig};
use zsfm_toto::{BurnTotoModel, InferConfig as TotoInferConfig};

pub struct ForecastEngine {
    chronos: BurnChronosModel,
    toto: BurnTotoModel,
}

impl ForecastEngine {
    /// chronos_path / toto_path — GGUF файлууд
    /// chronos_cfg_path / toto_cfg_path — config.json файлууд
    pub fn new(
        chronos_path: &str,
        chronos_cfg_path: &str,
        toto_path: &str,
        toto_cfg_path: &str,
    ) -> Result<Self> {
        // 1. Chronos-2 — config.json-оос (From<&Chronos2Config> mapping)
        let c2_json = std::fs::read_to_string(chronos_cfg_path)
            .with_context(|| format!("Cannot read chronos config: {chronos_cfg_path}"))?;
        let c2 = Chronos2Config::from_json(&c2_json)?;
        let chronos =
            BurnChronosModel::load(Path::new(chronos_path), ChronosInferConfig::from(&c2))?;

        // 2. Toto-2 — config.json-оос builder methods
        let t2_str = std::fs::read_to_string(toto_cfg_path)
            .with_context(|| format!("Cannot read toto config: {toto_cfg_path}"))?;
        let t2: serde_json::Value = serde_json::from_str(&t2_str)?;
        let u = |k: &str, d: u64| t2[k].as_u64().unwrap_or(d) as usize;
        let f = |k: &str, d: f64| t2[k].as_f64().unwrap_or(d);
        let b = |k: &str, d: bool| t2[k].as_bool().unwrap_or(d);
        let toto_cfg = TotoInferConfig::default()
            .with_d_model(u("d_model", 1024))
            .with_num_layers(u("num_layers", 24))
            .with_num_heads(u("num_heads", 16))
            .with_num_groups(u("num_groups", 16))
            .with_qk_dim(u("qk_dim", 64))
            .with_v_dim(u("v_dim", 64))
            .with_patch_size(u("patch_size", 32))
            .with_norm_eps(f("norm_eps", 1e-4))
            .with_layer_group_size(u("layer_group_size", 24))
            .with_num_variate_layers_per_group(u("num_variate_layers_per_group", 1))
            .with_variate_layer_first(b("variate_layer_first", false))
            .with_use_xpos(b("use_xpos", true))
            .with_residual_mult(f("residual_mult", 0.75))
            .with_residual_attn_ratio(f("residual_attn_ratio", 5.136215466577748));
        let toto = BurnTotoModel::load(Path::new(toto_path), toto_cfg)?;

        Ok(Self { chronos, toto })
    }

    /// Хоёр загварын таамаглалыг нэгтгэж, тайлбартай буцаана
    pub fn predict_ensemble(
        &self,
        context: &[Vec<f64>],
        horizon: usize,
    ) -> Result<(Vec<Vec<f64>>, String)> {
        let n_var = context.len();

        // --- Chronos: variate бүрд тусдаа таамаглал ---
        let mut chronos_pred: Vec<Vec<f64>> = Vec::with_capacity(n_var);
        for v in 0..n_var {
            let var_f32: Vec<f32> = context[v].iter().map(|&x| x as f32).collect();
            let fcst = self.chronos.forecast(&var_f32, horizon)?;
            let mid = fcst.len() / 2;
            let median = fcst.into_iter().nth(mid).unwrap_or_default();
            chronos_pred.push(median.into_iter().map(|x| x as f64).collect());
        }

        // --- Toto: multivariate, нэг дуудалт ---
        let ctx_f32: Vec<Vec<f32>> = context
            .iter()
            .map(|v| v.iter().map(|&x| x as f32).collect())
            .collect();
        let mask: Vec<Vec<bool>> = context.iter().map(|v| vec![true; v.len()]).collect();
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

        // --- OHLC засвар (context = [O, H, L, C]) ---
        if n_var >= 4 {
            for t in 0..horizon {
                let o = ensemble_pred[0][t];
                let c = ensemble_pred[3][t];
                ensemble_pred[1][t] = ensemble_pred[1][t].max(o).max(c);
                ensemble_pred[2][t] = ensemble_pred[2][t].min(o).min(c);
            }
        }

        let last_close = context[3].last().copied().unwrap_or(0.0);
        let final_close = ensemble_pred[3][horizon - 1];
        let trend = if final_close > last_close {
            "ӨСӨХ (BUY)"
        } else {
            "БУУРАХ (SELL)"
        };

        let explanation = format!(
            "Загваруудын нэгдсэн дүгнэлт:\n\
            - Одоогийн хаалтын үнэ: {last_close:.2}\n\
            - Таамагласан хаалтын үнэ ({horizon} бар): {final_close:.2}\n\
            - Хандлага: {trend}\n\
            - Тайлбар: Chronos-2 болон Toto-2 хоёулаа {trend} чиглэлийг баталж байна."
        );

        Ok((ensemble_pred, explanation))
    }
}
