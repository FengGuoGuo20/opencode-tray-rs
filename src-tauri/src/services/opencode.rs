//! OpenCode 数据源服务
//!
//! 数据库：`~/.local/share/opencode/opencode.db`
//! 表：`session`
//! 关键字段：`tokens_input, tokens_output, tokens_cache_read, tokens_cache_write, tokens_reasoning, cost, model(JSON), time_created(ms)`

#![allow(dead_code)]

use crate::commands::usage::{DailyUsage, ModelUsage, UsageStats};
use crate::db::helper;

/// 获取 OpenCode 数据库路径
pub fn get_db_path() -> Option<std::path::PathBuf> {
    let home = dirs::home_dir()?;
    Some(home.join(".local/share/opencode/opencode.db"))
}

/// 获取今日统计
pub fn get_today_stats() -> UsageStats {
    let db_path = match get_db_path() {
        Some(p) if p.exists() => p,
        _ => return UsageStats::default(),
    };

    let conn = match helper::open_read_only(&db_path) {
        Ok(c) => c,
        Err(e) => {
            log::warn!("OpenCode: 打开数据库失败: {e}");
            return UsageStats::default();
        }
    };

    let today_start = helper::today_start_epoch_ms();

    let mut stmt = match conn.prepare(
        "SELECT
            COALESCE(SUM(tokens_input), 0),
            COALESCE(SUM(tokens_output), 0),
            COALESCE(SUM(tokens_cache_read), 0),
            COALESCE(SUM(tokens_cache_write), 0),
            COALESCE(SUM(tokens_reasoning), 0),
            COALESCE(SUM(cost), 0),
            COUNT(*)
         FROM session
         WHERE time_created >= $1"
    ) {
        Ok(s) => s,
        Err(e) => {
            log::warn!("OpenCode: 查询失败: {e}");
            return UsageStats::default();
        }
    };

    let result = stmt.query_row(rusqlite::params![today_start], |row| {
        Ok(UsageStats {
            input_tokens: row.get(0)?,
            output_tokens: row.get(1)?,
            cache_read_tokens: row.get(2)?,
            cache_write_tokens: row.get(3)?,
            reasoning_tokens: row.get(4)?,
            cost_usd: row.get(5)?,
            sessions: row.get(6)?,
        })
    });

    result.unwrap_or_default()
}

/// 获取本周统计
pub fn get_week_stats() -> UsageStats {
    let db_path = match get_db_path() {
        Some(p) if p.exists() => p,
        _ => return UsageStats::default(),
    };

    let conn = match helper::open_read_only(&db_path) {
        Ok(c) => c,
        Err(_) => return UsageStats::default(),
    };

    let week_start = helper::week_start_epoch_ms();

    let mut stmt = match conn.prepare(
        "SELECT
            COALESCE(SUM(tokens_input), 0),
            COALESCE(SUM(tokens_output), 0),
            COALESCE(SUM(tokens_cache_read), 0),
            COALESCE(SUM(tokens_cache_write), 0),
            COALESCE(SUM(tokens_reasoning), 0),
            COALESCE(SUM(cost), 0),
            COUNT(*)
         FROM session
         WHERE time_created >= $1"
    ) {
        Ok(s) => s,
        Err(_) => return UsageStats::default(),
    };

    stmt.query_row(rusqlite::params![week_start], |row| {
        Ok(UsageStats {
            input_tokens: row.get(0)?,
            output_tokens: row.get(1)?,
            cache_read_tokens: row.get(2)?,
            cache_write_tokens: row.get(3)?,
            reasoning_tokens: row.get(4)?,
            cost_usd: row.get(5)?,
            sessions: row.get(6)?,
        })
    }).unwrap_or_default()
}

/// 获取每日用量（最近 N 天）
pub fn get_daily_usage(days: i32) -> Vec<DailyUsage> {
    let db_path = match get_db_path() {
        Some(p) if p.exists() => p,
        _ => return vec![],
    };

    let conn = match helper::open_read_only(&db_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let days_ago = helper::days_ago_epoch_ms(days);

    let mut stmt = match conn.prepare(
        "SELECT
            DATE(time_created / 1000, 'unixepoch', 'localtime') as day,
            COALESCE(SUM(tokens_input), 0),
            COALESCE(SUM(tokens_output), 0),
            COALESCE(SUM(cost), 0)
         FROM session
         WHERE time_created >= $1
         GROUP BY day
         ORDER BY day ASC"
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let rows = stmt.query_map(rusqlite::params![days_ago], |row| {
        Ok(DailyUsage {
            date: row.get(0)?,
            total_input_tokens: row.get(1)?,
            total_output_tokens: row.get(2)?,
            total_cost_usd: row.get(3)?,
        })
    });

    match rows {
        Ok(r) => r.filter_map(|v| v.ok()).collect(),
        Err(_) => vec![],
    }
}

/// 获取今日模型分布
pub fn get_today_model_stats() -> Vec<ModelUsage> {
    let db_path = match get_db_path() {
        Some(p) if p.exists() => p,
        _ => return vec![],
    };

    let conn = match helper::open_read_only(&db_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let today_start = helper::today_start_epoch_ms();

    let mut stmt = match conn.prepare(
        "SELECT
            COALESCE(json_extract(model, '$.id'), model) as model_name,
            COALESCE(SUM(tokens_input), 0),
            COALESCE(SUM(tokens_output), 0),
            COALESCE(SUM(cost), 0),
            COUNT(*)
         FROM session
         WHERE time_created >= $1
         GROUP BY model_name
         ORDER BY COALESCE(SUM(tokens_input), 0) + COALESCE(SUM(tokens_output), 0) DESC"
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let rows = stmt.query_map(rusqlite::params![today_start], |row| {
        Ok(ModelUsage {
            model: row.get(0)?,
            input_tokens: row.get(1)?,
            output_tokens: row.get(2)?,
            cost_usd: row.get(3)?,
            sessions: row.get(4)?,
        })
    });

    match rows {
        Ok(r) => r.filter_map(|v| v.ok()).collect(),
        Err(_) => vec![],
    }
}
