//! 窗口控制命令模块

use tauri::Manager;

/// 显示主面板窗口
#[tauri::command]
pub async fn show_panel(app: tauri::AppHandle) -> Result<(), String> {
    crate::show_main_window(&app)
}

/// 切换悬浮条显示/隐藏
#[tauri::command]
pub async fn toggle_floating_bar(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(crate::FLOATING_BAR_WINDOW_LABEL) {
        let visible = window
            .is_visible()
            .map_err(|e| format!("检查窗口可见性失败: {e}"))?;
        if visible {
            crate::hide_floating_bar(&app)
        } else {
            crate::show_floating_bar(&app)
        }
    } else {
        crate::show_floating_bar(&app)
    }
}

/// 设置悬浮条位置
#[tauri::command]
pub async fn set_floating_bar_position(
    app: tauri::AppHandle,
    x: f64,
    y: f64,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(crate::FLOATING_BAR_WINDOW_LABEL) {
        window
            .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: x.round() as i32,
                y: y.round() as i32,
            }))
            .map_err(|e| format!("设置悬浮条位置失败: {e}"))?;

        // TODO: 保存位置到 Store
    }
    Ok(())
}
