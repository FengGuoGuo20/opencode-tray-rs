//! ZCode 数据源服务
//!
//! 数据库：`~/.zcode/cli/db/db.sqlite`
//! 表：`model_usage`
//! 关键字段：`input_tokens, output_tokens, cache_read_input_tokens, cache_creation_input_tokens, reasoning_tokens, model_id, provider_id, started_at(毫秒)`

#![allow(dead_code)]
//! 无 cost 字段（ZCode 费用始终为 0）

use crate::commands::usage::{DailyUsage, ModelUsage, SourceReport, UsageStats};
use crate::db::helper;

const SOURCE_ID: &str = "zcode";
const SOURCE_NAME: &str = "ZCode";
const DETAIL: &str = "model_usage 表（无费用）";

/// 解析后的数据库路径（含 override 展开），不检查存在性
fn resolved_path() -> std::path::PathBuf {
    let settings = crate::services::current_settings();
    crate::services::paths::zcode_db(&settings)
}

/// 获取 ZCode 数据库路径（支持设置覆盖）
pub fn get_db_path() -> Option<std::path::PathBuf> {
    crate::services::paths::existing(resolved_path())
}

/// 今日统计：在已有连接上执行查询（毫秒级时间戳）
fn query_today(conn: &rusqlite::Connection) -> Result<UsageStats, rusqlite::Error> {
    let today_start = helper::today_start_epoch_ms();
    let mut stmt = conn.prepare(
        "SELECT
            COALESCE(SUM(input_tokens), 0),
            COALESCE(SUM(output_tokens), 0),
            COALESCE(SUM(cache_read_input_tokens), 0),
            COALESCE(SUM(cache_creation_input_tokens), 0),
            COALESCE(SUM(reasoning_tokens), 0),
            COUNT(*)
         FROM model_usage
         WHERE started_at >= $1",
    )?;
    let stats = stmt.query_row(rusqlite::params![today_start], |row| {
        Ok(UsageStats {
            input_tokens: row.get(0)?,
            output_tokens: row.get(1)?,
            cache_read_tokens: row.get(2)?,
            cache_write_tokens: row.get(3)?,
            reasoning_tokens: row.get(4)?,
            cost_usd: 0.0,
            sessions: row.get(5)?,
        })
    })?;
    Ok(stats)
}

/// 今日诊断报告（含状态/路径/错误），get_today_stats 委托给它保证口径一致
pub fn today_report() -> SourceReport {
    let path = resolved_path();
    let path_str = path.display().to_string();
    if !path.exists() {
        return SourceReport::not_found(SOURCE_ID, SOURCE_NAME, path_str);
    }
    let conn = match helper::open_read_only(&path) {
        Ok(c) => c,
        Err(e) => return SourceReport::error(SOURCE_ID, SOURCE_NAME, path_str, e),
    };
    match query_today(&conn) {
        Ok(stats) => SourceReport::ok(SOURCE_ID, SOURCE_NAME, path_str, DETAIL, stats),
        Err(e) => SourceReport::error(SOURCE_ID, SOURCE_NAME, path_str, e.to_string()),
    }
}

/// 获取今日统计
pub fn get_today_stats() -> UsageStats {
    today_report().stats
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
            COALESCE(SUM(input_tokens), 0),
            COALESCE(SUM(output_tokens), 0),
            COALESCE(SUM(cache_read_input_tokens), 0),
            COALESCE(SUM(cache_creation_input_tokens), 0),
            COALESCE(SUM(reasoning_tokens), 0),
            COUNT(*)
         FROM model_usage
         WHERE started_at >= $1"
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
            cost_usd: 0.0,
            sessions: row.get(5)?,
        })
    }).unwrap_or_default()
}

/// 获取每日用量
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
            DATE(started_at / 1000, 'unixepoch', 'localtime') as day,
            COALESCE(SUM(input_tokens), 0),
            COALESCE(SUM(output_tokens), 0),
            COALESCE(SUM(cache_read_input_tokens), 0),
            COALESCE(SUM(cache_creation_input_tokens), 0),
            COALESCE(SUM(reasoning_tokens), 0),
            0 as cost,
            COUNT(*)
         FROM model_usage
         WHERE started_at >= $1
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
            total_cache_read_tokens: row.get(3)?,
            total_cache_write_tokens: row.get(4)?,
            total_reasoning_tokens: row.get(5)?,
            total_cost_usd: row.get(6)?,
            session_count: row.get(7)?,
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
            model_id,
            COALESCE(SUM(input_tokens), 0),
            COALESCE(SUM(output_tokens), 0),
            COALESCE(SUM(cache_read_input_tokens), 0),
            COALESCE(SUM(cache_creation_input_tokens), 0),
            COALESCE(SUM(reasoning_tokens), 0),
            0 as cost,
            COUNT(*)
         FROM model_usage
         WHERE started_at >= $1
         GROUP BY model_id
         ORDER BY COALESCE(SUM(input_tokens), 0) + COALESCE(SUM(output_tokens), 0) DESC"
    ) {
        Ok(s) => s,
        Err(_) => return vec![],
    };

    let rows = stmt.query_map(rusqlite::params![today_start], |row| {
        Ok(ModelUsage {
            model: row.get(0)?,
            input_tokens: row.get(1)?,
            output_tokens: row.get(2)?,
            cache_read_tokens: row.get(3)?,
            cache_write_tokens: row.get(4)?,
            reasoning_tokens: row.get(5)?,
            cost_usd: row.get(6)?,
            sessions: row.get(7)?,
        })
    });

    match rows {
        Ok(r) => r.filter_map(|v| v.ok()).collect(),
        Err(_) => vec![],
    }
}

/// 获取本月统计
pub fn get_month_stats() -> UsageStats {
    let db_path = match get_db_path() {
        Some(p) if p.exists() => p,
        _ => return UsageStats::default(),
    };

    let conn = match helper::open_read_only(&db_path) {
        Ok(c) => c,
        Err(_) => return UsageStats::default(),
    };

    let month_start = helper::month_start_epoch_ms();

    let mut stmt = match conn.prepare(
        "SELECT
            COALESCE(SUM(input_tokens), 0),
            COALESCE(SUM(output_tokens), 0),
            COALESCE(SUM(cache_read_input_tokens), 0),
            COALESCE(SUM(cache_creation_input_tokens), 0),
            COALESCE(SUM(reasoning_tokens), 0),
            COUNT(*)
         FROM model_usage
         WHERE started_at >= $1"
    ) {
        Ok(s) => s,
        Err(_) => return UsageStats::default(),
    };

    stmt.query_row(rusqlite::params![month_start], |row| {
        Ok(UsageStats {
            input_tokens: row.get(0)?,
            output_tokens: row.get(1)?,
            cache_read_tokens: row.get(2)?,
            cache_write_tokens: row.get(3)?,
            reasoning_tokens: row.get(4)?,
            cost_usd: 0.0,
            sessions: row.get(5)?,
        })
    }).unwrap_or_default()
}

/// 获取全部统计
pub fn get_all_time_stats() -> UsageStats {
    let db_path = match get_db_path() {
        Some(p) if p.exists() => p,
        _ => return UsageStats::default(),
    };

    let conn = match helper::open_read_only(&db_path) {
        Ok(c) => c,
        Err(_) => return UsageStats::default(),
    };

    let mut stmt = match conn.prepare(
        "SELECT
            COALESCE(SUM(input_tokens), 0),
            COALESCE(SUM(output_tokens), 0),
            COALESCE(SUM(cache_read_input_tokens), 0),
            COALESCE(SUM(cache_creation_input_tokens), 0),
            COALESCE(SUM(reasoning_tokens), 0),
            COUNT(*)
         FROM model_usage
         WHERE started_at >= 0"
    ) {
        Ok(s) => s,
        Err(_) => return UsageStats::default(),
    };

    stmt.query_row([], |row| {
        Ok(UsageStats {
            input_tokens: row.get(0)?,
            output_tokens: row.get(1)?,
            cache_read_tokens: row.get(2)?,
            cache_write_tokens: row.get(3)?,
            reasoning_tokens: row.get(4)?,
            cost_usd: 0.0,
            sessions: row.get(5)?,
        })
    }).unwrap_or_default()
}
