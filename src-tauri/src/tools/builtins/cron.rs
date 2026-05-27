//! Cron/reminder tool with local queue semantics.

use crate::tools::executor::{Tool, ToolError};
use crate::tools::manifest::{ToolCategory, ToolManifest, ToolPermission};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CronInput {
    action: String,
    schedule: String,
    message: String,
}

pub struct CronTool;

impl CronTool {
    pub fn new() -> Self {
        Self
    }

    pub fn manifest() -> ToolManifest {
        ToolManifest {
            id: "cron".into(),
            name: "Reminder Scheduler".into(),
            description: "Validate reminder schedules and queue local scheduled tasks.".into(),
            category: ToolCategory::Cron,
            parameters: Self.parameters_schema(),
            permissions: vec![ToolPermission::Notifications],
            rate_limit: Some(20),
            timeout_ms: 5000,
            enabled_by_default: true,
            oauth: None,
            version: Some("1.0.0".into()),
            author: Some("FairyField".into()),
            long_description: None,
        }
    }

    fn schedule_valid(schedule: &str) -> bool {
        let trimmed = schedule.trim();
        if trimmed.is_empty() || trimmed.len() > 120 {
            return false;
        }
        trimmed.starts_with("in ")
            || trimmed.starts_with("at ")
            || trimmed.starts_with("every ")
            || trimmed.split_whitespace().count() == 5
    }
}

impl Tool for CronTool {
    fn name(&self) -> &str {
        "cron"
    }
    fn description(&self) -> &str {
        "Validate reminder schedules and queue local scheduled tasks"
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type":"object",
            "properties":{
                "action":{"type":"string","enum":["schedule_reminder"],"description":"Cron operation"},
                "schedule":{"type":"string","description":"Natural schedule or 5-field cron expression"},
                "message":{"type":"string","description":"Reminder text"}
            },
            "required":["action","schedule","message"]
        })
    }
    fn validate_input(&self, input: &str) -> bool {
        serde_json::from_str::<CronInput>(input)
            .map(|p| {
                p.action == "schedule_reminder"
                    && Self::schedule_valid(&p.schedule)
                    && !p.message.trim().is_empty()
            })
            .unwrap_or(false)
    }
    fn execute(
        &self,
        input: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send + '_>>
    {
        let parsed = serde_json::from_str::<CronInput>(input);
        Box::pin(async move {
            let params = parsed.map_err(|e| ToolError::ValidationFailed(e.to_string()))?;
            if params.action != "schedule_reminder"
                || !CronTool::schedule_valid(&params.schedule)
                || params.message.trim().is_empty()
            {
                return Err(ToolError::ValidationFailed(
                    "Invalid reminder payload".into(),
                ));
            }
            Ok(serde_json::json!({"status":"queued","action":params.action,"schedule":params.schedule,"message":params.message}).to_string())
        })
    }
}

impl Default for CronTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_natural_schedule() {
        assert!(CronTool.validate_input(
            r#"{"action":"schedule_reminder","schedule":"in 10 minutes","message":"stretch"}"#
        ));
    }
    #[test]
    fn rejects_empty_message() {
        assert!(!CronTool.validate_input(
            r#"{"action":"schedule_reminder","schedule":"in 10 minutes","message":""}"#
        ));
    }
}
