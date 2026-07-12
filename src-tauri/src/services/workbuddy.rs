//! WorkBuddy 数据源服务
//!
//! 数据源：`~/.workbuddy/projects/{project}/{session}.jsonl`
//! 每行 JSON：`{ "timestamp": ms, "providerData": { "usage": {...}, "rawUsage": {...} } }`

#![allow(dead_code)]

use crate::commands::usage::{DailyUsage, ModelUsage, UsageStats};
use crate::db::helper;
use chrono::TimeZone;
use std::collections::HashMap;

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

/// 从 JSON 值中提取一条记录的 token 数据，若全部为 0 则返回 None
fn extract_record(json: &serde_json::Value) -> Option<(i64, i64, i64, i64, i64, f64)> {
    let mut input = 0i64;
    let mut output = 0i64;
    let mut cache_read = 0i64;
    let mut cache_write = 0i64;
    let mut reasoning = 0i64;
    let mut cost = 0.0f64;

    if let Some(usage) = json.pointer("/providerData/usage") {
        input += usage.get("inputTokens").and_then(|v| v.as_i64()).unwrap_or(0);
        output += usage.get("outputTokens").and_then(|v| v.as_i64()).unwrap_or(0);
    }

    if let Some(raw) = json.pointer("/providerData/rawUsage") {
        cache_read += raw.get("cached_tokens")
            .or_else(|| raw.get("prompt_cache_hit_tokens"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        cache_write += raw.get("prompt_cache_write_tokens")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        reasoning += raw.get("reasoning_tokens")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        cost += raw.get("credit").and_then(|v| v.as_f64()).unwrap_or(0.0);
    }

    // 过滤零 usage 记录
    if input + output + cache_read + cache_write + reasoning == 0 && cost == 0.0 {
        return None;
    }

    Some((input, output, cache_read, cache_write, reasoning, cost))
}

/// 获取今日统计
pub fn get_today_stats() -> UsageStats {
    get_stats_since(helper::today_start_epoch_ms() as u64)
}

/// 获取本周统计
pub fn get_week_stats() -> UsageStats {
    get_stats_since(helper::week_start_epoch_ms() as u64)
}

/// 获取本月统计
pub fn get_month_stats() -> UsageStats {
    get_stats_since(helper::month_start_epoch_ms() as u64)
}

/// 获取全部统计
pub fn get_all_time_stats() -> UsageStats {
    get_stats_since(0)
}

/// 通用：获取指定时间戳之后的统计
fn get_stats_since(epoch_start: u64) -> UsageStats {
    let data_dir = match get_data_dir() {
        Some(d) => d,
        _ => return UsageStats::default(),
    };

    let mut total = UsageStats::default();

    if let Ok(entries) = std::fs::read_dir(&data_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                collect_jsonl_stats(&entry.path(), epoch_start, &mut total);
            }
        }
    }

    total
}

/// 递归扫描 JSONL 文件并累加统计
fn collect_jsonl_stats(project_dir: &std::path::Path, epoch_start: u64, total: &mut UsageStats) {
    if let Ok(entries) = std::fs::read_dir(project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                parse_jsonl_file_stats(&path, epoch_start, total);
            } else if path.is_dir() {
                collect_jsonl_stats(&path, epoch_start, total);
            }
        }
    }
}

/// 解析单个 JSONL 文件，累加 UsageStats
fn parse_jsonl_file_stats(path: &std::path::Path, epoch_start: u64, total: &mut UsageStats) {
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                let timestamp = json.get("timestamp")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                if timestamp < epoch_start {
                    continue;
                }

                if let Some((input, output, cache_read, cache_write, reasoning, cost)) = extract_record(&json) {
                    total.input_tokens += input;
                    total.output_tokens += output;
                    total.cache_read_tokens += cache_read;
                    total.cache_write_tokens += cache_write;
                    total.reasoning_tokens += reasoning;
                    total.cost_usd += cost;
                    total.sessions += 1;
                }
            }
        }
    }
}

/// 获取每日用量
pub fn get_daily_usage(days: i32) -> Vec<DailyUsage> {
    let data_dir = match get_data_dir() {
        Some(d) => d,
        _ => return vec![],
    };

    let days_ago = helper::days_ago_epoch_ms(days) as u64;
    let mut map: HashMap<String, DailyUsage> = HashMap::new();

    if let Ok(entries) = std::fs::read_dir(&data_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                collect_jsonl_daily(&entry.path(), days_ago, &mut map);
            }
        }
    }

    let mut result: Vec<DailyUsage> = map.into_values().collect();
    result.sort_by(|a, b| a.date.cmp(&b.date));
    result
}

/// 递归扫描 JSONL 文件，按日期聚合
fn collect_jsonl_daily(project_dir: &std::path::Path, epoch_start: u64, map: &mut HashMap<String, DailyUsage>) {
    if let Ok(entries) = std::fs::read_dir(project_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
                parse_jsonl_daily(&path, epoch_start, map);
            } else if path.is_dir() {
                collect_jsonl_daily(&path, epoch_start, map);
            }
        }
    }
}

/// 解析单个 JSONL 文件，按日期聚合 DailyUsage
fn parse_jsonl_daily(path: &std::path::Path, epoch_start: u64, map: &mut HashMap<String, DailyUsage>) {
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                let timestamp = json.get("timestamp")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                if timestamp < epoch_start {
                    continue;
                }

                if let Some((input, output, cache_read, cache_write, reasoning, cost)) = extract_record(&json) {
                    // 将 epoch ms 转为日期字符串 YYYY-MM-DD
                    let date = chrono::Local
                        .timestamp_millis_opt(timestamp as i64)
                        .single()
                        .map(|dt| dt.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|| "unknown".to_string());

                    let entry = map.entry(date.clone()).or_insert_with(|| DailyUsage {
                        date,
                        ..Default::default()
                    });
                    entry.total_input_tokens += input;
                    entry.total_output_tokens += output;
                    entry.total_cache_read_tokens += cache_read;
                    entry.total_cache_write_tokens += cache_write;
                    entry.total_reasoning_tokens += reasoning;
                    entry.total_cost_usd += cost;
                    entry.session_count += 1;
                }
            }
        }
    }
}

/// 获取今日模型分布
pub fn get_today_model_stats() -> Vec<ModelUsage> {
    // WorkBuddy JSONL 中模型信息不统一，暂返回空
    vec![]
}
