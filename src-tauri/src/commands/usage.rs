//! 用量查询命令模块
//!
//! 提供 Tauri 命令供前端查询各数据源的 token 用量、内存占用等。

use tauri::Emitter;

use crate::services::AppState;

/// 用量统计结果
#[derive(Debug, Clone, serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UsageStats {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub reasoning_tokens: i64,
    pub cost_usd: f64,
    pub sessions: i64,
}

/// 每日用量
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyUsage {
    pub date: String,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_usd: f64,
}

/// 模型用量
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsage {
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_usd: f64,
    pub sessions: i64,
}

/// 悬浮条显示数据
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayDisplay {
    pub token_text: String,
    pub mem_percent: f64,
    pub cost_text: String,
}

/// 内存使用信息
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    pub usage_percent: f64,
    pub total_gb: f64,
    pub used_gb: f64,
    pub available_gb: f64,
}

/// 合并五数据源的今日统计
fn merge_today_stats() -> UsageStats {
    let mut total = UsageStats::default();
    let oc = crate::services::opencode::get_today_stats();
    let cc = crate::services::ccswitch::get_today_stats();
    let wb = crate::services::workbuddy::get_today_stats();
    let hm = crate::services::hermes::get_today_stats();
    let zc = crate::services::zcode::get_today_stats();

    for s in [&oc, &cc, &wb, &hm, &zc] {
        total.input_tokens += s.input_tokens;
        total.output_tokens += s.output_tokens;
        total.cache_read_tokens += s.cache_read_tokens;
        total.cache_write_tokens += s.cache_write_tokens;
        total.reasoning_tokens += s.reasoning_tokens;
        total.cost_usd += s.cost_usd;
        total.sessions += s.sessions;
    }
    total
}

/// 获取今日用量统计
#[tauri::command]
pub async fn get_today_stats(_state: tauri::State<'_, AppState>) -> Result<UsageStats, String> {
    Ok(merge_today_stats())
}

/// 获取本周用量统计
#[tauri::command]
pub async fn get_week_stats(_state: tauri::State<'_, AppState>) -> Result<UsageStats, String> {
    let total = UsageStats::default();
    // TODO: 合并五数据源周统计
    Ok(total)
}

/// 获取本月用量统计
#[tauri::command]
pub async fn get_month_stats(_state: tauri::State<'_, AppState>) -> Result<UsageStats, String> {
    let total = UsageStats::default();
    // TODO: 合并五数据源月统计
    Ok(total)
}

/// 获取全部用量统计
#[tauri::command]
pub async fn get_all_time_stats(_state: tauri::State<'_, AppState>) -> Result<UsageStats, String> {
    let total = UsageStats::default();
    // TODO: 合并五数据源全部统计
    Ok(total)
}

/// 获取每日用量（最近 N 天）
#[tauri::command]
pub async fn get_daily_usage(
    _state: tauri::State<'_, AppState>,
    days: i32,
) -> Result<Vec<DailyUsage>, String> {
    // TODO: 合并五数据源每日数据
    let _ = days;
    Ok(vec![])
}

/// 获取今日模型分布
#[tauri::command]
pub async fn get_today_model_stats(
    _state: tauri::State<'_, AppState>,
) -> Result<Vec<ModelUsage>, String> {
    // TODO: 合并五数据源模型分布
    Ok(vec![])
}

/// 获取内存使用信息
#[tauri::command]
pub async fn get_memory_usage() -> Result<MemoryInfo, String> {
    let mut sys = sysinfo::System::new();
    sys.refresh_memory();

    let total = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let used = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let available = sys.available_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
    let usage_percent = if total > 0.0 { used / total * 100.0 } else { 0.0 };

    Ok(MemoryInfo {
        usage_percent,
        total_gb: total,
        used_gb: used,
        available_gb: available,
    })
}

/// 获取悬浮条显示数据
#[tauri::command]
pub async fn get_tray_display(_state: tauri::State<'_, AppState>) -> Result<TrayDisplay, String> {
    let today = merge_today_stats();
    let mem = get_memory_usage().await?;

    let token_text = format_tokens(today.input_tokens + today.output_tokens);
    let cost_text = format_cost(today.cost_usd);

    Ok(TrayDisplay {
        token_text,
        mem_percent: mem.usage_percent,
        cost_text,
    })
}

/// 刷新用量数据并推送到前端
pub async fn refresh_usage_data(app: &tauri::AppHandle) -> Result<(), String> {
    let today = merge_today_stats();
    let mem = get_memory_usage().await?;

    let token_text = format_tokens(today.input_tokens + today.output_tokens);

    // 推送更新事件到悬浮条和主面板
    let _ = app.emit("usage-updated", serde_json::json!({
        "tokenText": token_text,
        "memPercent": mem.usage_percent,
        "costUsd": today.cost_usd,
        "inputTokens": today.input_tokens,
        "outputTokens": today.output_tokens,
    }));

    Ok(())
}

/// 启动定时刷新计时器
pub async fn start_refresh_timer(app: tauri::AppHandle) {
    let mut usage_interval = tokio::time::interval(std::time::Duration::from_secs(30));
    let mut mem_interval = tokio::time::interval(std::time::Duration::from_secs(2));

    // 首次立即刷新
    let _ = refresh_usage_data(&app).await;

    loop {
        tokio::select! {
            _ = usage_interval.tick() => {
                if let Err(e) = refresh_usage_data(&app).await {
                    log::warn!("刷新用量数据失败: {e}");
                }
            }
            _ = mem_interval.tick() => {
                if let Ok(mem) = get_memory_usage().await {
                    let _ = app.emit("memory-updated", serde_json::json!({
                        "memPercent": mem.usage_percent,
                    }));
                }
            }
        }
    }
}

/// 格式化 token 数（中文习惯：万/亿）
fn format_tokens(tokens: i64) -> String {
    if tokens >= 1_0000_0000 {
        let v = tokens as f64 / 1_0000_0000.0;
        format!("{:.2}亿", v)
    } else if tokens >= 1_0000 {
        let v = tokens as f64 / 1_0000.0;
        format!("{:.1}万", v)
    } else if tokens >= 1000 {
        let v = tokens as f64 / 1000.0;
        format!("{:.1}K", v)
    } else {
        tokens.to_string()
    }
}

/// 格式化费用
fn format_cost(cost_usd: f64) -> String {
    if cost_usd >= 1.0 {
        format!("${:.2}", cost_usd)
    } else if cost_usd >= 0.01 {
        format!("${:.3}", cost_usd)
    } else if cost_usd > 0.0 {
        format!("${:.4}", cost_usd)
    } else {
        "$0".to_string()
    }
}
