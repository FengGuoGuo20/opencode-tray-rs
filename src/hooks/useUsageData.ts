//! 用量数据 Hook
//!
//! 刷新策略（轻量）：
//! - 挂载时全量拉一次（今日/周/月/历史/趋势/模型/内存）。
//! - 后端 `usage-updated` 每 N 秒推送一次"今日"完整字段，这里只 `setToday`，
//!   不再全量回查，避免每个窗口随定时器重扫五数据源。
//! - 周/月/历史/趋势/模型分布仅在挂载或手动 `refresh()` 时更新。
//! - 切换趋势天数只拉 daily，不影响其它状态。

import { useState, useEffect, useCallback, useRef } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import {
  getTodayStats,
  getWeekStats,
  getMonthStats,
  getAllTimeStats,
  getDailyUsage,
  getTodayModelStats,
  getTodaySourceStats,
  getMemoryUsage,
  type UsageStats,
  type DailyUsage,
  type ModelUsage,
  type MemoryInfo,
  type SourceReport,
  type UsageUpdatedPayload,
  type MemoryUpdatedPayload,
} from "../lib/commands";

interface UsageData {
  today: UsageStats | null;
  week: UsageStats | null;
  month: UsageStats | null;
  allTime: UsageStats | null;
  daily: DailyUsage[];
  models: ModelUsage[];
  sources: SourceReport[];
  memory: MemoryInfo | null;
  loading: boolean;
  error: string | null;
  /** 切换趋势天数后调用，仅刷新 daily */
  refreshDaily: (days: number) => Promise<void>;
  /** 全量刷新（手动触发） */
  refresh: () => Promise<void>;
}

export function useUsageData(initialDays: number = 30): UsageData {
  const [today, setToday] = useState<UsageStats | null>(null);
  const [week, setWeek] = useState<UsageStats | null>(null);
  const [month, setMonth] = useState<UsageStats | null>(null);
  const [allTime, setAllTime] = useState<UsageStats | null>(null);
  const [daily, setDaily] = useState<DailyUsage[]>([]);
  const [models, setModels] = useState<ModelUsage[]>([]);
  const [sources, setSources] = useState<SourceReport[]>([]);
  const [memory, setMemory] = useState<MemoryInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // 趋势天数用 ref 持有，避免进入 refresh 依赖导致身份变化
  const daysRef = useRef<number>(initialDays);
  // 标记是否已首次加载，避免 usage-updated 在首屏完成前覆盖
  const loadedRef = useRef<boolean>(false);

  /** 全量加载（空依赖，身份稳定） */
  const refresh = useCallback(async () => {
    try {
      const [t, w, m, a, d, mod, src, mem] = await Promise.all([
        getTodayStats(),
        getWeekStats(),
        getMonthStats(),
        getAllTimeStats(),
        getDailyUsage(daysRef.current),
        getTodayModelStats(),
        getTodaySourceStats(),
        getMemoryUsage(),
      ]);
      setToday(t);
      setWeek(w);
      setMonth(m);
      setAllTime(a);
      setDaily(d);
      setModels(mod);
      setSources(src);
      setMemory(mem);
      setError(null);
      loadedRef.current = true;
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  /** 仅刷新每日趋势数据（切换天数时调用，避免重复加载不变的统计） */
  const refreshDaily = useCallback(async (nextDays: number) => {
    daysRef.current = nextDays;
    try {
      const d = await getDailyUsage(nextDays);
      setDaily(d);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  // 初始加载（refresh 稳定，只跑一次）
  useEffect(() => {
    void refresh();
  }, [refresh]);

  // 监听后端推送事件（依赖稳定，只注册一次）
  useEffect(() => {
    let unlisteners: UnlistenFn[] = [];

    async function setupListeners() {
      // 今日走事件：payload 已含完整字段，直接更新，不回查
      const u1 = await listen<UsageUpdatedPayload>("usage-updated", (event) => {
        const p = event.payload;
        if (!loadedRef.current) return; // 首屏完成前不覆盖
        setToday({
          inputTokens: p.inputTokens,
          outputTokens: p.outputTokens,
          cacheReadTokens: p.cacheReadTokens,
          cacheWriteTokens: p.cacheWriteTokens,
          reasoningTokens: p.reasoningTokens,
          sessions: p.sessions,
          costUsd: p.costUsd,
        });
      });

      const u2 = await listen<MemoryUpdatedPayload>("memory-updated", (event) => {
        const pct = event.payload.memPercent;
        setMemory((prev) =>
          prev
            ? { ...prev, usagePercent: pct }
            : { usagePercent: pct, totalGb: 0, usedGb: 0, availableGb: 0 }
        );
      });

      unlisteners = [u1, u2];
    }

    setupListeners();

    return () => {
      unlisteners.forEach((u) => u());
    };
  }, []);

  return { today, week, month, allTime, daily, models, sources, memory, loading, error, refreshDaily, refresh };
}
