//! 详细面板组件
//!
//! 右下角弹出卡片，显示今日/本周/本月统计、数据源状态、30日趋势、模型分布。

import { useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useUsageData } from "../../hooks/useUsageData";
import StatsCards from "./StatsCards";
import TrendChart from "./TrendChart";
import ModelList from "./ModelList";

export default function Panel() {
  const { today, week, month, daily, models, memory, loading } = useUsageData(30);
  const [range, setRange] = useState<number>(30);

  const handleClose = () => {
    const win = getCurrentWindow();
    win.hide();
  };

  // 格式化 token 数
  const fmt = (n: number | undefined): string => {
    if (!n) return "0";
    if (n >= 1_0000_0000) return `${(n / 1_0000_0000).toFixed(2)}亿`;
    if (n >= 1_0000) return `${(n / 1_0000).toFixed(1)}万`;
    if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
    return n.toString();
  };

  // 格式化费用
  const fmtCost = (n: number | undefined): string => {
    if (!n) return "$0";
    if (n >= 1) return `$${n.toFixed(2)}`;
    if (n >= 0.01) return `$${n.toFixed(3)}`;
    if (n > 0) return `$${n.toFixed(4)}`;
    return "$0";
  };

  return (
    <div className="w-full h-full bg-[#0F172A] text-[#E2E8F0] overflow-auto p-5 select-text">
      {/* 标题栏 - 可拖拽 */}
      <div className="flex items-center justify-between mb-4" data-tauri-drag-region>
        <h1 className="text-lg font-bold text-[#3B82F6]">📊 OpenCodeTray 用量统计</h1>
        <div className="flex items-center gap-3">
          <span className="text-xs text-[#64748B]">
            内存 {memory?.usagePercent.toFixed(0) ?? 0}%
          </span>
          <button
            onClick={handleClose}
            className="w-6 h-6 flex items-center justify-center rounded hover:bg-[#1E293B] text-[#64748B] hover:text-white transition-colors"
          >
            ✕
          </button>
        </div>
      </div>

      {loading ? (
        <div className="flex items-center justify-center h-64 text-[#64748B]">加载中...</div>
      ) : (
        <>
          {/* 统计卡片 */}
          <StatsCards
            today={today}
            week={week}
            month={month}
            fmt={fmt}
            fmtCost={fmtCost}
          />

          {/* 30日趋势 */}
          <div className="mt-4 p-3 rounded-lg bg-[#111827] border border-[#1E293B]">
            <div className="flex items-center justify-between mb-2">
              <h2 className="text-sm font-semibold text-[#94A3B8]">趋势</h2>
              <div className="flex gap-1">
                {[7, 14, 30].map((d) => (
                  <button
                    key={d}
                    onClick={() => setRange(d)}
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
            <ModelList models={models} fmt={fmt} fmtCost={fmtCost} />
          </div>
        </>
      )}
    </div>
  );
}
