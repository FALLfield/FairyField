//! 智能模型路由
//!
//! 根据输入复杂度选择合适的模型层级（Fast/Balanced/Powerful）。

use serde::{Deserialize, Serialize};

/// 模型层级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelTier {
    /// 快速响应（Haiku 级）
    Fast,
    /// 平衡（Sonnet 级）
    Balanced,
    /// 深度推理（Opus 级）
    Powerful,
}

/// 路由策略
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoutingStrategy {
    /// 最便宜
    Cheapest,
    /// 最快
    Fastest,
    /// 最好
    Best,
}

/// 模型路由器
pub struct ModelRouter {
    strategy: RoutingStrategy,
}

impl ModelRouter {
    pub fn new(strategy: RoutingStrategy) -> Self {
        Self { strategy }
    }

    pub fn with_default_strategy() -> Self {
        Self::new(RoutingStrategy::Best)
    }

    /// 根据输入特征选择模型层级
    pub fn select_model(&self, input: &str) -> ModelTier {
        match self.strategy {
            RoutingStrategy::Cheapest => ModelTier::Fast,
            RoutingStrategy::Fastest => ModelTier::Fast,
            RoutingStrategy::Best => self.classify_by_complexity(input),
        }
    }

    /// 基于输入复杂度分类
    fn classify_by_complexity(&self, input: &str) -> ModelTier {
        let token_estimate = input.split_whitespace().count();
        let has_reasoning = contains_reasoning_keywords(input);
        let has_code =
            input.contains("fn ") || input.contains("function ") || input.contains("impl ");

        // 长输入 + 需要推理 → Powerful
        if token_estimate > 500 && (has_reasoning || has_code) {
            return ModelTier::Powerful;
        }

        // 中等长度 + 有推理关键词 → Balanced
        if token_estimate > 100 || has_reasoning {
            return ModelTier::Balanced;
        }

        // 短输入 → Fast
        ModelTier::Fast
    }

    /// 获取层级对应的建议模型名称
    pub fn suggested_model(&self, tier: ModelTier) -> &'static str {
        match tier {
            ModelTier::Fast => "claude-haiku-4-5-20251001",
            ModelTier::Balanced => "claude-sonnet-4-6",
            ModelTier::Powerful => "claude-opus-4-6",
        }
    }
}

/// 检测推理相关关键词
fn contains_reasoning_keywords(text: &str) -> bool {
    let keywords = [
        "分析",
        "为什么",
        "如何",
        "原因",
        "解释",
        "分析",
        "compare",
        "why",
        "how",
        "reason",
        "explain",
        "analyze",
        "evaluate",
    ];
    let lower = text.to_lowercase();
    keywords.iter().any(|k| lower.contains(k))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cheapest_always_fast() {
        let router = ModelRouter::new(RoutingStrategy::Cheapest);
        assert_eq!(router.select_model("复杂分析请求"), ModelTier::Fast);
    }

    #[test]
    fn short_input_is_fast() {
        let router = ModelRouter::new(RoutingStrategy::Best);
        assert_eq!(router.select_model("你好"), ModelTier::Fast);
    }

    #[test]
    fn reasoning_keywords_balanced() {
        let router = ModelRouter::new(RoutingStrategy::Best);
        assert_eq!(router.select_model("为什么天是蓝的"), ModelTier::Balanced);
    }

    #[test]
    fn long_code_is_powerful() {
        let router = ModelRouter::new(RoutingStrategy::Best);
        let long = "fn main() { ".repeat(100) + &"analyze ".repeat(400);
        assert_eq!(router.select_model(&long), ModelTier::Powerful);
    }

    #[test]
    fn suggested_model_names() {
        let router = ModelRouter::with_default_strategy();
        assert!(router.suggested_model(ModelTier::Fast).contains("haiku"));
        assert!(router
            .suggested_model(ModelTier::Balanced)
            .contains("sonnet"));
        assert!(router.suggested_model(ModelTier::Powerful).contains("opus"));
    }
}
