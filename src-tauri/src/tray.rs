//! 系统托盘菜单管理模块
//!
//! 负责系统托盘图标和菜单的创建、更新和事件处理。

use tauri::menu::{MenuBuilder, MenuItem, PredefinedMenuItem};

/// 托盘图标 ID
pub const TRAY_ID: &str = "opencode-tray";

/// 创建托盘菜单
pub fn create_tray_menu(app: &tauri::AppHandle) -> Result<tauri::menu::Menu<tauri::Wry>, tauri::Error> {
    let show_floating = MenuItem::with_id(app, "show_floating", "显示悬浮条", true, None::<&str>)?;
    let open_panel = MenuItem::with_id(app, "open_panel", "打开面板", true, None::<&str>)?;
    let _separator1 = PredefinedMenuItem::separator(app)?;
    let refresh = MenuItem::with_id(app, "refresh", "刷新数据", true, None::<&str>)?;
    let _separator2 = PredefinedMenuItem::separator(app)?;
    let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;

    let menu = MenuBuilder::new(app)
        .item(&show_floating)
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
        "show_floating" => {
            if let Err(e) = crate::show_floating_bar(app) {
                log::error!("显示悬浮条失败: {e}");
            }
        }
        "open_panel" => {
            if let Err(e) = crate::show_main_window(app) {
                log::error!("打开面板失败: {e}");
            }
        }
        "refresh" => {
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                let _ = crate::commands::usage::refresh_usage_data(&app_handle).await;
            });
        }
        "settings" => {
            // TODO: 打开设置窗口
            if let Err(e) = crate::show_main_window(app) {
                log::error!("打开设置失败: {e}");
            }
        }
        "quit" => {
            app.exit(0);
        }
        _ => {
            log::debug!("未处理的托盘菜单事件: {menu_id}");
        }
    }
}
