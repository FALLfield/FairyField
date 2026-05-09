//! 自主成长模块
//!
//! 经验沉淀、技能管理、用户画像。

pub mod commands;
pub mod engine;
pub mod skill;

pub use commands::GrowthState;
pub use engine::GrowthEngine;
pub use skill::SkillManager;
