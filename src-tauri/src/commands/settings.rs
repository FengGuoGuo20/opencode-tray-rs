//! 设置读写命令模块

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Manager};
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
    /// 开机自启
    #[serde(default)]
    pub start_with_windows: bool,
    /// 悬浮条位置（逻辑像素）
    #[serde(default)]
    pub floating_bar_left: Option<f64>,
    #[serde(default)]
    pub floating_bar_top: Option<f64>,
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
            start_with_windows: false,
            floating_bar_left: None,
            floating_bar_top: None,
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
/// 注册表项名（独立命名，避免与旧版 WPF 的 OpenCodeTray 项互相覆盖）
const STARTUP_APP_NAME: &str = "OpenCodeTrayRS";

/// 获取当前可执行文件路径
fn current_exe_path() -> Option<String> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
}

/// 应用开机自启设置（Windows 注册表 HKCU\...\Run）
#[cfg(target_os = "windows")]
pub fn apply_startup_setting(enabled: bool) -> Result<(), String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run")
        .map_err(|e| format!("打开开机自启注册表失败: {e}"))?;

    if enabled {
        let exe = current_exe_path().ok_or_else(|| "无法获取当前程序路径".to_string())?;
        let value = format!("\"{exe}\"");
        key.set_value(STARTUP_APP_NAME, &value)
            .map_err(|e| format!("写入开机自启失败: {e}"))?;
    } else {
        let _ = key.delete_value(STARTUP_APP_NAME);
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn apply_startup_setting(_enabled: bool) -> Result<(), String> {
    Ok(())
}

/// 启动时校正注册表路径（exe 搬家后仍能自启）
#[cfg(target_os = "windows")]
pub fn ensure_startup_path_valid(enabled: bool) {
    if !enabled {
        return;
    }
    let Some(exe) = current_exe_path() else {
        return;
    };
    let expected = format!("\"{exe}\"");

    use winreg::enums::*;
    use winreg::RegKey;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let Ok((key, _)) = hkcu.create_subkey(r"Software\Microsoft\Windows\CurrentVersion\Run") else {
        return;
    };
    let existing: String = key.get_value(STARTUP_APP_NAME).unwrap_or_default();
    if !existing.eq_ignore_ascii_case(&expected) {
        let _ = key.set_value(STARTUP_APP_NAME, &expected);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn ensure_startup_path_valid(_enabled: bool) {}

/// 从 store 同步加载设置
pub fn load_settings_sync(app: &tauri::AppHandle) -> AppSettings {
    let store = match app.store_builder(STORE_FILE).build() {
        Ok(s) => s,
        Err(_) => return AppSettings::default(),
    };
    store
        .get(SETTINGS_KEY)
        .and_then(|v| serde_json::from_value::<AppSettings>(v.clone()).ok())
        .unwrap_or_default()
}

/// 将设置写入 store（不处理自启）
fn persist_settings(app: &tauri::AppHandle, settings: &AppSettings) -> Result<(), String> {
    let store = app
        .store_builder(STORE_FILE)
        .build()
        .map_err(|e| format!("打开设置存储失败: {e}"))?;
    let value = serde_json::to_value(settings).map_err(|e| format!("序列化设置失败: {e}"))?;
    store.set(SETTINGS_KEY, value);
    store.save().map_err(|e| format!("保存设置失败: {e}"))?;
    Ok(())
}

/// 应用设置到运行时（路径/刷新间隔/通知前端）
pub async fn apply_runtime_settings(
    app: &tauri::AppHandle,
    settings: &AppSettings,
) -> Result<(), String> {
    crate::services::set_runtime_settings(settings.clone());

    if let Some(state) = app.try_state::<crate::services::AppState>() {
        {
            let mut guard = state.settings.write().await;
            *guard = settings.clone();
        }
        let interval = settings.refresh_interval_secs.clamp(5, 600);
        let _ = state.refresh_interval_tx.send(interval);
    }

    let _ = app.emit("settings-updated", settings);
    Ok(())
}

/// 获取应用设置
#[tauri::command]
pub async fn get_settings(app: tauri::AppHandle) -> Result<AppSettings, String> {
    Ok(load_settings_sync(&app))
}

/// 保存应用设置
#[tauri::command]
pub async fn save_settings(app: tauri::AppHandle, mut settings: AppSettings) -> Result<(), String> {
    // 规范化
    settings.refresh_interval_secs = settings.refresh_interval_secs.clamp(5, 600);
    if settings.usd_to_cny_rate <= 0.0 {
        settings.usd_to_cny_rate = 7.2;
    }

    apply_startup_setting(settings.start_with_windows)?;
    persist_settings(&app, &settings)?;
    apply_runtime_settings(&app, &settings).await?;

    // 立即按新设置刷新一次悬浮条
    let _ = crate::commands::usage::refresh_usage_data(&app).await;

    Ok(())
}

/// 仅更新并保存悬浮条位置
pub fn save_floating_bar_position(app: &tauri::AppHandle, x: f64, y: f64) -> Result<(), String> {
    let mut settings = load_settings_sync(app);
    settings.floating_bar_left = Some(x);
    settings.floating_bar_top = Some(y);
    persist_settings(app, &settings)?;
    crate::services::set_runtime_settings(settings);
    Ok(())
}

/// 启动时加载设置并校正自启路径
pub fn load_and_apply_startup(app: &tauri::AppHandle) -> AppSettings {
    let settings = load_settings_sync(app);
    ensure_startup_path_valid(settings.start_with_windows);
    crate::services::set_runtime_settings(settings.clone());
    settings
}
