//! WorkBuddy 数据源服务
//!
//! 数据源：`~/.workbuddy/projects/{project}/{session}.jsonl`
//! 每行 JSON：`{ "timestamp": ms, "providerData": { "usage": {...}, "rawUsage": {...} } }`

#![allow(dead_code)]

use crate::commands::usage::{DailyUsage, ModelUsage, UsageStats};
use crate::db::helper;

/// 获取 WorkBuddy 项目目录
pub fn get_data_dir() -> Option<std::path::PathBuf> {
    let home = dirs::home_dir()?;
    let dir = home.join(".workbuddy/projects");
    if dir.exists() {
        Some(dir)
    } else {
        None
    }
}

/// 获取今日统计
pub fn get_today_stats() -> UsageStats {
    let data_dir = match get_data_dir() {
        Some(d) => d,
        _ => return UsageStats::default(),
    };

    let today_start = helper::today_start_epoch_ms() as u64;
    let mut total = UsageStats::default();

    if let Ok(entries) = std::fs::read_dir(&data_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                collect_jsonl_stats(&entry.path(), today_start, &mut total);
            }
        }
    }

    total
}

/// 递归扫描 JSONL 文件并累加统计
fn collect_jsonl_stats(project_dir: &std::path::Path, today_start: u64, total: &mut UsageStats) {
    if let Ok(entries) = std::fs::read_dir(project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                parse_jsonl_file(&path, today_start, total);
            } else if path.is_dir() {
                collect_jsonl_stats(&path, today_start, total);
            }
        }
    }
}

/// 解析单个 JSONL 文件
fn parse_jsonl_file(path: &std::path::Path, today_start: u64, total: &mut UsageStats) {
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                let timestamp = json.get("timestamp")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                if timestamp < today_start {
                    continue;
                }

                // providerData.usage
                if let Some(usage) = json.pointer("/providerData/usage") {
                    total.input_tokens += usage.get("inputTokens")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    total.output_tokens += usage.get("outputTokens")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                }

                // providerData.rawUsage
                if let Some(raw) = json.pointer("/providerData/rawUsage") {
                    total.cache_read_tokens += raw.get("cached_tokens")
                        .or_else(|| raw.get("prompt_cache_hit_tokens"))
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    total.cache_write_tokens += raw.get("prompt_cache_write_tokens")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    total.reasoning_tokens += raw.get("reasoning_tokens")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);
                    total.cost_usd += raw.get("credit")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(0.0);
                }

                total.sessions += 1;
            }
        }
    }
}

/// 获取本周统计
pub fn get_week_stats() -> UsageStats {
    // TODO: 实现周统计
    UsageStats::default()
}

/// 获取本月统计
pub fn get_month_stats() -> UsageStats {
    // TODO: 实现月统计
    UsageStats::default()
}

/// 获取全部统计
pub fn get_all_time_stats() -> UsageStats {
    // TODO: 实现全部统计
    UsageStats::default()
}

/// 获取每日用量
pub fn get_daily_usage(_days: i32) -> Vec<DailyUsage> {
    // TODO: 实现每日用量
    vec![]
}

/// 获取今日模型分布
pub fn get_today_model_stats() -> Vec<ModelUsage> {
    // TODO: WorkBuddy JSONL 中模型信息不统一，暂返回空
    vec![]
}
