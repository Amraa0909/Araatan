{
  "system": {
    "name": "ARAATAN",
    "version": "0.6.1",
    "symbols": ["XAUUSD", "USDJPY"],
    "timeframe": "M5",
    "timeframe_ms": 300000,
    "bars_per_day": 288,
    "bars_per_week": 1440,
    "bar_close_utc_offset_sec": 0,
    "model_runtime": "zsfm-rs",
    "quantization": "Q8_0",
    "rust_msrv": "1.78.0",
    "note": "M5 bar = 300,000 ms. Live inference budget (~600 ms/bar) uses 0.20% of bar — abundant headroom."
  },

  "upstream": {
    "zsfm": {
      "repo": "https://github.com/Amraa0909/zsfm-rs",
      "upstream": "https://github.com/amaye15/zsfm-rs",
      "pin": "main",
      "license": "Apache-2.0",
      "crates": ["zsfm-core", "zsfm-checkpoint", "zsfm-nn", "zsfm-chronos", "zsfm-toto", "zsfm-tabicl"],
      "rationale": "Bit-exact GGUF + Candle inference for Chronos-2, Toto-2, TabICL v2. No Python, no PyTorch, no ONNX."
    }
  },

  "rust_crates": {
    "msrv": "1.78.0",
    "registry": "crates.io",
    "lockfile": "Cargo.lock (committed)",
    "supply_chain": {
      "cargo-deny": "0.16+",
      "cargo-audit": "0.20+",
      "policy": "deny yanked, deny unmaintained, warn duplicate versions"
    },
    "core": {
      "serde":              { "version": "1.0.219", "features": ["derive"], "role": "Serialization" },
      "serde_json":         { "version": "1.0.140", "role": "JSON I/O for audit + config" },
      "toml":               { "version": "0.8.22",  "role": "araatan.toml config parsing" },
      "figment":            { "version": "0.10.19", "role": "Layered config (env + file)" },
      "chrono":             { "version": "0.4.41",  "features": ["serde"], "role": "UTC timestamps" },
      "thiserror":          { "version": "1.0.69",  "role": "Error enums" },
      "anyhow":             { "version": "1.0.98",  "role": "Application errors" },
      "uuid":               { "version": "1.11.0",  "features": ["v4", "serde"], "role": "setup_id, decision_id" },
      "async-trait":        { "version": "0.1.88",  "role": "async Broker/Storage traits" },
      "once_cell":          { "version": "1.21.0",  "role": "Static Lazy" }
    },
    "async_runtime": {
      "tokio":              { "version": "1.53.1",  "features": ["rt-multi-thread", "macros", "time", "sync", "signal"], "role": "Async runtime" },
      "tokio-util":         { "version": "0.7.16",  "role": "Async helpers" }
    },
    "concurrency": {
      "parking_lot":        { "version": "0.12.5",  "role": "Mutex/RwLock for shared state" },
      "dashmap":            { "version": "6.2.1",   "role": "Concurrent HashMaps (open setups/positions)" },
      "crossbeam":          { "version": "0.8.4",   "role": "Channels for bar feed" },
      "rayon":              { "version": "1.11.0",  "role": "Parallel feature computation across symbols" }
    },
    "ml_and_tensor": {
      "candle-core":        { "version": "0.11.0",  "features": ["cuda", "metal"], "role": "Tensor ops (zsfm backend)" },
      "candle-nn":          { "version": "0.11.0",  "role": "NN layers" },
      "candle-transformers":{ "version": "0.11.0",  "role": "Transformer building blocks" },
      "ndarray":            { "version": "0.17.2",  "features": ["serde"], "role": "Feature matrices (L1_7a)" }
    },
    "cli_and_logging": {
      "clap":               { "version": "4.6.7",   "features": ["derive", "env"], "role": "CLI (live/backtest/replay/convert/check)" },
      "tracing":            { "version": "0.1.41",  "role": "Structured logging" },
      "tracing-subscriber": { "version": "0.3.19",  "features": ["env-filter", "json"], "role": "Log subscriber" }
    },
    "money_and_metrics": {
      "rust_decimal":       { "version": "1.41.0",  "features": ["serde-with-str"], "role": "Price, lot, PnL" },
      "metrics":            { "version": "0.24.6",  "role": "Metrics facade" },
      "metrics-exporter-prometheus": { "version": "0.17.0", "role": "Prometheus exporter" }
    },
    "storage": {
      "sqlx":               { "version": "0.8.6",   "features": ["runtime-tokio", "sqlite", "chrono", "uuid", "rust_decimal", "json"], "role": "L7 storage; 0.9 deferred — requires Rust 1.94 (above MSRV)" }
    },
    "bytes":                { "version": "1.10.1",  "role": "Byte buffers" }
  },

  "rust_crates_notes": [
    "candle-* 0.7 → 0.11: zsfm-rs шинэ candle API-тай нийцсэн эсэхийг tag-аас хамаарч тогтоох; хэрэв zsfm-rs 0.7-г pin хийсэн бол 0.7-д үлдээнэ.",
    "sqlx 0.9.0 нь Rust 1.94 MSRV шаарддаг тул MSRV 1.78-д 0.8.6 дээр үлдэв.",
    "ndarray 0.15 → 0.17: breaking өөрчлөлтүүд (Array API) — feature crate-ийг дахин шалгах.",
    "clap 4.6.x нь MSRV 1.74 — OK.",
    "rust_decimal 1.41 — serde-with-str feature хэвээр ажиллана.",
    "candle-core cuda feature-ийг CUDA toolkit 12.x-тай тулгах; metal зөвхөн macOS-д.",
    "Бүх сан crates.io-оос авна; zsfm-* нь зөвхөн git dependency.",
    "cargo-deny / cargo-audit CI-д заавал ажиллана."
  ],

  "input": {
    "RawBar": {
      "fields": ["symbol", "ts_utc", "open", "high", "low", "close", "volume", "timeframe"]
    },
    "SymbolConfig": {
      "fields": ["contract", "lot_step", "lot_min", "lot_max", "stop_level", "freeze_level", "spread_avg", "commission", "swap_long", "swap_short"]
    },
    "quote_to_acct": {
      "XAUUSD": 1.0,
      "USDJPY": "1 / current_USDJPY_mid",
      "note": "per-bar resolved; referenced as quote_to_acct[bar.symbol] in L5"
    },
    "warmup": {
      "copy_bars": 600,
      "breakdown": {
        "chronos_context": 512,
        "label_horizon": 10,
        "lag": 20,
        "buffer": 58
      },
      "wall_clock_hours": 50,
      "note": "600 × 5 min = 50 hours of M5 history (~2 trading days).",
      "support_set_window": {
        "min_index": 10,
        "max_index": 510,
        "sample_count": 490,
        "min_required": 300,
        "status": "WARM_OK"
      },
      "runtime": {
        "warmup": {
          "batch_sec": 240,
          "mode": "batched_parallel",
          "per_bar_effective_ms": 400,
          "note": "Q8 heavier than Q5_K_M. 600 bars / 240 s = 400 ms/bar (batch amortized via candle batching)."
        },
        "live": {
          "per_bar_inference_ms": 600,
          "budget_ms": {
            "chronos": 250,
            "toto": 200,
            "tabicl": 100,
            "buffer": 50
          },
          "note": "M5 bar = 300,000 ms → 600 ms = 0.20% utilisation. CUDA/Metal expect ~2–3× faster."
        }
      }
    }
  },

  "models": {
    "runtime": {
      "backend": "zsfm",
      "device": {
        "default": "auto",
        "options": ["cpu", "cuda", "metal"],
        "note": "candle::Device::cuda_if_available() | Device::new_metal(0) | Device::Cpu"
      },
      "quantization": {
        "default": "Q8_0",
        "supported": ["Q8_0", "Q8_K"],
        "note": "8-bit block-32. ~4× larger than Q5_K_M, but bit-exact vs original checkpoint (zsfm-rs verified ~1e-6)."
      },
      "threads": 8,
      "batch_size": 1,
      "n_predict": 1,
      "verify_on_start": true,
      "checksums_file": "models/SHA256SUMS",
      "manifest": "models/manifest.json"
    },
    "chronos": {
      "crate": "zsfm-chronos",
      "gguf_path": "models/chronos-2-Q8_0.gguf",
      "quant": "Q8_0",
      "n_ctx": 512,
      "n_predict": 1,
      "size_mb_estimate": 420,
      "trait": "zsfm_core::Forecaster"
    },
    "toto": {
      "crate": "zsfm-toto",
      "gguf_path": "models/toto-2-Q8_0.gguf",
      "quant": "Q8_0",
      "n_ctx": 96,
      "n_predict": 1,
      "size_mb_estimate": 260,
      "trait": "zsfm_core::Forecaster"
    },
    "tabicl": {
      "crate": "zsfm-tabicl",
      "gguf_path": "models/tabicl-Q8_0.gguf",
      "quant": "Q8_0",
      "n_ctx": 512,
      "n_predict": 1,
      "size_mb_estimate": 180,
      "support_set_per_symbol": 300,
      "trait": "zsfm_core::Classifier",
      "note": "In-context learning fully supported inside GGUF via attention mask; no Python sidecar."
    }
  },

  "ohlcv_variants": {
    "raw_ohlcv": {
      "space": "price",
      "fields": ["open", "high", "low", "close", "volume"],
      "consumer": "toto"
    },
    "logret_ohlcv": {
      "space": "log_return",
      "reference": "close_{t-1}",
      "fields": {
        "log_ret_o": "ln(open_t / close_{t-1})",
        "log_ret_h": "ln(high_t / close_{t-1})",
        "log_ret_l": "ln(low_t / close_{t-1})",
        "log_ret_c": "ln(close_t / close_{t-1})",
        "log_ret_v": "ln(1 + volume_t / max(volume_{t-1}, epsilon))"
      },
      "consumer": "chronos"
    }
  },

  "layers": {
    "L1": {
      "name": "FEATURE_ENGINEERING",
      "in": ["raw_bars_x2", "history_600"],
      "out": {
        "EnrichedBar": {
          "features_count": 52,
          "XAU": {
            "count": 15,
            "features": ["log_return", "range", "body", "upper_wick", "lower_wick", "rsi_14", "macd_hist", "atr_norm", "vwap_dist", "bb_width", "stoch_k", "obv_slope", "mfi_14", "cci_20", "roc_10"]
          },
          "JPY": {
            "count": 15,
            "features": ["log_return", "range", "body", "upper_wick", "lower_wick", "rsi_14", "macd_hist", "atr_norm", "vwap_dist", "bb_width", "stoch_k", "obv_slope", "mfi_14", "cci_20", "roc_10"]
          },
          "Cross": {
            "count": 5,
            "features": ["corr_50", "corr_20", "dxy_return", "yield_10y_change", "risk_on_off"]
          },
          "Candle": {
            "count": 17,
            "features": ["doji", "hammer", "shooting_star", "engulf_bull", "engulf_bear", "harami", "morning_star", "evening_star", "pin_bull", "pin_bear", "inside_bar", "outside_bar", "three_white", "three_black", "tweezer_top", "tweezer_bottom", "marubozu"]
          }
        }
      }
    },

    "L1_5": {
      "name": "MARKET_CONTEXT",
      "in": ["EnrichedBar", "history"],
      "out": {
        "MarketContext": {
          "count": 33,
          "regime": {
            "count": 4,
            "classes": ["Trending", "Ranging", "Volatile", "Calm"],
            "activation": "softmax",
            "hysteresis_bars": 20,
            "hysteresis_minutes": 100
          },
          "trend": {
            "count": 5,
            "features": ["trend_strength", "slope_20", "slope_50", "ema_align", "adx_14"]
          },
          "sr": {
            "count": 5,
            "features": ["dist_to_nearest_sr", "sr_strength", "sr_side", "sr_count_50", "sr_age"]
          },
          "session": {
            "count": 6,
            "classes": ["Tokyo", "London", "NY", "Tokyo-London", "London-NY", "Off-hours"],
            "encoding": "onehot",
            "calendar": "UTC+DST+JP/UK/US holiday",
            "off_hours_window_utc": ["20:00", "00:00"]
          },
          "chart": {
            "count": 13,
            "features": ["hh", "hl", "lh", "ll", "bos_bull", "bos_bear", "choch_bull", "choch_bear", "fvg_count", "ob_count", "trendline_dist", "wedge_flag", "triangle_flag"]
          }
        }
      }
    },

    "L1_6": {
      "name": "DYNAMIC_PARAMETERS",
      "in": ["MarketContext", "history"],
      "out": {
        "ParameterSet": {
          "vol_factor": {
            "formula": "f(vol) = clamp(1 + 0.6 * (vol_pct - 0.5), 0.85, 1.15)",
            "clamp_semantics": "clamp(value, min, max)",
            "vol_pct": "ATR(14) percentile over 100 bars"
          },
          "entry_thr_buy": "0.55 * f(vol)",
          "entry_thr_sell": "0.45 * f(vol)",
          "cancel_thr_buy": "0.40 * f(vol)",
          "cancel_thr_sell": "0.60 * f(vol)",
          "atr_mult": "2.5 * f(vol)",
          "label_atr_mult": 2.0,
          "risk_pct": "1.0% * f(vol)",
          "max_loss": "1.0%",
          "partial_levels": [1.0, 2.0, 3.0],
          "partial_pcts": [0.5, 0.3, 0.2],
          "breakeven_at": "1.5R",
          "chandelier_mult": "3.0 * ATR(14)",
          "cooldown_bars": 4,
          "cooldown_minutes": 20,
          "time_exit_bars": 96,
          "time_exit_minutes": 480,
          "time_exit_hours": 8,
          "expire_bars": 8,
          "expire_minutes": 40,
          "max_slippage": "max(2.0pt, 0.5 * spread_avg)",
          "corr_block_thr": {
            "value": 0.40,
            "pair": ["XAUUSD", "USDJPY"],
            "window": 50,
            "applies_to": "same-direction dual position",
            "action": "forbid_entry"
          },
          "corr_exit_thr": {
            "value": -0.20,
            "pair": ["XAUUSD", "USDJPY"],
            "window": 50,
            "rationale": "cancel setup if negative cross-correlation weakens above -0.20"
          },
          "conf_min": 0.55,
          "min_edge": 0.05,
          "regime_threshold_lag": 20,
          "regime_threshold_lag_minutes": 100,
          "max_concurrent": 2,
          "daily_dd_limit": 0.05,
          "daily_dd_reset": "00:00 UTC",
          "weekly_dd_limit": 0.10,
          "weekly_dd_reset": "Mon 00:00 UTC",
          "kill_switch": {
            "rule": "(consec_losses >= 5) OR (count(|P_up - 0.5| < 0.03 AND conf < 0.40, last 20 bars) >= 15)",
            "rationale": "15/20 bars = 75% dormancy threshold; replaces overly strict all() rule"
          },
          "margin_buffer": "max(0.20 * required_margin, 100 USD)",
          "refit_trigger": "(every 50 bars) OR (Sharpe(20) < 0.5 AND closed_trades >= 5)",
          "refit_every_minutes": 250,
          "note_m5": "All per-bar values preserved. time_exit_bars=96 → 8 h at M5 (was 24 h at M15). To restore 24 h, set 288."
        }
      }
    },

    "L1_7a": {
      "name": "FEATURE_SELECTOR",
      "in": {
        "EnrichedBar": 52,
        "MarketContext": 33
      },
      "out": {
        "chronos_in": {
          "count": 24,
          "target_logret": {
            "count": 5,
            "source": "logret_ohlcv"
          },
          "past_cov": {
            "count": 19,
            "groups": {
              "XAU": ["log_return", "range", "body", "upper_wick", "lower_wick", "rsi_14", "macd_hist", "atr_norm", "vwap_dist", "adx_14"],
              "JPY": ["rsi_14", "macd_hist", "atr_norm", "vwap_dist"],
              "Cross": ["corr_50", "corr_20", "dxy_return", "yield_10y_change", "risk_on_off"]
            }
          }
        },
        "toto_in": {
          "count": 12,
          "raw_xau": ["open", "high", "low", "close", "volume"],
          "raw_jpy": ["open", "high", "low", "close", "volume"],
          "cross": ["corr_50", "corr_20"]
        },
        "tabicl_static": {
          "count": 15,
          "ctx_sel": {
            "count": 10,
            "regime_prob": 4,
            "trend_strength": 1,
            "sr_dist_norm": 1,
            "session_onehot": {
              "count": 4,
              "classes": ["Tokyo", "London", "NY", "Off-hours"],
              "mapping": {
                "Tokyo": "Tokyo",
                "London": "London",
                "NY": "NY",
                "Tokyo-London": "London",
                "London-NY": "NY",
                "Off-hours": "Off-hours"
              },
              "mapping_rationale": "later active session dominates liquidity; overlap ranges collapsed to later session",
              "off_hours_window_utc": ["20:00", "00:00"]
            }
          },
          "dyn_lagged": {
            "count": 5,
            "features": ["entry_thr_buy", "entry_thr_sell", "atr_mult", "risk_pct", "conf_min"],
            "lag_bars": 20,
            "lag_minutes": 100
          }
        }
      }
    },

    "L2A": {
      "name": "CHRONOS_2",
      "runtime": "zsfm-chronos",
      "model": "models/chronos-2-Q8_0.gguf",
      "quant": "Q8_0",
      "in": {
        "target_logret": {
          "shape": "512 x 5",
          "source": "logret_ohlcv"
        },
        "past_cov": {
          "shape": "512 x 19"
        },
        "quantiles": {
          "count": 21,
          "grid": "np.linspace(0.01, 0.99, 21)",
          "grid_values": [0.010, 0.059, 0.108, 0.157, 0.206, 0.255, 0.304, 0.353, 0.402, 0.451, 0.500, 0.549, 0.598, 0.647, 0.696, 0.745, 0.794, 0.843, 0.892, 0.941, 0.990],
          "derived_quantiles": {
            "method": "linear interpolation between adjacent grid points",
            "q05": "linear_interp(grid, 0.05)",
            "q10": "linear_interp(grid, 0.10)",
            "q25": "linear_interp(grid, 0.25)",
            "q75": "linear_interp(grid, 0.75)",
            "q90": "linear_interp(grid, 0.90)",
            "q95": "linear_interp(grid, 0.95)"
          }
        },
        "context_length": 512,
        "pred_length": 1
      },
      "out": {
        "p_up_chronos": {
          "formula": "1 - CDF_logret(0)",
          "space": "log_return",
          "assumption": "E[log_return] ≈ 0 over 200-bar window (validated during warmup; else adjust to CDF_z(-mean/sigma))"
        },
        "conf_chronos": "1 - (q90 - q10) / (q95 - q05 + epsilon)",
        "median_forecast": {
          "count": 5,
          "space": "log_return"
        },
        "quantile_forecast": {
          "shape": "21 x 5",
          "space": "log_return"
        },
        "chronos_out": {
          "count": 10,
          "fields": ["p_up_chronos", "conf_chronos", "med_close_z", "med_range_z", "q05_z", "q25_z", "q75_z", "q95_z", "q_spread_z", "dir_flag"]
        },
        "denormalization": {
          "formula": "q_price = last_close * exp(q_zscore * sigma_ret)",
          "q_zscore": "Chronos output quantile (z-score space)",
          "sigma_ret": "std(log_return over 200 bars)",
          "note": "Chronos outputs z-score quantiles; multiply by sigma_ret to recover log-return, then exp for price"
        }
      }
    },

    "L2B": {
      "name": "TOTO_2",
      "runtime": "zsfm-toto",
      "model": "models/toto-2-Q8_0.gguf",
      "quant": "Q8_0",
      "in": {
        "variate_raw": {
          "shape": "96 x 12",
          "source": "toto_in"
        },
        "quantiles": {
          "count": 9,
          "values": [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9]
        },
        "pred_length": 1
      },
      "out": {
        "p_up_toto": {
          "formula": "P(price_1 > last_close)",
          "computation": "1 - linear_interp(quantile_CDF, last_close) over 9 Toto quantiles [0.1..0.9]"
        },
        "conf_toto": "1 - (q80 - q20) / (q90 - q10 + epsilon)",
        "toto_out": {
          "count": 9,
          "fields": ["p_up_toto", "conf_toto", "med_xau", "med_jpy", "med_corr_50", "q10_xau", "q90_xau", "q10_jpy", "q90_jpy"],
          "space": "price"
        },
        "quantile_forecast": {
          "shape": "9 x 12",
          "space": "price",
          "consumed_by": "L4_5_entry_zone"
        }
      }
    },

    "L3": {
      "name": "EVIDENCE_POOLING",
      "in": ["p_up_chronos", "p_up_toto", "EnrichedBar", "MarketContext", "corr_matrix"],
      "evidence_base": {
        "count": 4,
        "items": ["p_chronos", "p_toto", "candle_pattern", "chart_pattern"]
      },
      "evidence_group": {
        "count": 2,
        "momentum_group": {
          "formula": "softAND(rsi, macd, cci, roc, mfi; w)",
          "members": ["rsi_14", "macd_hist", "cci_20", "roc_10", "mfi_14"]
        },
        "structure_group": {
          "formula": "softAND(sr_prox, corr_cross, adx, vwap; w)",
          "members": ["sr_proximity", "corr_cross", "adx_14", "vwap_dist"]
        },
        "softAND": "prod(x_i ^ w_i) with sum(w_i) = 1"
      },
      "selector": {
        "corr_on": "evidence_items (6: p_chronos, p_toto, candle_pattern, chart_pattern, momentum_group, structure_group)",
        "max_pairwise_corr_lt_0_30": "Noisy-OR",
        "max_pairwise_corr_between_0_30_and_0_60": "Log-odds",
        "max_pairwise_corr_gt_0_60": "Soft-AND"
      },
      "out": {
        "P_pool": "0..1",
        "evidence": 6,
        "evidence_weights": 6,
        "pooling_method_used": "enum",
        "agree_flag": "sign(p_up_chronos - 0.5) == sign(p_up_toto - 0.5)",
        "agree_flag_rationale": "computed here since L3 has both model outputs"
      }
    },

    "L4": {
      "name": "TABICL_META_LEARNER",
      "runtime": "zsfm-tabicl",
      "model": "models/tabicl-Q8_0.gguf",
      "quant": "Q8_0",
      "in": {
        "X_query": {
          "count": 42,
          "components": {
            "P_pool": 1,
            "evidence": 6,
            "agree_flag": 1,
            "tabicl_static": 15,
            "chronos_out": 10,
            "toto_out": 9
          }
        },
        "support_set": {
          "count_per_symbol": 300,
          "type": "sliding",
          "label_horizon": 10,
          "label_horizon_minutes": 50,
          "label_method": "triple_barrier",
          "barrier": "±2 × ATR",
          "class_balance": {
            "mode": "balanced_weights",
            "min_class_samples": 30,
            "resample_if_under": true
          }
        }
      },
      "labels": ["Up", "Down", "Neutral"],
      "derived": {
        "P_up": "P(Up) + 0.5 * P(Neutral)",
        "confidence": "1 - P(Neutral)",
        "confidence_class": "max(P(Up), P(Down), P(Neutral))",
        "rationale": "directional confidence = 1-P(Neutral); prevents neutral-heavy predictions from passing conf_min"
      },
      "cold_start_stages": {
        "stage_0": {
          "support_range": [0, 100],
          "P_up": "P_pool",
          "conf": 0
        },
        "stage_1": {
          "support_range": [100, 300],
          "P_up": "w * P_pool + (1 - w) * P_tabicl",
          "w": "max(0, (300 - support) / 200)",
          "conf": "min(1, support / 300)"
        },
        "stage_2": {
          "support_range": [300, "inf"],
          "P_up": "P_tabicl",
          "conf": "P_tabicl_conf"
        }
      },
      "refit": {
        "trigger": "(every 50 bars) OR (Sharpe(20) < 0.5 AND closed_trades >= 5)",
        "every_bars": 50,
        "every_minutes": 250,
        "sharpe_window": "20 closed trades (PnL%)",
        "min_trades": 5
      },
      "out": {
        "P_up": "0..1",
        "confidence": "0..1",
        "label": "enum",
        "cold_start": "bool",
        "cold_start_stage": "enum {0, 1, 2}",
        "top_features": 5,
        "evidence_weights": 6
      }
    },

    "L4_6": {
      "name": "INTERPRETATION_LAYER",
      "mode": "advisory_only",
      "gate": false,
      "in": ["tabicl", "chronos_out", "toto_out", "MarketContext", "ParameterSet"],
      "agreement_analyzer": {
        "p_vec": ["p_up_chronos", "p_up_toto", "p_up_tabicl"],
        "agreement_score": "1 - std(p_vec) / 0.5",
        "dispersion": "max(p_vec) - min(p_vec)",
        "conflict_flags": {
          "direction_conflict": "sign mismatch of (p_vec - 0.5)",
          "confidence_conflict": "max_conf - min_conf > 0.30",
          "regime_conflict": "label vs regime mismatch"
        }
      },
      "explanation_extractor": {
        "source": "TabICL-MCP",
        "top_features": 5,
        "counterfactual": {
          "fields": ["feature", "delta_needed", "effect_on_P_up"]
        }
      },
      "narrative_composer": {
        "engine": "jinja2",
        "method": "rule-based",
        "reason_code_enum": ["trend_continuation", "range_rejection", "breakout_confirmed", "liquidity_sweep", "momentum_divergence", "structure_break"],
        "scenario": {
          "fields": ["bull", "base", "bear"],
          "source": "chronos/toto quantiles"
        }
      },
      "out": {
        "interpretation": {
          "agreement_score": "float",
          "dispersion": "float",
          "conflict_flags": "array",
          "top_features": "array",
          "counterfactual": "object",
          "scenario": "object",
          "narrative_text": "string",
          "reason_code": "enum"
        }
      }
    },

    "L4_5": {
      "name": "SETUP_GENERATOR_AND_PORTFOLIO_ARBITER",
      "in": ["P_up", "conf", "label", "MarketContext", "ParameterSet", "chronos_out", "toto_out", "price", "open_setups", "open_positions", "interpretation"],
      "gate": {
        "cold_start_stage_0": "-> empty",
        "cold_start_stage_1": "-> allow; size_multiplier = 0.5",
        "conf_lt_conf_min": "-> empty",
        "abs_P_up_minus_0_5_lt_min_edge": "-> empty",
        "daily_dd_gt_limit": "-> empty",
        "weekly_dd_gt_limit": "-> empty",
        "kill_switch_active": "-> empty"
      },
      "entry_zone": {
        "primary": "[q25_toto_price, q75_toto_price]",
        "toto_quantile_values": [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9],
        "q25_interpolation": {
          "formula": "linear_interp(toto_quantiles, 0.25)",
          "simplified": "0.5*(q20 + q30)",
          "method": "linear_between_adjacent"
        },
        "q75_interpolation": {
          "formula": "linear_interp(toto_quantiles, 0.75)",
          "simplified": "0.5*(q70 + q80)",
          "method": "linear_between_adjacent"
        },
        "cross_check": "[q25_chronos_price, q75_chronos_price] (exact from 21-grid)",
        "shrink_rule": "if |median_toto - median_chronos| > 1.5 * ATR then intersection(primary, cross_check)"
      },
      "initial_sl": {
        "formula": "entry_zone_far ∓ atr_mult * ATR(14) (minus for buy, plus for sell)",
        "precheck": "sl_distance >= SymbolConfig.stop_level + spread_avg",
        "on_fail": "-> empty (no setup)"
      },
      "trigger_set_regime_conditional": {
        "Trending": ["Breakout(BB/SR)", "Retest(SR)", "Momentum(RSI>65+MACD>0)", "Structure(BOS)"],
        "Ranging": ["ZoneTouch(SR±0.2*ATR)", "Momentum", "CandleClose(engulf/hammer)"],
        "Volatile": ["Breakout", "VolumeSpike(>2σ)", "SpreadNormalize(<1.5*avg)"],
        "Calm": ["Breakout", "TimeWindow(session_open±30min)", "CandleClose"]
      },
      "primary_trigger": "trigger_set[0]",
      "partial_policy": {
        "if_lot_lt_4x_lot_min": "FULL_TO_3R",
        "else": "SPLIT_50_30_20"
      },
      "out": {
        "TradeSetup": {
          "fields": ["direction", "entry_zone_lo", "entry_zone_hi", "initial_sl", "sl_distance", "trigger_set", "primary_trigger", "expire_bars", "max_slippage", "confidence", "setup_id", "setup_regime", "partial_policy", "size_multiplier"]
        }
      },
      "arbiter": {
        "slot_per_symbol": 1,
        "max_concurrent_global": 2,
        "priority": "abs(P_up - 0.5) * conf",
        "cross_guard": "corr_50 > 0.40 -> forbid same-direction dual position",
        "tie_break": "XAUUSD > USDJPY"
      },
      "state_machine": ["IDLE", "ARMED", "TRIGGERED", "EXPIRED", "CANCELLED"]
    },

    "L4_7": {
      "name": "TRIGGER_ENGINE",
      "logic": "OR",
      "in": ["TradeSetup", "RawBar", "MarketContext", "P_up", "spread", "price"],
      "cancel_rules": {
        "opposite_p_up": {
          "buy": "P_up < cancel_thr_buy",
          "sell": "P_up > cancel_thr_sell"
        },
        "regime_changed": "regime != setup_regime AND 20-bar hysteresis",
        "corr_broken": {
          "rule": "corr_50(XAUUSD, USDJPY) > -0.20 for 2 consecutive bars",
          "rationale": "cross-pair negative correlation weakened; cancel hedged assumption",
          "source": "L1_6.corr_exit_thr"
        },
        "atr_regime_shift": "ATR(t) / ATR(t-20) not in [0.5, 1.5]"
      },
      "new_decision_action": ["KEEP", "REPLACE", "CANCEL"],
      "expire_action": "EXPIRED when bars_since_setup >= expire_bars",
      "trigger_action": "first satisfied trigger in trigger_set -> TRIGGERED",
      "fill_price": {
        "market_triggers": {
          "applies_to": ["Breakout", "Momentum", "VolumeSpike", "BOS"],
          "formula": "next_bar_open + half_spread + slippage"
        },
        "limit_triggers": {
          "applies_to": ["ZoneTouch", "Retest", "SR"],
          "formula": "trigger_price + half_spread + slippage"
        },
        "slippage_cap": "max_slippage"
      },
      "out": {
        "TriggerEvent": {
          "fields": ["trigger_name", "fill_price", "slippage", "ts", "setup_id"]
        }
      },
      "state_machine": ["ARMED", "TRIGGERED", "EXPIRED", "CANCELLED"]
    },

    "L5": {
      "name": "EXECUTION_AND_RISK",
      "in": ["TriggerEvent", "TradeSetup", "ParameterSet", "SymbolConfig", "Portfolio"],
      "formulas": {
        "eff_risk_pct": "min(risk_pct, max_loss)",
        "risk_amount": "balance * eff_risk_pct",
        "lot_size": "risk_amount / (sl_distance * contract * quote_to_acct[bar.symbol])",
        "lot_size_round": "round_down(lot_size, lot_step)",
        "lot_size_cap": "min(lot_size, lot_max)",
        "lot_size_final": "round_down(lot_size_cap * size_multiplier, lot_step)",
        "size_multiplier_note": "0.5 during cold_start_stage_1, 1.0 otherwise"
      },
      "checks": {
        "lot_min": "lot_size_final >= lot_min else NO_TRADE",
        "margin_check": "free_margin >= required_margin + margin_buffer",
        "stop_level_check": "sl_distance >= stop_level"
      },
      "order_send": {
        "retry_count": 3,
        "retry_interval_ms": 500,
        "dev_cap": "max_slippage",
        "on_fail": "CANCELLED"
      },
      "out": {
        "Position": {
          "fields": ["ticket", "symbol", "direction", "entry", "lot", "sl", "r_state", "be_set", "setup_id"]
        }
      }
    },

    "L6": {
      "name": "POSITION_MANAGER",
      "in": ["Position", "ParameterSet"],
      "partial_policies": {
        "SPLIT_50_30_20": {
          "1R": "close 50%",
          "1.5R": "SL -> BE",
          "2R": "close 30%",
          "3R": "close 20% + trail"
        },
        "FULL_TO_3R": {
          "partials": "none",
          "1.5R": "SL -> BE",
          "3R": "exit OR trail"
        }
      },
      "trailing": {
        "method": "ATR + Chandelier(3.0 * ATR)"
      },
      "exit_conditions": {
        "structure_break": {
          "against_buy": "L1_5.chart.bos_bear OR L1_5.chart.choch_bear",
          "against_sell": "L1_5.chart.bos_bull OR L1_5.chart.choch_bull"
        },
        "sr_touch": {
          "rule": "L1_5.sr.dist_to_nearest_sr <= 0.2 * ATR(14)",
          "direction": "TP side of position"
        },
        "time_exit": "bars_since_entry >= time_exit_bars (96 = 8h at M5)"
      },
      "out": {
        "PositionUpdate": {
          "fields": ["action", "new_sl", "close_lot", "realized_pnl", "reason"]
        }
      }
    },

    "L7": {
      "name": "STORAGE_AND_AUDIT",
      "tables": {
        "raw_bars": {
          "columns": ["symbol", "ts", "open", "high", "low", "close", "volume", "timeframe"],
          "indexes": ["(symbol, ts)"]
        },
        "enriched_bars": {
          "columns": ["symbol", "ts", "features_json"]
        },
        "market_context": {
          "columns": ["symbol", "ts", "regime", "trend_json", "sr_json", "session", "chart_json"]
        },
        "dynamic_params": {
          "columns": ["symbol", "ts", "params_json"]
        },
        "decisions": {
          "columns": ["decision_id PK", "symbol", "ts", "p_up", "confidence", "label", "cold_start", "cold_start_stage", "model_version", "feature_version", "seed", "support_hash", "interpretation_json"],
          "unique": ["(symbol, ts)"],
          "indexes": ["(symbol, ts)"]
        },
        "setups": {
          "columns": ["setup_id PK", "symbol", "created_ts", "direction", "entry_low", "entry_high", "initial_sl", "sl_distance", "primary_trigger", "expire_bars", "max_slippage", "confidence", "setup_regime", "partial_policy", "size_multiplier", "status"],
          "indexes": ["(status, symbol)"]
        },
        "setup_events": {
          "columns": ["event_id PK", "setup_id FK", "event_type", "trigger_name", "fill_price", "slippage", "ts"],
          "terminal_event_constraint": {
            "rule": "exactly one of {TRIGGERED, EXPIRED, CANCELLED} per setup_id",
            "enforce_by": "UNIQUE partial index on (setup_id) WHERE event_type IN ('TRIGGERED','EXPIRED','CANCELLED')"
          }
        },
        "orders": {
          "columns": ["order_id PK", "setup_id FK", "ticket", "retry_count", "dev", "status", "ts"]
        },
        "positions": {
          "columns": ["position_id PK", "order_id FK", "setup_id FK", "entry", "lot", "sl_initial", "sl_current", "r_state", "be_set", "open_price", "open_ts", "close_price", "close_ts"]
        },
        "trades": {
          "columns": ["trade_id PK", "position_id FK", "close_lot", "price", "pnl", "reason", "ts"]
        }
      }
    }
  },

  "output": {
    "modes": ["live", "backtest"],
    "artifacts": ["report", "equity_curve", "audit_log"],
    "ui_panels": ["setup_card", "trigger_log", "portfolio_panel", "audit_view"]
  },

  "verification": {
    "bit_exact": {
      "tolerance": 1e-5,
      "desc": "zsfm output vs PyTorch reference for fixed seed"
    },
    "gguf_smoke": {
      "desc": "Load each Q8_0 GGUF, run 1 inference, assert shape"
    },
    "checksum_verify_on_start": true,
    "crate_audit": {
      "cargo_deny": true,
      "cargo_audit": true,
      "duplicate_policy": "warn",
      "yanked_policy": "deny"
    }
  },

  "notes": [
    "Моделиуд: Chronos-2, Toto-2, TabICL v2 — бүгд zsfm-rs-ийн crate-уудаар ажиллана.",
    "Quantization: Q8_0 (block-32). Q5_K_M-ээс ~4× том (~860 MB нийт) боловч эх checkpoint-той bit-exact (~1e-6).",
    "Backend: candle-* 0.11 (zsfm-rs дотооддоо ашигладаг). CPU / CUDA / Metal.",
    "Rust crates: candle-* 0.11, ndarray 0.17, clap 4.6, tokio 1.53, tracing 0.1.41, rust_decimal 1.41, dashmap 6.2, parking_lot 0.12.5, rayon 1.11, metrics 0.24.6.",
    "sqlx 0.8.6 дээр үлдэв — sqlx 0.9 нь Rust 1.94 MSRV шаарддаг (одоогийн MSRV 1.78-аас дээш).",
    "M15 → M5: timing параметрүүд per-bar утгаараа хадгалагдсан; time_exit_bars=96 нь M5-д 8 цаг (M15-д 24 цаг байсан). 24 цаг хадгалахыг хүсвэл 288 болго.",
    "Warmup 600 бар = 50 цаг M5 түүх (~2 арилжааны өдөр).",
    "Live inference 600 ms/bar = M5-ийн 0.20%. CUDA/Metal ~2–3× хурдан.",
    "zsfm-rs нь Python runtime, PyTorch, ONNX шаарддаггүй — зөвхөн Rust + GGUF + candle.",
    "TabICL in-context learning — zsfm-tabicl нь GGUF дотор attention mask-аар бүрэн дэмждэг; python-sidecar шаардлагагүй.",
    "Мөнгө/үнийн тооцоонд rust_decimal, feature/магадлалд f64/f32.",
    "L4_6 (Interpretation) нь advisory — pipeline-ийг блоклохгүй.",
    "L7 setup_events дээр (setup_id) UNIQUE partial index — terminal event-ийг баталгаажуулна.",
    "Feature-уудыг deterministic (seeded) болгож golden test-ээр хамгаална.",
    "GGUF integrity: SHA256SUMS + manifest.json-оор startup үед шалгана.",
    "CI: cargo fmt/clippy/test/build + cargo deny + cargo audit + bit-exact test."
  ]
}