//! Growth/Skill Tauri IPC 命令
//!
//! 提供成长引擎和技能管理的 Tauri 命令。

use super::engine::GrowthEngine;
use super::skill::SkillManager;
use std::sync::Mutex;
use tauri::State;

/// Growth 子系统管理状态
pub struct GrowthState {
    pub engine: Mutex<GrowthEngine>,
    pub skills: Mutex<SkillManager>,
}

impl GrowthState {
    /// 使用内存数据库创建（用于测试）
    pub fn new_in_memory() -> Result<Self, String> {
        Ok(Self {
            engine: Mutex::new(GrowthEngine::new_in_memory()?),
            skills: Mutex::new(SkillManager::new_in_memory()?),
        })
    }
}

// === 成长引擎命令 ===

#[tauri::command]
pub fn growth_save_experience(
    category: String,
    content: String,
    emotional_valence: Option<f32>,
    state: State<'_, GrowthState>,
) -> Result<i64, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.save_experience(&category, &content, emotional_valence)
}

#[tauri::command]
pub fn growth_get_user_profile(state: State<'_, GrowthState>) -> Result<serde_json::Value, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let profile = engine.get_user_profile()?;
    Ok(serde_json::to_value(profile).unwrap_or_default())
}

#[tauri::command]
pub fn growth_get_recent_experiences(
    limit: Option<usize>,
    state: State<'_, GrowthState>,
) -> Result<Vec<serde_json::Value>, String> {
    let limit = limit.unwrap_or(20);
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let experiences = engine.get_recent_experiences(limit)?;
    Ok(experiences
        .iter()
        .map(|e| serde_json::to_value(e).unwrap_or_default())
        .collect())
}

#[tauri::command]
pub fn growth_get_category_stats(
    state: State<'_, GrowthState>,
) -> Result<Vec<serde_json::Value>, String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    let stats = engine.get_category_stats()?;
    Ok(stats
        .iter()
        .map(|(cat, count)| {
            serde_json::json!({
                "category": cat,
                "count": count,
            })
        })
        .collect())
}

// === 技能管理命令 ===

#[tauri::command]
pub fn skill_list(state: State<'_, GrowthState>) -> Result<Vec<serde_json::Value>, String> {
    let skills = state.skills.lock().map_err(|e| e.to_string())?;
    let list = skills.list_skills()?;
    Ok(list
        .iter()
        .map(|s| serde_json::to_value(s).unwrap_or_default())
        .collect())
}

#[tauri::command]
pub fn skill_view(
    name: String,
    state: State<'_, GrowthState>,
) -> Result<serde_json::Value, String> {
    let skills = state.skills.lock().map_err(|e| e.to_string())?;
    match skills.view_skill(&name)? {
        Some(skill) => Ok(serde_json::to_value(skill).map_err(|e| e.to_string())?),
        None => Err(format!("技能 '{}' 不存在", name)),
    }
}

#[tauri::command]
pub fn skill_create(
    name: String,
    description: String,
    category: String,
    content: String,
    state: State<'_, GrowthState>,
) -> Result<(), String> {
    let skills = state.skills.lock().map_err(|e| e.to_string())?;
    skills.create_skill(&name, &description, &category, &content)
}

#[tauri::command]
pub fn skill_update(
    name: String,
    content: String,
    state: State<'_, GrowthState>,
) -> Result<(), String> {
    let skills = state.skills.lock().map_err(|e| e.to_string())?;
    skills.update_skill(&name, &content)
}

#[tauri::command]
pub fn skill_delete(name: String, state: State<'_, GrowthState>) -> Result<(), String> {
    let skills = state.skills.lock().map_err(|e| e.to_string())?;
    skills.delete_skill(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn growth_state_in_memory() {
        let state = GrowthState::new_in_memory().unwrap();
        // 验证 engine 和 skills 可正常锁定
        let engine = state.engine.lock().unwrap();
        let profile = engine.get_user_profile().unwrap();
        assert_eq!(profile.experience_count, 0);
    }
}
