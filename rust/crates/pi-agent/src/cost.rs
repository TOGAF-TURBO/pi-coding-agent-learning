//! 费用估算 — 基于已知模型定价的 API 成本追踪。
//!
//! 定价数据为近似值，仅供用户参考。
//! 实际费用以 provider 账单为准。

/// 模型定价（每百万 token）。
#[derive(Debug, Clone)]
pub struct ModelPricing {
    pub input_per_m: f64,
    pub output_per_m: f64,
}

/// 已知模型定价表（美元/百万 token）。
pub fn get_pricing(model: &str) -> ModelPricing {
    let model_lower = model.to_lowercase();

    // Anthropic Claude
    if model_lower.contains("claude-4-opus") || model_lower.contains("claude-opus-4") {
        return ModelPricing {
            input_per_m: 15.0,
            output_per_m: 75.0,
        };
    }
    if model_lower.contains("claude-4-sonnet") || model_lower.contains("claude-sonnet-4") {
        return ModelPricing {
            input_per_m: 3.0,
            output_per_m: 15.0,
        };
    }
    if model_lower.contains("claude-3.7-sonnet") || model_lower.contains("claude-3-7") {
        return ModelPricing {
            input_per_m: 3.0,
            output_per_m: 15.0,
        };
    }
    if model_lower.contains("claude-3.5-sonnet") || model_lower.contains("claude-3-5") {
        return ModelPricing {
            input_per_m: 3.0,
            output_per_m: 15.0,
        };
    }
    if model_lower.contains("claude-3-haiku") {
        return ModelPricing {
            input_per_m: 0.25,
            output_per_m: 1.25,
        };
    }

    // OpenAI GPT
    if model_lower.contains("gpt-4o") && !model_lower.contains("mini") {
        return ModelPricing {
            input_per_m: 2.5,
            output_per_m: 10.0,
        };
    }
    if model_lower.contains("gpt-4o-mini") {
        return ModelPricing {
            input_per_m: 0.15,
            output_per_m: 0.6,
        };
    }
    if model_lower.contains("o3") {
        return ModelPricing {
            input_per_m: 2.0,
            output_per_m: 8.0,
        };
    }
    if model_lower.contains("o4-mini") {
        return ModelPricing {
            input_per_m: 1.1,
            output_per_m: 4.4,
        };
    }

    // Google Gemini
    if model_lower.contains("gemini-2.5-pro") {
        return ModelPricing {
            input_per_m: 1.25,
            output_per_m: 10.0,
        };
    }
    if model_lower.contains("gemini-2.5-flash") {
        return ModelPricing {
            input_per_m: 0.15,
            output_per_m: 0.6,
        };
    }

    // DeepSeek
    if model_lower.contains("deepseek-chat") || model_lower.contains("deepseek-v3") {
        return ModelPricing {
            input_per_m: 0.27,
            output_per_m: 1.1,
        };
    }
    if model_lower.contains("deepseek-reasoner") || model_lower.contains("deepseek-r1") {
        return ModelPricing {
            input_per_m: 0.55,
            output_per_m: 2.19,
        };
    }

    // GLM
    if model_lower.contains("glm") {
        return ModelPricing {
            input_per_m: 0.5,
            output_per_m: 0.5,
        };
    }

    // 默认：$3/$15（Claude Sonnet 级别）
    ModelPricing {
        input_per_m: 3.0,
        output_per_m: 15.0,
    }
}

/// 估算费用。
pub fn estimate_cost(model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
    let pricing = get_pricing(model);
    let input_cost = (input_tokens as f64 / 1_000_000.0) * pricing.input_per_m;
    let output_cost = (output_tokens as f64 / 1_000_000.0) * pricing.output_per_m;
    input_cost + output_cost
}

/// 格式化费用为可读字符串。
pub fn format_cost(cost: f64) -> String {
    if cost < 0.001 {
        format!("${:.4}", cost)
    } else {
        format!("${:.2}", cost)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_pricing() {
        let p = get_pricing("claude-sonnet-4-20250514");
        assert_eq!(p.input_per_m, 3.0);
        assert_eq!(p.output_per_m, 15.0);
    }

    #[test]
    fn gpt4o_pricing() {
        let p = get_pricing("gpt-4o-2024-08-06");
        assert_eq!(p.input_per_m, 2.5);
    }

    #[test]
    fn glm_pricing() {
        let p = get_pricing("glm-5.1");
        assert_eq!(p.input_per_m, 0.5);
    }

    #[test]
    fn unknown_model_default() {
        let p = get_pricing("some-unknown-model");
        assert_eq!(p.input_per_m, 3.0);
    }

    #[test]
    fn cost_estimation() {
        let cost = estimate_cost("gpt-4o", 1_000_000, 500_000);
        // $2.5 + $5.0 = $7.50
        assert!((cost - 7.5).abs() < 0.01);
    }

    #[test]
    fn format_small_cost() {
        assert_eq!(format_cost(0.0005), "$0.0005");
        assert_eq!(format_cost(0.05), "$0.05");
        assert_eq!(format_cost(1.5), "$1.50");
    }

    #[test]
    fn deepseek_pricing() {
        let p = get_pricing("deepseek-chat");
        assert_eq!(p.input_per_m, 0.27);
    }
}
