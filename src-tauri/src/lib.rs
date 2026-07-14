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
const MAIN_WINDOW_DEFAULT_WIDTH: f64 = 820.0;
const MAIN_WINDOW_DEFAULT_HEIGHT: f64 = 680.0;
const SETTINGS_WINDOW_LABEL: &str = "settings";
const SETTINGS_WINDOW_WIDTH: f64 = 520.0;
const SETTINGS_WINDOW_HEIGHT: f64 = 560.0;

/// 计算悬浮条右下角位置（物理像素，适配 DPI / 工作区 / 任务栏）
///
/// 每次启动（含开机自启）都强制贴到主显示器工作区右下角，
/// 避免恢复旧坐标导致位置错乱或跑到屏幕外。
fn bottom_right_position_physical(app: &tauri::AppHandle) -> (i32, i32) {
    let margin_logical = 24.0;
    if let Ok(Some(monitor)) = app.primary_monitor() {
        let work_area = monitor.work_area();
        let scale = monitor.scale_factor().max(0.5);
        let win_w = FLOATING_BAR_WIDTH * scale;
        let win_h = FLOATING_BAR_HEIGHT * scale;
        let margin = margin_logical * scale;

        let x = work_area.position.x as f64 + work_area.size.width as f64 - win_w - margin;
        let y = work_area.position.y as f64 + work_area.size.height as f64 - win_h - margin;

        // 夹在工作区内，防止负坐标/多显示器边界异常
        let min_x = work_area.position.x as f64;
        let min_y = work_area.position.y as f64;
        let max_x = min_x + work_area.size.width as f64 - win_w;
        let max_y = min_y + work_area.size.height as f64 - win_h;

        let x = x.clamp(min_x, max_x.max(min_x));
        let y = y.clamp(min_y, max_y.max(min_y));
        return (x.round() as i32, y.round() as i32);
    }
    ((margin_logical) as i32, (margin_logical) as i32)
}

/// 创建或显示悬浮条窗口（始终定位到工作区右下角）
pub fn show_floating_bar(app: &tauri::AppHandle) -> Result<(), String> {
    let (x, y) = bottom_right_position_physical(app);
    let pos = tauri::Position::Physical(tauri::PhysicalPosition { x, y });

    // 如果窗口已存在，移动到右下角并显示
    if let Some(window) = app.get_webview_window(FLOATING_BAR_WINDOW_LABEL) {
        window
            .set_position(pos)
            .map_err(|e| format!("移动悬浮条失败: {e}"))?;
        let _ = window.unminimize();
        window.show().map_err(|e| format!("显示悬浮条失败: {e}"))?;
        return Ok(());
    }

    // 创建新的悬浮条窗口
    // Windows 上仅 transparent(true) 不够，还要把窗口/WebView 背景设成 alpha=0
    // 否则会残留默认白底，看起来像不透明块。
    let transparent_bg = tauri::window::Color(0, 0, 0, 0);
    let window =
        tauri::WebviewWindowBuilder::new(app, FLOATING_BAR_WINDOW_LABEL, tauri::WebviewUrl::default())
            .title("OpenCodeTray Floating Bar")
            .inner_size(FLOATING_BAR_WIDTH, FLOATING_BAR_HEIGHT)
            .min_inner_size(FLOATING_BAR_WIDTH, FLOATING_BAR_HEIGHT)
            .resizable(false)
            .decorations(false)
            .transparent(true)
            .background_color(transparent_bg)
            .shadow(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .position(x as f64, y as f64) // builder 接受逻辑/物理混用时以物理为准再 set 一次
            .build()
            .map_err(|e| format!("创建悬浮条失败: {e}"))?;

    // 创建后再用物理坐标钉一次，避免 DPI 下初始 position 偏移
    let _ = window.set_position(pos);
    let _ = window.set_background_color(Some(transparent_bg));
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

/// 切换悬浮条显示
pub fn toggle_floating_bar_window(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(FLOATING_BAR_WINDOW_LABEL) {
        let visible = window
            .is_visible()
            .map_err(|e| format!("检查窗口可见性失败: {e}"))?;
        if visible {
            hide_floating_bar(app)
        } else {
            show_floating_bar(app)
        }
    } else {
        show_floating_bar(app)
    }
}

/// 显示主面板窗口，定位到屏幕右下角（工作区内，留 16px 边距）
pub fn show_main_window(app: &tauri::AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window("main") else {
        return Err("未找到主窗口 'main'".to_string());
    };

    // 关闭时可能设置了 skip_taskbar，显示前恢复
    #[cfg(target_os = "windows")]
    {
        let _ = window.set_skip_taskbar(false);
    }

    // 获取窗口尺寸（优先 outer_size，失败则用配置默认值）
    let win_size = window
        .outer_size()
        .or_else(|_| window.inner_size())
        .unwrap_or(tauri::PhysicalSize::new(
            MAIN_WINDOW_DEFAULT_WIDTH as u32,
            MAIN_WINDOW_DEFAULT_HEIGHT as u32,
        ));
    let mut win_w = win_size.width as f64;
    let mut win_h = win_size.height as f64;
    if win_w < 100.0 || win_h < 100.0 {
        win_w = MAIN_WINDOW_DEFAULT_WIDTH;
        win_h = MAIN_WINDOW_DEFAULT_HEIGHT;
        let _ = window.set_size(tauri::Size::Logical(tauri::LogicalSize::new(win_w, win_h)));
        if let Ok(size) = window.outer_size() {
            win_w = size.width as f64;
            win_h = size.height as f64;
        }
    }

    // 定位到屏幕右下角工作区内
    if let Ok(Some(monitor)) = app.primary_monitor() {
        let work_area = monitor.work_area();
        let margin = 16.0;
        let x = work_area.position.x as f64 + work_area.size.width as f64 - win_w - margin;
        let y = work_area.position.y as f64 + work_area.size.height as f64 - win_h - margin;
        let x = x.max(work_area.position.x as f64 + margin);
        let y = y.max(work_area.position.y as f64 + margin);
        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
            x: x.round() as i32,
            y: y.round() as i32,
        }));
    }

    let _ = window.unminimize();
    window
        .show()
        .map_err(|e| format!("显示主窗口失败: {e}"))?;
    window
        .set_focus()
        .map_err(|e| format!("聚焦主窗口失败: {e}"))?;

    Ok(())
}

/// 显示设置窗口（不存在则创建）
pub fn show_settings_window(app: &tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        let _ = window.unminimize();
        window
            .show()
            .map_err(|e| format!("显示设置窗口失败: {e}"))?;
        window
            .set_focus()
            .map_err(|e| format!("聚焦设置窗口失败: {e}"))?;
        return Ok(());
    }

    let window = tauri::WebviewWindowBuilder::new(
        app,
        SETTINGS_WINDOW_LABEL,
        tauri::WebviewUrl::default(),
    )
    .title("OpenCodeTray 设置")
    .inner_size(SETTINGS_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT)
    .min_inner_size(420.0, 480.0)
    .resizable(true)
    .decorations(false)
    .transparent(true)
    .skip_taskbar(false)
    .center()
    .build()
    .map_err(|e| format!("创建设置窗口失败: {e}"))?;

    window
        .show()
        .map_err(|e| format!("显示设置窗口失败: {e}"))?;
    window
        .set_focus()
        .map_err(|e| format!("聚焦设置窗口失败: {e}"))?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    // 单实例插件
    #[cfg(any(target_os = "macos", target_os = "windows", target_os = "linux"))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = show_main_window(app);
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
        // 拦截窗口关闭：主面板/设置隐藏到托盘而非退出
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let label = window.label().to_string();
                if label == "main" || label == SETTINGS_WINDOW_LABEL {
                    api.prevent_close();
                    let _ = window.hide();
                    #[cfg(target_os = "windows")]
                    if label == "main" {
                        let _ = window.set_skip_taskbar(true);
                    }
                }
            }
        })
        // 全局菜单事件（托盘右键菜单）
        .on_menu_event(|app, event| {
            tray::handle_tray_menu_event(app, event.id.as_ref());
        })
        // 全局托盘图标点击事件
        .on_tray_icon_event(|app, event| {
            if let TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                let _ = show_main_window(app);
            }
        })
        .setup(|app| {
            // 加载设置 + 校正自启
            let settings = commands::settings::load_and_apply_startup(app.handle());

            // 创建托盘菜单
            let menu = tray::create_tray_menu(app.handle())?;

            // 构建系统托盘（必须 manage 住，避免被 drop 后事件失效）
            let mut tray_builder = TrayIconBuilder::with_id(tray::TRAY_ID)
                .tooltip("OpenCodeTray")
                .menu(&menu)
                .show_menu_on_left_click(false);

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            } else {
                let ico_path =
                    std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/icons/icon.ico"));
                if ico_path.exists() {
                    if let Ok(img) = tauri::image::Image::from_path(&ico_path) {
                        tray_builder = tray_builder.icon(img);
                    }
                }
            }

            let tray = tray_builder.build(app)?;
            app.manage(tray);

            // 初始化应用状态（含设置与刷新间隔通道）
            let app_state = AppState::new(settings);
            app.manage(app_state);

            // 启动悬浮条
            if let Err(e) = show_floating_bar(app.handle()) {
                log::warn!("启动悬浮条失败: {e}");
            }

            // 启动定时刷新
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
            commands::usage::get_today_source_stats,
            commands::usage::get_memory_usage,
            commands::usage::get_tray_display,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::window::show_panel,
            commands::window::show_settings,
            commands::window::toggle_floating_bar,
            commands::window::set_floating_bar_position,
        ])
        .run(tauri::generate_context!())
        .expect("运行 Tauri 应用失败");
}
