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
#[derive(Debug, Clone, serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DailyUsage {
    pub date: String,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cache_read_tokens: i64,
    pub total_cache_write_tokens: i64,
    pub total_reasoning_tokens: i64,
    pub total_cost_usd: f64,
    pub session_count: i64,
}

/// 模型用量
#[derive(Debug, Clone, serde::Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsage {
    pub model: String,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
    pub reasoning_tokens: i64,
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

/// 合并多个 UsageStats（所有字段求和）
fn merge_stats(stats: &[UsageStats]) -> UsageStats {
    let mut total = UsageStats::default();
    for s in stats {
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

/// 合并多个 DailyUsage 列表（按日期分组求和）
fn merge_daily_usage(lists: Vec<Vec<DailyUsage>>) -> Vec<DailyUsage> {
    let mut map: std::collections::HashMap<String, DailyUsage> = std::collections::HashMap::new();
    for list in lists {
        for usage in list {
            let entry = map.entry(usage.date.clone()).or_insert_with(|| DailyUsage {
                date: usage.date.clone(),
                ..Default::default()
            });
            entry.total_input_tokens += usage.total_input_tokens;
            entry.total_output_tokens += usage.total_output_tokens;
            entry.total_cache_read_tokens += usage.total_cache_read_tokens;
            entry.total_cache_write_tokens += usage.total_cache_write_tokens;
            entry.total_reasoning_tokens += usage.total_reasoning_tokens;
            entry.total_cost_usd += usage.total_cost_usd;
            entry.session_count += usage.session_count;
        }
    }
    let mut result: Vec<DailyUsage> = map.into_values().collect();
    result.sort_by(|a, b| a.date.cmp(&b.date));
    result
}

/// 合并多个 ModelUsage 列表（扁平拼接，按 token 总量降序排列）
fn merge_model_stats(lists: Vec<Vec<ModelUsage>>) -> Vec<ModelUsage> {
    let mut all: Vec<ModelUsage> = lists.into_iter().flatten().collect();
    all.sort_by(|a, b| {
        let ta = a.input_tokens + a.output_tokens;
        let tb = b.input_tokens + b.output_tokens;
        tb.cmp(&ta)
    });
    all
}

/// 收集五数据源的今日统计
fn collect_today_stats() -> [UsageStats; 5] {
    [
        crate::services::opencode::get_today_stats(),
        crate::services::ccswitch::get_today_stats(),
        crate::services::workbuddy::get_today_stats(),
        crate::services::hermes::get_today_stats(),
        crate::services::zcode::get_today_stats(),
    ]
}

/// 收集五数据源的周统计
fn collect_week_stats() -> [UsageStats; 5] {
    [
        crate::services::opencode::get_week_stats(),
        crate::services::ccswitch::get_week_stats(),
        crate::services::workbuddy::get_week_stats(),
        crate::services::hermes::get_week_stats(),
        crate::services::zcode::get_week_stats(),
    ]
}

/// 收集五数据源的月统计
fn collect_month_stats() -> [UsageStats; 5] {
    [
        crate::services::opencode::get_month_stats(),
        crate::services::ccswitch::get_month_stats(),
        crate::services::workbuddy::get_month_stats(),
        crate::services::hermes::get_month_stats(),
        crate::services::zcode::get_month_stats(),
    ]
}

/// 收集五数据源的全部统计
fn collect_all_time_stats() -> [UsageStats; 5] {
    [
        crate::services::opencode::get_all_time_stats(),
        crate::services::ccswitch::get_all_time_stats(),
        crate::services::workbuddy::get_all_time_stats(),
        crate::services::hermes::get_all_time_stats(),
        crate::services::zcode::get_all_time_stats(),
    ]
}

/// 获取今日用量统计
#[tauri::command]
pub async fn get_today_stats(_state: tauri::State<'_, AppState>) -> Result<UsageStats, String> {
    Ok(merge_stats(&collect_today_stats()))
}

/// 获取本周用量统计
#[tauri::command]
pub async fn get_week_stats(_state: tauri::State<'_, AppState>) -> Result<UsageStats, String> {
    Ok(merge_stats(&collect_week_stats()))
}

/// 获取本月用量统计
#[tauri::command]
pub async fn get_month_stats(_state: tauri::State<'_, AppState>) -> Result<UsageStats, String> {
    Ok(merge_stats(&collect_month_stats()))
}

/// 获取全部用量统计
#[tauri::command]
pub async fn get_all_time_stats(_state: tauri::State<'_, AppState>) -> Result<UsageStats, String> {
    Ok(merge_stats(&collect_all_time_stats()))
}

/// 获取每日用量（最近 N 天）
#[tauri::command]
pub async fn get_daily_usage(
    _state: tauri::State<'_, AppState>,
    days: i32,
) -> Result<Vec<DailyUsage>, String> {
    Ok(merge_daily_usage(vec![
        crate::services::opencode::get_daily_usage(days),
        crate::services::ccswitch::get_daily_usage(days),
        crate::services::workbuddy::get_daily_usage(days),
        crate::services::hermes::get_daily_usage(days),
        crate::services::zcode::get_daily_usage(days),
    ]))
}

/// 获取今日模型分布
#[tauri::command]
pub async fn get_today_model_stats(
    _state: tauri::State<'_, AppState>,
) -> Result<Vec<ModelUsage>, String> {
    Ok(merge_model_stats(vec![
        crate::services::opencode::get_today_model_stats(),
        crate::services::ccswitch::get_today_model_stats(),
        crate::services::workbuddy::get_today_model_stats(),
        crate::services::hermes::get_today_model_stats(),
        crate::services::zcode::get_today_model_stats(),
    ]))
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
    let today = merge_stats(&collect_today_stats());
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
    let today = merge_stats(&collect_today_stats());
    let mem = get_memory_usage().await?;

    let token_text = format_tokens(today.input_tokens + today.output_tokens);

    // 推送更新事件到悬浮条和主面板
    let _ = app.emit("usage-updated", serde_json::json!({
        "tokenText": token_text,
        "memPercent": mem.usage_percent,
        "costUsd": today.cost_usd,
        "inputTokens": today.input_tokens,
        "outputTokens": today.output_tokens,
        "cacheReadTokens": today.cache_read_tokens,
        "cacheWriteTokens": today.cache_write_tokens,
        "reasoningTokens": today.reasoning_tokens,
        "sessions": today.sessions,
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
