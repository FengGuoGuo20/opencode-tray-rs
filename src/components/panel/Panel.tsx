//! 详细面板组件
//!
//! 右下角弹出卡片，显示今日/本周/本月/历史总统计、活跃热力图、30日趋势、模型分布。

import { useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useUsageData } from "../../hooks/useUsageData";
import { showSettings } from "../../lib/commands";
import StatsCards from "./StatsCards";
import SourceList from "./SourceList";
import Heatmap from "./Heatmap";
import TrendChart from "./TrendChart";
import ModelList from "./ModelList";

export default function Panel() {
  const { today, week, month, allTime, daily, models, sources, memory, loading, refreshDaily, refresh } = useUsageData(30);
  const [range, setRange] = useState<number>(30);
  const [refreshing, setRefreshing] = useState(false);

  const handleClose = () => {
    const win = getCurrentWindow();
    win.hide();
  };

  const handleOpenSettings = async () => {
    await showSettings();
  };

  // 手动全量刷新（今日/周/月/历史/趋势/模型）
  const handleRefresh = async () => {
    if (refreshing) return;
    setRefreshing(true);
    try {
      await refresh();
    } finally {
      setRefreshing(false);
    }
  };

  // 切换天数范围
  const handleRangeChange = (days: number) => {
    setRange(days);
    refreshDaily(days);
  };

  // 格式化 token 数
  const fmt = (n: number | undefined): string => {
    if (!n) return "0";
    if (n >= 1_0000_0000) return `${(n / 1_0000_0000).toFixed(2)}亿`;
    if (n >= 1_0000) return `${(n / 1_0000).toFixed(1)}万`;
    if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
    return n.toString();
  };

  return (
    <div className="flex flex-col w-full h-full bg-[#0F172A] text-[#E2E8F0] overflow-hidden select-text">
      {/* 拖拽标题栏 - 固定在顶部，使用 WebkitAppRegion 实现拖拽 */}
      <div
        className="flex-shrink-0 flex items-center justify-between px-5 py-2.5 border-b border-[#1E293B]"
        data-tauri-drag-region
        style={{ WebkitAppRegion: "drag" } as React.CSSProperties}
      >
        <h1 className="text-lg font-bold text-[#3B82F6] pointer-events-none">📊 OpenCodeTray 用量统计</h1>
        <div className="flex items-center gap-2" style={{ WebkitAppRegion: "no-drag" } as React.CSSProperties}>
          <span className="text-xs text-[#64748B]">
            内存 {memory?.usagePercent.toFixed(0) ?? 0}%
          </span>
          <button
            onClick={handleRefresh}
            disabled={refreshing}
            title={refreshing ? "刷新中..." : "刷新"}
            className="w-6 h-6 flex items-center justify-center rounded hover:bg-[#1E293B] text-[#64748B] hover:text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            ↻
          </button>
          <button
            onClick={handleOpenSettings}
            title="设置"
            className="w-6 h-6 flex items-center justify-center rounded hover:bg-[#1E293B] text-[#64748B] hover:text-white transition-colors"
          >
            ⚙
          </button>
          <button
            onClick={handleClose}
            className="w-6 h-6 flex items-center justify-center rounded hover:bg-[#1E293B] text-[#64748B] hover:text-white transition-colors"
          >
            ✕
          </button>
        </div>
      </div>

      {/* 可滚动内容区域 */}
      <div className="flex-1 overflow-auto p-5">
        {loading ? (
          <div className="flex items-center justify-center h-64 text-[#64748B]">加载中...</div>
        ) : (
          <>
            {/* 统计卡片（今日/本周/本月/历史总） */}
            <StatsCards
              today={today}
              week={week}
              month={month}
              allTime={allTime}
              fmt={fmt}
            />

            {/* 各数据源今日贡献 + 状态 */}
            <div className="mt-4 p-3 rounded-lg bg-[#111827] border border-[#1E293B]">
              <h2 className="text-sm font-semibold text-[#94A3B8] mb-2">各数据源今日</h2>
              <SourceList sources={sources} fmt={fmt} />
            </div>

            {/* 活跃热力图 */}
            <div className="mt-4 p-3 rounded-lg bg-[#111827] border border-[#1E293B]">
              <div className="flex items-center justify-between mb-2">
                <h2 className="text-sm font-semibold text-[#94A3B8]">活跃热力图</h2>
                <div className="flex gap-1">
                  {[7, 30].map((d) => (
                    <button
                      key={d}
                      onClick={() => handleRangeChange(d)}
                      className={`px-2 py-0.5 text-xs rounded transition-colors ${
                        range === d
                          ? "bg-[#3B82F6] text-white"
                          : "bg-[#1E293B] text-[#64748B] hover:text-white"
                      }`}
                    >
                      最近 {d} 天
                    </button>
                  ))}
                </div>
              </div>
              <Heatmap data={daily} days={range} />
            </div>

            {/* 趋势柱状图 */}
            <div className="mt-4 p-3 rounded-lg bg-[#111827] border border-[#1E293B]">
              <div className="flex items-center justify-between mb-2">
                <h2 className="text-sm font-semibold text-[#94A3B8]">趋势</h2>
                <div className="flex gap-1">
                  {[7, 14, 30].map((d) => (
                    <button
                      key={d}
                      onClick={() => handleRangeChange(d)}
                      className={`px-2 py-0.5 text-xs rounded transition-colors ${
                        range === d
                          ? "bg-[#3B82F6] text-white"
                          : "bg-[#1E293B] text-[#64748B] hover:text-white"
                      }`}
                    >
                      {d}天
                    </button>
                  ))}
                </div>
              </div>
              <TrendChart data={daily} />
            </div>

            {/* 模型分布 */}
            <div className="mt-4 p-3 rounded-lg bg-[#111827] border border-[#1E293B]">
              <h2 className="text-sm font-semibold text-[#94A3B8] mb-2">今日模型分布</h2>
              <ModelList models={models} fmt={fmt} />
            </div>
          </>
        )}
      </div>
    </div>
  );
}
