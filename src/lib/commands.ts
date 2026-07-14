//! Tauri 命令类型封装
//!
//! 提供类型安全的 invoke() 调用，与 Rust 后端 commands/usage.rs 对应。

import { invoke } from "@tauri-apps/api/core";

// ===== 类型定义 =====

export interface UsageStats {
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
  reasoningTokens: number;
  costUsd: number;
  sessions: number;
}

export interface DailyUsage {
  date: string;
  totalInputTokens: number;
  totalOutputTokens: number;
  totalCacheReadTokens: number;
  totalCacheWriteTokens: number;
  totalReasoningTokens: number;
  totalCostUsd: number;
  sessionCount: number;
}

export interface ModelUsage {
  model: string;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
  reasoningTokens: number;
  costUsd: number;
  sessions: number;
}

/**
 * 单个数据源的诊断报告（今日贡献 + 健康状态）。
 */
export interface SourceReport {
  sourceId: string;
  sourceName: string;
  /** 解析后的数据源路径，供诊断 */
  path: string;
  pathExists: boolean;
  /** 状态短码：ok / not_found / error */
  status: string;
  /** 状态文本：已连接 / 未找到 / 读取失败 */
  statusText: string;
  /** 失败原因 / 表名等补充信息 */
  detailText: string;
  stats: UsageStats;
  totalTokens: number;
}

export interface MemoryInfo {
  usagePercent: number;
  totalGb: number;
  usedGb: number;
  availableGb: number;
}

export interface TrayDisplay {
  tokenText: string;
  memPercent: number;
  costText: string;
  displayMode: string;
}

/**
 * usage-updated 事件 payload。
 * 携带今日完整字段，前端可直接更新"今日"卡，无需回查后端。
 */
export interface UsageUpdatedPayload {
  tokenText: string;
  memPercent: number;
  displayMode: string;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
  reasoningTokens: number;
  sessions: number;
  costUsd: number;
}

/**
 * memory-updated 事件 payload。
 */
export interface MemoryUpdatedPayload {
  memPercent: number;
}

export interface AppSettings {
  refreshIntervalSecs: number;
  trayDisplayMode: string;
  usdToCnyRate: number;
  /** 开机自启 */
  startWithWindows: boolean;
  floatingBarLeft?: number | null;
  floatingBarTop?: number | null;
  opencodeDbPath: string | null;
  ccswitchDbPath: string | null;
  workbuddyDirPath: string | null;
  hermesDbPath: string | null;
  zcodeDbPath: string | null;
}

// ===== 命令调用 =====

export async function getTodayStats(): Promise<UsageStats> {
  return invoke<UsageStats>("get_today_stats");
}

export async function getWeekStats(): Promise<UsageStats> {
  return invoke<UsageStats>("get_week_stats");
}

export async function getMonthStats(): Promise<UsageStats> {
  return invoke<UsageStats>("get_month_stats");
}

export async function getAllTimeStats(): Promise<UsageStats> {
  return invoke<UsageStats>("get_all_time_stats");
}

export async function getDailyUsage(days: number): Promise<DailyUsage[]> {
  return invoke<DailyUsage[]>("get_daily_usage", { days });
}

export async function getTodayModelStats(): Promise<ModelUsage[]> {
  return invoke<ModelUsage[]>("get_today_model_stats");
}

export async function getTodaySourceStats(): Promise<SourceReport[]> {
  return invoke<SourceReport[]>("get_today_source_stats");
}

export async function getMemoryUsage(): Promise<MemoryInfo> {
  return invoke<MemoryInfo>("get_memory_usage");
}

export async function getTrayDisplay(): Promise<TrayDisplay> {
  return invoke<TrayDisplay>("get_tray_display");
}

export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke("save_settings", { settings });
}

export async function showPanel(): Promise<void> {
  return invoke("show_panel");
}

export async function showSettings(): Promise<void> {
  return invoke("show_settings");
}

export async function toggleFloatingBar(): Promise<void> {
  return invoke("toggle_floating_bar");
}

export async function setFloatingBarPosition(x: number, y: number): Promise<void> {
  return invoke("set_floating_bar_position", { x, y });
}
