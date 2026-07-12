//! 设置读写命令模块

use serde::{Deserialize, Serialize};
use tauri_plugin_store::StoreExt;

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// 刷新间隔（秒）
    pub refresh_interval_secs: u64,
    /// 悬浮条显示模式
    pub tray_display_mode: String,
    /// 汇率 USD→CNY
    pub usd_to_cny_rate: f64,
    /// 各数据源路径覆盖
    pub opencode_db_path: Option<String>,
    pub ccswitch_db_path: Option<String>,
    pub workbuddy_dir_path: Option<String>,
    pub hermes_db_path: Option<String>,
    pub zcode_db_path: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            refresh_interval_secs: 30,
            tray_display_mode: "token_mem".to_string(),
            usd_to_cny_rate: 7.2,
            opencode_db_path: None,
            ccswitch_db_path: None,
            workbuddy_dir_path: None,
            hermes_db_path: None,
            zcode_db_path: None,
        }
    }
}

const STORE_FILE: &str = "settings.json";
const SETTINGS_KEY: &str = "settings";

/// 获取应用设置
#[tauri::command]
pub async fn get_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    let store = app
        .store_builder(STORE_FILE)
        .build()
        .map_err(|e| format!("打开设置存储失败: {e}"))?;

    let settings = store
        .get(SETTINGS_KEY)
        .and_then(|v| serde_json::from_value::<AppSettings>(v.clone()).ok())
        .unwrap_or_default();

    Ok(settings)
}

/// 保存应用设置
#[tauri::command]
pub async fn save_settings(
    app: tauri::AppHandle,
    settings: AppSettings,
) -> Result<(), String> {
    let store = app
        .store_builder(STORE_FILE)
        .build()
        .map_err(|e| format!("打开设置存储失败: {e}"))?;

    let value = serde_json::to_value(&settings).map_err(|e| format!("序列化设置失败: {e}"))?;
    store.set(SETTINGS_KEY, value);
    store.save().map_err(|e| format!("保存设置失败: {e}"))?;

    Ok(())
}
