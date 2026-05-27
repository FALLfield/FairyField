//! 简易定时任务系统
//!
//! 基于固定间隔的定时任务，每秒 tick 一次。

use serde::{Deserialize, Serialize};

/// 定时任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub id: String,
    pub name: String,
    /// 间隔秒数
    pub interval_secs: u64,
    /// 要执行的动作名称
    pub command: String,
    /// 上次执行时间（Unix 秒）
    pub last_run: u64,
    pub enabled: bool,
}

/// 定时任务调度器
pub struct CronScheduler {
    jobs: Vec<CronJob>,
}

impl Default for CronScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl CronScheduler {
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    /// 添加定时任务
    pub fn add_job(&mut self, name: &str, interval_secs: u64, command: &str) -> String {
        let id = format!("cron_{}", self.jobs.len() + 1);
        self.jobs.push(CronJob {
            id: id.clone(),
            name: name.to_string(),
            interval_secs,
            command: command.to_string(),
            last_run: 0,
            enabled: true,
        });
        id
    }

    /// 移除定时任务
    pub fn remove_job(&mut self, id: &str) -> Result<(), String> {
        let before = self.jobs.len();
        self.jobs.retain(|j| j.id != id);
        if self.jobs.len() == before {
            return Err(format!("任务 '{}' 不存在", id));
        }
        Ok(())
    }

    /// 列出所有任务
    pub fn list_jobs(&self) -> &[CronJob] {
        &self.jobs
    }

    /// Tick：检查并返回到期的任务命令列表
    pub fn tick(&mut self) -> Vec<String> {
        let now = now_secs();
        let mut due = Vec::new();
        for job in &mut self.jobs {
            if job.enabled && (now - job.last_run) >= job.interval_secs {
                due.push(job.command.clone());
                job.last_run = now;
            }
        }
        due
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_list() {
        let mut sched = CronScheduler::new();
        let id = sched.add_job("问候", 3600, "send_greeting");
        assert!(!id.is_empty());
        assert_eq!(sched.list_jobs().len(), 1);
    }

    #[test]
    fn remove_job() {
        let mut sched = CronScheduler::new();
        sched.add_job("A", 60, "cmd_a");
        let id = sched.add_job("B", 120, "cmd_b");
        sched.remove_job(&id).unwrap();
        assert_eq!(sched.list_jobs().len(), 1);
    }

    #[test]
    fn tick_fires_due_jobs() {
        let mut sched = CronScheduler::new();
        sched.add_job("即时", 0, "now_cmd");
        let due = sched.tick();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0], "now_cmd");
    }

    #[test]
    fn tick_no_duplicate() {
        let mut sched = CronScheduler::new();
        sched.add_job("慢任务", 99999, "slow_cmd");
        sched.tick();
        let due = sched.tick();
        assert!(due.is_empty());
    }
}
