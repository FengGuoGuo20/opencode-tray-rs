//! 系统托盘菜单管理模块
//!
//! 负责系统托盘图标和菜单的创建、更新和事件处理。

use tauri::menu::{MenuBuilder, MenuItem};
use tauri::Manager;

/// 托盘图标 ID
pub const TRAY_ID: &str = "opencode-tray";

/// 创建托盘菜单
pub fn create_tray_menu(app: &tauri::AppHandle) -> Result<tauri::menu::Menu<tauri::Wry>, tauri::Error> {
    let floating_visible = app
        .get_webview_window(crate::FLOATING_BAR_WINDOW_LABEL)
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);
    let floating_label = if floating_visible {
        "隐藏悬浮条"
    } else {
        "显示悬浮条"
    };

    let toggle_floating = MenuItem::with_id(app, "toggle_floating", floating_label, true, None::<&str>)?;
    let open_panel = MenuItem::with_id(app, "open_panel", "打开面板", true, None::<&str>)?;
    let refresh = MenuItem::with_id(app, "refresh", "刷新数据", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = MenuBuilder::new(app)
        .item(&toggle_floating)
        .item(&open_panel)
        .separator()
        .item(&refresh)
        .separator()
        .item(&settings)
        .item(&quit)
        .build()?;

    Ok(menu)
}

/// 处理托盘菜单事件
pub fn handle_tray_menu_event(app: &tauri::AppHandle, menu_id: &str) {
    match menu_id {
        "show_floating" | "toggle_floating" => {
            if let Err(e) = crate::toggle_floating_bar_window(app) {
                log::error!("切换悬浮条失败: {e}");
            }
        }
        "open_panel" => {
            if let Err(e) = crate::show_main_window(app) {
                log::error!("打开面板失败: {e}");
            }
        }
        "settings" => {
            if let Err(e) = crate::show_settings_window(app) {
                log::error!("打开设置失败: {e}");
            }
        }
        "refresh" => {
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                let _ = crate::commands::usage::refresh_usage_data(&app_handle).await;
            });
        }
        "quit" => {
            app.exit(0);
        }
        _ => {
            log::debug!("未处理的托盘菜单事件: {menu_id}");
        }
    }
}
