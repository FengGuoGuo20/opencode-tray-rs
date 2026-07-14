//! 数据源路径解析
//!
//! 优先使用设置中的自定义路径，否则回退到各数据源默认路径。
//! 支持 `~` 与环境变量展开。

use std::path::{Path, PathBuf};

use crate::commands::settings::AppSettings;

/// 展开路径中的 `~` 与环境变量
pub fn expand_path(path: &str) -> PathBuf {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return PathBuf::new();
    }

    let expanded = if trimmed == "~" {
        dirs::home_dir().unwrap_or_default()
    } else if let Some(rest) = trimmed.strip_prefix("~/").or_else(|| trimmed.strip_prefix("~\\")) {
        dirs::home_dir()
            .unwrap_or_default()
            .join(rest)
    } else {
        PathBuf::from(trimmed)
    };

    // 展开环境变量 %VAR% / $VAR
    let s = expanded.to_string_lossy();
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            let mut name = String::new();
            while let Some(&n) = chars.peek() {
                if n == '%' {
                    chars.next();
                    break;
                }
                name.push(n);
                chars.next();
            }
            if let Ok(val) = std::env::var(&name) {
                result.push_str(&val);
            } else {
                result.push('%');
                result.push_str(&name);
                result.push('%');
            }
        } else if c == '$' {
            let mut name = String::new();
            while let Some(&n) = chars.peek() {
                if n.is_alphanumeric() || n == '_' {
                    name.push(n);
                    chars.next();
                } else {
                    break;
                }
            }
            if !name.is_empty() {
                if let Ok(val) = std::env::var(&name) {
                    result.push_str(&val);
                } else {
                    result.push('$');
                    result.push_str(&name);
                }
            } else {
                result.push('$');
            }
        } else {
            result.push(c);
        }
    }
    PathBuf::from(result)
}

/// 若 override 非空且展开后非空，则使用；否则 default
pub fn resolve_path(override_path: &Option<String>, default: PathBuf) -> PathBuf {
    if let Some(p) = override_path {
        let expanded = expand_path(p);
        if !expanded.as_os_str().is_empty() {
            return expanded;
        }
    }
    default
}

/// 从 settings 解析各数据源路径
pub fn opencode_db(settings: &AppSettings) -> PathBuf {
    let default = dirs::home_dir()
        .unwrap_or_default()
        .join(".local/share/opencode/opencode.db");
    resolve_path(&settings.opencode_db_path, default)
}

pub fn ccswitch_db(settings: &AppSettings) -> PathBuf {
    let default = dirs::home_dir()
        .unwrap_or_default()
        .join(".cc-switch/cc-switch.db");
    resolve_path(&settings.ccswitch_db_path, default)
}

pub fn workbuddy_dir(settings: &AppSettings) -> PathBuf {
    let default = dirs::home_dir()
        .unwrap_or_default()
        .join(".workbuddy/projects");
    resolve_path(&settings.workbuddy_dir_path, default)
}

pub fn hermes_db(settings: &AppSettings) -> PathBuf {
    let default = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::data_local_dir().unwrap_or_default())
        .join("Hermes/state.db");
    resolve_path(&settings.hermes_db_path, default)
}

pub fn zcode_db(settings: &AppSettings) -> PathBuf {
    let default = dirs::home_dir()
        .unwrap_or_default()
        .join(".zcode/cli/db/db.sqlite");
    resolve_path(&settings.zcode_db_path, default)
}

/// 仅当路径存在时返回 Some
pub fn existing(path: PathBuf) -> Option<PathBuf> {
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

/// 判断路径是否像文件/目录（仅用于日志）
#[allow(dead_code)]
pub fn path_label(path: &Path) -> String {
    path.display().to_string()
}
