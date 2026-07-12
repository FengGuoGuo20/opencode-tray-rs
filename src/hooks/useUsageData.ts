//! 用量数据 Hook
//!
//! 监听 Tauri 后端推送的 usage-updated / memory-updated 事件，
//! 并提供主动查询方法。

import { useState, useEffect, useCallback } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import {
  getTodayStats,
  getWeekStats,
  getMonthStats,
  getAllTimeStats,
  getDailyUsage,
  getTodayModelStats,
  getMemoryUsage,
  type UsageStats,
  type DailyUsage,
  type ModelUsage,
  type MemoryInfo,
} from "../lib/commands";

interface UsageData {
  today: UsageStats | null;
  week: UsageStats | null;
  month: UsageStats | null;
  allTime: UsageStats | null;
  daily: DailyUsage[];
  models: ModelUsage[];
  memory: MemoryInfo | null;
  loading: boolean;
  error: string | null;
}

export function useUsageData(days: number = 30): UsageData {
  const [today, setToday] = useState<UsageStats | null>(null);
  const [week, setWeek] = useState<UsageStats | null>(null);
  const [month, setMonth] = useState<UsageStats | null>(null);
  const [allTime, setAllTime] = useState<UsageStats | null>(null);
  const [daily, setDaily] = useState<DailyUsage[]>([]);
  const [models, setModels] = useState<ModelUsage[]>([]);
  const [memory, setMemory] = useState<MemoryInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      const [t, w, m, a, d, mod, mem] = await Promise.all([
        getTodayStats(),
        getWeekStats(),
        getMonthStats(),
        getAllTimeStats(),
        getDailyUsage(days),
        getTodayModelStats(),
        getMemoryUsage(),
      ]);
      setToday(t);
      setWeek(w);
      setMonth(m);
      setAllTime(a);
      setDaily(d);
      setModels(mod);
      setMemory(mem);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, [days]);

  // 初始加载
  useEffect(() => {
    refresh();
  }, [refresh]);

  // 监听后端推送事件
  useEffect(() => {
    let unlisteners: UnlistenFn[] = [];

    async function setupListeners() {
      const u1 = await listen<UsageStats>("usage-updated", (event) => {
        // 后端推送今日数据更新
        setToday(event.payload);
      });

      const u2 = await listen<{ memPercent: number }>("memory-updated", (event) => {
        // 后端推送内存更新
        setMemory((prev) =>
          prev
            ? { ...prev, usagePercent: event.payload.memPercent }
            : { usagePercent: event.payload.memPercent, totalGb: 0, usedGb: 0, availableGb: 0 }
        );
      });

      unlisteners = [u1, u2];
    }

    setupListeners();

    return () => {
      unlisteners.forEach((u) => u());
    };
  }, []);

  return { today, week, month, allTime, daily, models, memory, loading, error };
}
