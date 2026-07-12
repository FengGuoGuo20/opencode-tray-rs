pub mod opencode;
pub mod ccswitch;
pub mod workbuddy;
pub mod hermes;
pub mod zcode;

use std::sync::Arc;
use tokio::sync::RwLock;

use crate::commands::usage::UsageStats;

/// 全局应用状态
pub struct AppState {
    /// 缓存的今日用量数据
    pub today_stats: Arc<RwLock<UsageStats>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            today_stats: Arc::new(RwLock::new(UsageStats::default())),
        }
    }
}
