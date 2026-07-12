mod commands;
mod db;
mod services;
mod tray;

use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

use crate::services::AppState;

/// 悬浮条窗口标签
pub const FLOATING_BAR_WINDOW_LABEL: &str = "floating-bar";
const FLOATING_BAR_WIDTH: f64 = 150.0;
const FLOATING_BAR_HEIGHT: f64 = 26.0;

/// 计算悬浮条默认位置（屏幕右下角任务栏上方，距边缘 24px）
fn fallback_position(app: &tauri::AppHandle) -> (f64, f64) {
    let margin = 24.0;
    if let Ok(Some(monitor)) = app.primary_monitor() {
        let work_area = monitor.work_area();
        let x = work_area.position.x as f64 + work_area.size.width as f64 - FLOATING_BAR_WIDTH - margin;
        let y = work_area.position.y as f64 + work_area.size.height as f64 - FLOATING_BAR_HEIGHT - margin;
        return (x.max(work_area.position.x as f64), y.max(work_area.position.y as f64));
    }
    (margin, margin)
}

/// 检查位置是否在可见显示器范围内
#[allow(dead_code)]
fn is_position_visible(app: &tauri::AppHandle, left: f64, top: f64) -> bool {
    if let Ok(monitors) = app.available_monitors() {
        for monitor in monitors {
            let position = monitor.position();
            let size = monitor.size();
            let min_x = position.x as f64;
            let min_y = position.y as f64;
            let max_x = min_x + size.width as f64 - FLOATING_BAR_WIDTH;
            let max_y = min_y + size.height as f64 - FLOATING_BAR_HEIGHT;
            if left >= min_x && left <= max_x && top >= min_y && top <= max_y {
                return true;
            }
        }
    }
    false
}

/// 解析悬浮条位置：优先使用保存的位置，否则回退到默认位置
fn resolve_floating_bar_position(app: &tauri::AppHandle) -> (f64, f64) {
    // TODO: 从 Store 读取保存的位置
    fallback_position(app)
}

/// 创建或显示悬浮条窗口
pub fn show_floating_bar(app: &tauri::AppHandle) -> Result<(), String> {
    let (left, top) = resolve_floating_bar_position(app);

    // 如果窗口已存在，移动并显示
    if let Some(window) = app.get_webview_window(FLOATING_BAR_WINDOW_LABEL) {
        window
            .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: left.round() as i32,
                y: top.round() as i32,
            }))
            .map_err(|e| format!("移动悬浮条失败: {e}"))?;
        let _ = window.unminimize();
        window.show().map_err(|e| format!("显示悬浮条失败: {e}"))?;
        return Ok(());
    }

    // 创建新的悬浮条窗口
    let window =
        tauri::WebviewWindowBuilder::new(app, FLOATING_BAR_WINDOW_LABEL, tauri::WebviewUrl::default())
            .title("OpenCodeTray Floating Bar")
            .inner_size(FLOATING_BAR_WIDTH, FLOATING_BAR_HEIGHT)
            .min_inner_size(FLOATING_BAR_WIDTH, FLOATING_BAR_HEIGHT)
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .position(left, top)
            .build()
            .map_err(|e| format!("创建悬浮条失败: {e}"))?;

    window
        .show()
        .map_err(|e| format!("显示悬浮条失败: {e}"))?;
    Ok(())
}

/// 隐藏悬浮条窗口
pub fn hide_floating_bar(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(FLOATING_BAR_WINDOW_LABEL) {
        window
            .hide()
            .map_err(|e| format!("隐藏悬浮条失败: {e}"))?;
    }
    Ok(())
}

/// 显示主面板窗口，定位到屏幕右下角
pub fn show_main_window(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        // 定位到屏幕右下角
        if let Ok(Some(monitor)) = app.primary_monitor() {
            let work_area = monitor.work_area();
            let x = work_area.position.x as f64 + work_area.size.width as f64 - 840.0;
            let y = work_area.position.y as f64 + work_area.size.height as f64 - 700.0;
            let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: x.round() as i32,
                y: y.round() as i32,
            }));
        }
        let _ = window.unminimize();
        window.show().map_err(|e| format!("显示主窗口失败: {e}"))?;
        window
            .set_focus()
            .map_err(|e| format!("聚焦主窗口失败: {e}"))?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    // 单实例插件
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // 已有实例运行时，聚焦主窗口
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.show();
                let _ = window.set_focus();
            }
        }));
    }

    builder
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        // 拦截窗口关闭：隐藏到托盘而非退出
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let label = window.label().to_string();
                if label == "main" {
                    // 主窗口关闭时隐藏而非退出
                    api.prevent_close();
                    let _ = window.hide();
                    #[cfg(target_os = "windows")]
                    {
                        let _ = window.set_skip_taskbar(true);
                    }
                }
            }
        })
        .setup(|app| {
            // 创建托盘菜单
            let menu = tray::create_tray_menu(app.handle())?;

            // 构建系统托盘
            let mut tray_builder = TrayIconBuilder::with_id(tray::TRAY_ID)
                .tooltip("OpenCodeTray")
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click { .. } => {
                        let app = tray.app_handle().clone();
                        // 点击托盘图标时刷新数据
                        tauri::async_runtime::spawn(async move {
                            let _ = crate::commands::usage::refresh_usage_data(&app).await;
                        });
                    }
                    _ => {}
                })
                .menu(&menu)
                .on_menu_event(|app, event| {
                    tray::handle_tray_menu_event(app, &event.id.0);
                })
                .show_menu_on_left_click(true);

            // 使用应用默认图标
            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            let _tray = tray_builder.build(app)?;

            // 初始化应用状态
            let app_state = AppState::new();
            app.manage(app_state);

            // 启动悬浮条
            if let Err(e) = show_floating_bar(app.handle()) {
                log::warn!("启动悬浮条失败: {e}");
            }

            // 启动定时刷新（30秒用量 + 2秒内存）
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                crate::commands::usage::start_refresh_timer(app_handle).await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::usage::get_today_stats,
            commands::usage::get_week_stats,
            commands::usage::get_month_stats,
            commands::usage::get_all_time_stats,
            commands::usage::get_daily_usage,
            commands::usage::get_today_model_stats,
            commands::usage::get_memory_usage,
            commands::usage::get_tray_display,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::window::show_panel,
            commands::window::toggle_floating_bar,
            commands::window::set_floating_bar_position,
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用失败");
}
