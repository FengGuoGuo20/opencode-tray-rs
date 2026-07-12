//! SQLite 通用工具模块
//!
//! 提供只读连接、时间戳转换等共享功能。

#![allow(dead_code)]

use chrono::{Local, Datelike, Duration, TimeZone};
use rusqlite::Connection;
use std::path::Path;

/// 以只读模式打开 SQLite 数据库
///
/// 使用 `PRAGMA query_only` 确保不会意外写入。
/// 启用 WAL 模式以支持并发读取。
pub fn open_read_only(path: &Path) -> Result<Connection, String> {
    let conn = Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| format!("打开数据库失败 {}: {e}", path.display()))?;

    conn.execute_batch("PRAGMA query_only = ON; PRAGMA journal_mode = WAL;")
        .map_err(|e| format!("设置 PRAGMA 失败: {e}"))?;

    Ok(conn)
}

/// 获取今日 00:00 的 Unix 时间戳（毫秒）
pub fn today_start_epoch_ms() -> i64 {
    let now = Local::now();
    let midnight = now.date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap_or_else(|| now.date_naive().and_hms_opt(0, 0, 0).unwrap());
    match Local.from_local_datetime(&midnight) {
        chrono::LocalResult::Single(dt) => dt.timestamp_millis(),
        chrono::LocalResult::Ambiguous(earliest, _) => earliest.timestamp_millis(),
        chrono::LocalResult::None => now.timestamp_millis(),
    }
}

/// 获取今日 00:00 的 Unix 时间戳（秒）
pub fn today_start_epoch_secs() -> i64 {
    today_start_epoch_ms() / 1000
}

/// 获取本周一 00:00 的 Unix 时间戳（毫秒）
pub fn week_start_epoch_ms() -> i64 {
    let now = Local::now();
    let weekday = now.weekday().num_days_from_monday() as i64;
    let monday = now.date_naive() - Duration::days(weekday);
    let midnight = monday.and_hms_opt(0, 0, 0).unwrap_or(monday.and_hms_opt(0, 0, 0).unwrap());
    match Local.from_local_datetime(&midnight) {
        chrono::LocalResult::Single(dt) => dt.timestamp_millis(),
        chrono::LocalResult::Ambiguous(earliest, _) => earliest.timestamp_millis(),
        chrono::LocalResult::None => now.timestamp_millis(),
    }
}

/// 获取本周一 00:00 的 Unix 时间戳（秒）
pub fn week_start_epoch_secs() -> i64 {
    week_start_epoch_ms() / 1000
}

/// 获取 N 天前 00:00 的 Unix 时间戳（毫秒）
pub fn days_ago_epoch_ms(days: i32) -> i64 {
    let now = Local::now();
    let target = now.date_naive() - Duration::days(days as i64);
    let midnight = target.and_hms_opt(0, 0, 0).unwrap_or(target.and_hms_opt(0, 0, 0).unwrap());
    match Local.from_local_datetime(&midnight) {
        chrono::LocalResult::Single(dt) => dt.timestamp_millis(),
        chrono::LocalResult::Ambiguous(earliest, _) => earliest.timestamp_millis(),
        chrono::LocalResult::None => now.timestamp_millis(),
    }
}

/// 获取 N 天前 00:00 的 Unix 时间戳（秒）
pub fn days_ago_epoch_secs(days: i32) -> i64 {
    days_ago_epoch_ms(days) / 1000
}

/// 获取本月 1 日 00:00 的 Unix 时间戳（毫秒）
pub fn month_start_epoch_ms() -> i64 {
    let now = Local::now();
    let first_of_month = now.date_naive()
        .with_day(1)
        .unwrap_or(now.date_naive())
        .and_hms_opt(0, 0, 0)
        .unwrap_or(now.date_naive().and_hms_opt(0, 0, 0).unwrap());
    match Local.from_local_datetime(&first_of_month) {
        chrono::LocalResult::Single(dt) => dt.timestamp_millis(),
        chrono::LocalResult::Ambiguous(earliest, _) => earliest.timestamp_millis(),
        chrono::LocalResult::None => now.timestamp_millis(),
    }
}

/// 获取本月 1 日 00:00 的 Unix 时间戳（秒）
pub fn month_start_epoch_secs() -> i64 {
    month_start_epoch_ms() / 1000
}
