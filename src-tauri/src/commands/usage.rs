//! 用量查询命令模块
//!
//! 提供 Tauri 命令供前端查询各数据源的 token 用量、内存占用等。

use tauri::Emitter;
use tauri::Manager;

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
    pub display_mode: String,
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

/// 数据源状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceStatus {
    /// 已连接（路径存在且查询成功）
    Ok,
    /// 未找到（路径不存在）
    NotFound,
    /// 读取失败（路径存在但查询/解析出错）
    Error,
}

impl SourceStatus {
    pub fn as_text(self) -> &'static str {
        match self {
            SourceStatus::Ok => "已连接",
            SourceStatus::NotFound => "未找到",
            SourceStatus::Error => "读取失败",
        }
    }

    /// 前端用短标签
    pub fn as_code(self) -> &'static str {
        match self {
            SourceStatus::Ok => "ok",
            SourceStatus::NotFound => "not_found",
            SourceStatus::Error => "error",
        }
    }
}

/// 单个数据源的诊断报告（今日贡献 + 健康状态）
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceReport {
    pub source_id: String,
    pub source_name: String,
    /// 解析后的数据源路径（含 override 展开），供诊断
    pub path: String,
    pub path_exists: bool,
    pub status: String,
    /// 状态文本（"已连接" / "未找到" / "读取失败"）
    pub status_text: String,
    /// 失败原因 / 表名等补充信息
    pub detail_text: String,
    pub stats: UsageStats,
    pub total_tokens: i64,
}

impl SourceReport {
    pub fn not_found(id: &str, name: &str, path: String) -> Self {
        Self {
            source_id: id.to_string(),
            source_name: name.to_string(),
            path_exists: false,
            status: SourceStatus::NotFound.as_code().to_string(),
            status_text: SourceStatus::NotFound.as_text().to_string(),
            detail_text: "数据源文件/目录不存在".to_string(),
            stats: UsageStats::default(),
            total_tokens: 0,
            path,
        }
    }

    pub fn ok(id: &str, name: &str, path: String, detail: &str, stats: UsageStats) -> Self {
        let total_tokens = total_tokens(&stats);
        Self {
            source_id: id.to_string(),
            source_name: name.to_string(),
            path_exists: true,
            status: SourceStatus::Ok.as_code().to_string(),
            status_text: SourceStatus::Ok.as_text().to_string(),
            detail_text: detail.to_string(),
            stats,
            total_tokens,
            path,
        }
    }

    pub fn error(id: &str, name: &str, path: String, detail: String) -> Self {
        log::warn!("{name}: 读取失败: {detail}");
        Self {
            source_id: id.to_string(),
            source_name: name.to_string(),
            path_exists: true,
            status: SourceStatus::Error.as_code().to_string(),
            status_text: SourceStatus::Error.as_text().to_string(),
            detail_text: detail,
            stats: UsageStats::default(),
            total_tokens: 0,
            path,
        }
    }
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

/// 合并多个 ModelUsage 列表（同名模型聚合，按 token 总量降序）
fn merge_model_stats(lists: Vec<Vec<ModelUsage>>) -> Vec<ModelUsage> {
    let mut map: std::collections::HashMap<String, ModelUsage> = std::collections::HashMap::new();
    for list in lists {
        for m in list {
            let key = if m.model.trim().is_empty() {
                "unknown".to_string()
            } else {
                m.model.clone()
            };
            let entry = map.entry(key.clone()).or_insert_with(|| ModelUsage {
                model: key,
                ..Default::default()
            });
            entry.input_tokens += m.input_tokens;
            entry.output_tokens += m.output_tokens;
            entry.cache_read_tokens += m.cache_read_tokens;
            entry.cache_write_tokens += m.cache_write_tokens;
            entry.reasoning_tokens += m.reasoning_tokens;
            entry.cost_usd += m.cost_usd;
            entry.sessions += m.sessions;
        }
    }
    let mut all: Vec<ModelUsage> = map.into_values().collect();
    all.sort_by(|a, b| {
        let ta = a.input_tokens
            + a.output_tokens
            + a.cache_read_tokens
            + a.cache_write_tokens
            + a.reasoning_tokens;
        let tb = b.input_tokens
            + b.output_tokens
            + b.cache_read_tokens
            + b.cache_write_tokens
            + b.reasoning_tokens;
        tb.cmp(&ta)
    });
    all
}

/// 在阻塞线程中收集五数据源今日统计
fn collect_today_stats_blocking() -> UsageStats {
    merge_stats(&[
        crate::services::opencode::get_today_stats(),
        crate::services::ccswitch::get_today_stats(),
        crate::services::workbuddy::get_today_stats(),
        crate::services::hermes::get_today_stats(),
        crate::services::zcode::get_today_stats(),
    ])
}

fn collect_week_stats_blocking() -> UsageStats {
    merge_stats(&[
        crate::services::opencode::get_week_stats(),
        crate::services::ccswitch::get_week_stats(),
        crate::services::workbuddy::get_week_stats(),
        crate::services::hermes::get_week_stats(),
        crate::services::zcode::get_week_stats(),
    ])
}

fn collect_month_stats_blocking() -> UsageStats {
    merge_stats(&[
        crate::services::opencode::get_month_stats(),
        crate::services::ccswitch::get_month_stats(),
        crate::services::workbuddy::get_month_stats(),
        crate::services::hermes::get_month_stats(),
        crate::services::zcode::get_month_stats(),
    ])
}

fn collect_all_time_stats_blocking() -> UsageStats {
    merge_stats(&[
        crate::services::opencode::get_all_time_stats(),
        crate::services::ccswitch::get_all_time_stats(),
        crate::services::workbuddy::get_all_time_stats(),
        crate::services::hermes::get_all_time_stats(),
        crate::services::zcode::get_all_time_stats(),
    ])
}

fn collect_daily_usage_blocking(days: i32) -> Vec<DailyUsage> {
    merge_daily_usage(vec![
        crate::services::opencode::get_daily_usage(days),
        crate::services::ccswitch::get_daily_usage(days),
        crate::services::workbuddy::get_daily_usage(days),
        crate::services::hermes::get_daily_usage(days),
        crate::services::zcode::get_daily_usage(days),
    ])
}

fn collect_model_stats_blocking() -> Vec<ModelUsage> {
    merge_model_stats(vec![
        crate::services::opencode::get_today_model_stats(),
        crate::services::ccswitch::get_today_model_stats(),
        crate::services::workbuddy::get_today_model_stats(),
        crate::services::hermes::get_today_model_stats(),
        crate::services::zcode::get_today_model_stats(),
    ])
}

/// 读取今日统计：优先走缓存，缓存未填充时单次采集并回填
///
/// 定时器每轮会刷新缓存，悬浮条/tray/面板"今日"卡都走这里，
/// 避免每个窗口各自重扫五数据源。
async fn cached_today(app: &tauri::AppHandle) -> Result<UsageStats, String> {
    if let Some(state) = app.try_state::<AppState>() {
        // 快速路径：缓存命中直接返回
        if let Some(stats) = state.today_stats.read().await.clone() {
            return Ok(stats);
        }
    }
    // 缓存未填充（首启/定时器尚未跑）：单次采集并回填
    let today = tokio::task::spawn_blocking(collect_today_stats_blocking)
        .await
        .map_err(|e| format!("查询今日统计失败: {e}"))?;
    if let Some(state) = app.try_state::<AppState>() {
        let mut guard = state.today_stats.write().await;
        // 仅在仍未被定时器填充时写入，避免覆盖更新鲜的数据
        if guard.is_none() {
            *guard = Some(today.clone());
        }
    }
    Ok(today)
}

/// 获取今日用量统计（读缓存，不每次扫库）
#[tauri::command]
pub async fn get_today_stats(app: tauri::AppHandle) -> Result<UsageStats, String> {
    cached_today(&app).await
}

/// 获取本周用量统计
#[tauri::command]
pub async fn get_week_stats(_state: tauri::State<'_, AppState>) -> Result<UsageStats, String> {
    tokio::task::spawn_blocking(collect_week_stats_blocking)
        .await
        .map_err(|e| format!("查询本周统计失败: {e}"))
}

/// 获取本月用量统计
#[tauri::command]
pub async fn get_month_stats(_state: tauri::State<'_, AppState>) -> Result<UsageStats, String> {
    tokio::task::spawn_blocking(collect_month_stats_blocking)
        .await
        .map_err(|e| format!("查询本月统计失败: {e}"))
}

/// 获取全部用量统计
#[tauri::command]
pub async fn get_all_time_stats(_state: tauri::State<'_, AppState>) -> Result<UsageStats, String> {
    tokio::task::spawn_blocking(collect_all_time_stats_blocking)
        .await
        .map_err(|e| format!("查询全部统计失败: {e}"))
}

/// 获取每日用量（最近 N 天）
#[tauri::command]
pub async fn get_daily_usage(
    _state: tauri::State<'_, AppState>,
    days: i32,
) -> Result<Vec<DailyUsage>, String> {
    tokio::task::spawn_blocking(move || collect_daily_usage_blocking(days))
        .await
        .map_err(|e| format!("查询每日用量失败: {e}"))
}

/// 获取今日模型分布
#[tauri::command]
pub async fn get_today_model_stats(
    _state: tauri::State<'_, AppState>,
) -> Result<Vec<ModelUsage>, String> {
    tokio::task::spawn_blocking(collect_model_stats_blocking)
        .await
        .map_err(|e| format!("查询模型分布失败: {e}"))
}

/// 在阻塞线程中收集五数据源今日诊断报告
fn collect_source_stats_blocking() -> Vec<SourceReport> {
    let mut all = vec![
        crate::services::opencode::today_report(),
        crate::services::ccswitch::today_report(),
        crate::services::workbuddy::today_report(),
        crate::services::hermes::today_report(),
        crate::services::zcode::today_report(),
    ];
    all.sort_by(|a, b| b.total_tokens.cmp(&a.total_tokens));
    all
}

/// 获取五数据源今日贡献 + 健康状态（诊断用）
#[tauri::command]
pub async fn get_today_source_stats() -> Result<Vec<SourceReport>, String> {
    tokio::task::spawn_blocking(collect_source_stats_blocking)
        .await
        .map_err(|e| format!("查询数据源状态失败: {e}"))
}

/// 获取内存使用信息
#[tauri::command]
pub async fn get_memory_usage() -> Result<MemoryInfo, String> {
    tokio::task::spawn_blocking(|| {
        let mut sys = sysinfo::System::new();
        sys.refresh_memory();

        let total = sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let used = sys.used_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let available = sys.available_memory() as f64 / 1024.0 / 1024.0 / 1024.0;
        let usage_percent = if total > 0.0 {
            used / total * 100.0
        } else {
            0.0
        };

        MemoryInfo {
            usage_percent,
            total_gb: total,
            used_gb: used,
            available_gb: available,
        }
    })
    .await
    .map_err(|e| format!("读取内存失败: {e}"))
}

/// 计算总 token 数（与 WPF 一致：input + output + cache_read + cache_write + reasoning）
fn total_tokens(s: &UsageStats) -> i64 {
    s.input_tokens
        + s.output_tokens
        + s.cache_read_tokens
        + s.cache_write_tokens
        + s.reasoning_tokens
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

/// 规范化悬浮条显示模式（费用相关模式已隐藏，统一回退）
fn normalize_display_mode(mode: &str) -> String {
    match mode {
        "token_only" | "mem_only" | "token_mem" => mode.to_string(),
        _ => "token_mem".to_string(),
    }
}

/// 获取悬浮条显示数据（今日数据走缓存，避免与定时器重复扫库）
#[tauri::command]
pub async fn get_tray_display(app: tauri::AppHandle) -> Result<TrayDisplay, String> {
    let settings = crate::services::current_settings();
    let today = cached_today(&app).await?;
    let mem = get_memory_usage().await?;

    Ok(TrayDisplay {
        token_text: format_tokens(total_tokens(&today)),
        mem_percent: mem.usage_percent,
        cost_text: String::new(),
        display_mode: normalize_display_mode(&settings.tray_display_mode),
    })
}

/// 更新托盘 tooltip
fn update_tray_tooltip(app: &tauri::AppHandle, token_text: &str, mem_percent: f64) {
    if let Some(tray) = app.tray_by_id(crate::tray::TRAY_ID) {
        let tip = format!(
            "OpenCodeTray\n今日 Token: {}\n内存: {:.0}%",
            token_text, mem_percent
        );
        let _ = tray.set_tooltip(Some(tip));
    }
}

/// 刷新用量数据并推送到前端
pub async fn refresh_usage_data(app: &tauri::AppHandle) -> Result<(), String> {
    let settings = crate::services::current_settings();
    let today = tokio::task::spawn_blocking(collect_today_stats_blocking)
        .await
        .map_err(|e| format!("刷新统计失败: {e}"))?;
    let mem = get_memory_usage().await?;

    let token_text = format_tokens(total_tokens(&today));

    // 缓存今日数据
    if let Some(state) = app.try_state::<AppState>() {
        let mut guard = state.today_stats.write().await;
        *guard = Some(today.clone());
    }

    update_tray_tooltip(app, &token_text, mem.usage_percent);

    // 推送更新事件到悬浮条和主面板
    // payload 已携带今日完整字段，前端可直接更新"今日"卡，无需回查
    let _ = app.emit(
        "usage-updated",
        serde_json::json!({
            "tokenText": token_text,
            "memPercent": mem.usage_percent,
            "displayMode": normalize_display_mode(&settings.tray_display_mode),
            "inputTokens": today.input_tokens,
            "outputTokens": today.output_tokens,
            "cacheReadTokens": today.cache_read_tokens,
            "cacheWriteTokens": today.cache_write_tokens,
            "reasoningTokens": today.reasoning_tokens,
            "sessions": today.sessions,
            "costUsd": today.cost_usd,
        }),
    );

    Ok(())
}

/// 启动定时刷新计时器（支持设置热更新刷新间隔）
pub async fn start_refresh_timer(app: tauri::AppHandle) {
    let mut interval_rx = if let Some(state) = app.try_state::<AppState>() {
        state.refresh_interval_rx.clone()
    } else {
        let (_tx, rx) = tokio::sync::watch::channel(30u64);
        rx
    };

    let mut current_secs = (*interval_rx.borrow()).clamp(5, 600);
    let mut usage_interval =
        tokio::time::interval(std::time::Duration::from_secs(current_secs));
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
            Ok(()) = interval_rx.changed() => {
                let next = (*interval_rx.borrow()).clamp(5, 600);
                if next != current_secs {
                    current_secs = next;
                    usage_interval = tokio::time::interval(std::time::Duration::from_secs(current_secs));
                    // 立即 tick 一次避免长时间等待
                    usage_interval.reset();
                    log::info!("刷新间隔已更新为 {current_secs}s");
                }
            }
        }
    }
}
