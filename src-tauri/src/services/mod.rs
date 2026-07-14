pub mod opencode;
pub mod ccswitch;
pub mod workbuddy;
pub mod hermes;
pub mod zcode;
pub mod paths;

use std::sync::{Arc, OnceLock, RwLock as StdRwLock};

use tokio::sync::RwLock;

use crate::commands::settings::AppSettings;
use crate::commands::usage::UsageStats;

/// 进程级当前设置快照（供同步数据源读取路径覆盖）
static RUNTIME_SETTINGS: OnceLock<StdRwLock<AppSettings>> = OnceLock::new();

fn runtime_settings_lock() -> &'static StdRwLock<AppSettings> {
    RUNTIME_SETTINGS.get_or_init(|| StdRwLock::new(AppSettings::default()))
}

/// 初始化/更新运行时设置
pub fn set_runtime_settings(settings: AppSettings) {
    if let Ok(mut guard) = runtime_settings_lock().write() {
        *guard = settings;
    }
}

/// 读取运行时设置快照
pub fn current_settings() -> AppSettings {
    runtime_settings_lock()
        .read()
        .map(|g| g.clone())
        .unwrap_or_default()
}

/// 全局应用状态
pub struct AppState {
    /// 缓存的今日用量数据（None 表示尚未填充，避免与真实 0 混淆）
    pub today_stats: Arc<RwLock<Option<UsageStats>>>,
    /// 当前生效设置（热更新）
    pub settings: Arc<RwLock<AppSettings>>,
    /// 刷新间隔变更通知（秒）
    pub refresh_interval_tx: tokio::sync::watch::Sender<u64>,
    pub refresh_interval_rx: tokio::sync::watch::Receiver<u64>,
}

impl AppState {
    pub fn new(initial_settings: AppSettings) -> Self {
        set_runtime_settings(initial_settings.clone());
        let interval = initial_settings.refresh_interval_secs.clamp(5, 600);
        let (tx, rx) = tokio::sync::watch::channel(interval);
        Self {
            today_stats: Arc::new(RwLock::new(None)),
            settings: Arc::new(RwLock::new(initial_settings)),
            refresh_interval_tx: tx,
            refresh_interval_rx: rx,
        }
    }
}
