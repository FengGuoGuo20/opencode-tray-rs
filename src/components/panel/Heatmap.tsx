//! 活跃热力图组件
//!
//! 仿 GitHub 贡献热力图，5 级蓝色色阶，按周排列。
//! 与 WPF 版本对齐：Level 0-4，颜色 #1E2D45 / #1E3A5F / #2563EB / #60A5FA / #93C5FD。

import { useMemo } from "react";
import type { DailyUsage } from "../../lib/commands";

/** 热力图颜色映射（与 WPF HeatmapLevelToBrushConverter 一致） */
const LEVEL_COLORS = ["#1E2D45", "#1E3A5F", "#2563EB", "#60A5FA", "#93C5FD"];

interface HeatmapCell {
  date: string; // YYYY-MM-DD
  totalTokens: number;
  sessionCount: number;
  level: number; // 0-4
  inRange: boolean; // 是否在选定的天数范围内
}

interface HeatmapProps {
  data: DailyUsage[];
  days: number;
}

/** 本地时区日期格式化，避免 toISOString 的 UTC 偏移 */
function formatLocalDate(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${y}-${m}-${day}`;
}

/** 将每日数据按日期索引 */
function buildDateMap(data: DailyUsage[]): Map<string, DailyUsage> {
  const map = new Map<string, DailyUsage>();
  for (const d of data) {
    map.set(d.date, d);
  }
  return map;
}

/** 计算热力图色阶等级 */
function calcLevel(tokens: number, maxTokens: number): number {
  if (tokens <= 0 || maxTokens <= 0) return 0;
  const ratio = tokens / maxTokens;
  if (ratio <= 0.25) return 1;
  if (ratio <= 0.5) return 2;
  if (ratio <= 0.75) return 3;
  return 4;
}

/** 格式化 token 数为简短文本 */
function fmtShort(n: number): string {
  if (n >= 1_0000_0000) return `${(n / 1_0000_0000).toFixed(1)}亿`;
  if (n >= 1_0000) return `${(n / 1_0000).toFixed(0)}万`;
  if (n >= 1000) return `${(n / 1000).toFixed(0)}K`;
  return n.toString();
}

export default function Heatmap({ data, days }: HeatmapProps) {
  const weeks = useMemo((): HeatmapCell[][] => {
    const dateMap = buildDateMap(data);
    const today = new Date();
    today.setHours(0, 0, 0, 0);
    const startDate = new Date(today);
    startDate.setDate(startDate.getDate() - days + 1);

    // 构建所有日期的单元格
    const cells: HeatmapCell[] = [];
    let maxT = 0;
    for (let i = 0; i < days; i++) {
      const d = new Date(startDate);
      d.setDate(d.getDate() + i);
      const dateStr = formatLocalDate(d);
      const usage = dateMap.get(dateStr);
      const tokens = usage
        ? usage.totalInputTokens +
          usage.totalOutputTokens +
          usage.totalCacheReadTokens +
          usage.totalCacheWriteTokens +
          usage.totalReasoningTokens
        : 0;
      const sessionCount = usage?.sessionCount ?? 0;
      if (tokens > maxT) maxT = tokens;
      cells.push({ date: dateStr, totalTokens: tokens, sessionCount, level: 0, inRange: true });
    }

    // 计算色阶
    for (const cell of cells) {
      cell.level = calcLevel(cell.totalTokens, maxT);
    }

    // 对齐到整周：从 startDate 前的周日开始
    const gridStart = new Date(startDate);
    gridStart.setDate(gridStart.getDate() - gridStart.getDay());

    // 对齐到整周：到 endDate 后的周六结束
    const gridEnd = new Date(today);
    gridEnd.setDate(gridEnd.getDate() + (6 - gridEnd.getDay()));

    const totalDays = Math.round((gridEnd.getTime() - gridStart.getTime()) / 86400000) + 1;
    const weekCount = Math.ceil(totalDays / 7);

    const result: HeatmapCell[][] = [];
    for (let w = 0; w < weekCount; w++) {
      const week: HeatmapCell[] = [];
      for (let d = 0; d < 7; d++) {
        const cellDate = new Date(gridStart);
        cellDate.setDate(cellDate.getDate() + w * 7 + d);
        const dateStr = formatLocalDate(cellDate);
        const existing = cells.find((c) => c.date === dateStr);
        if (existing) {
          week.push(existing);
        } else {
          week.push({ date: dateStr, totalTokens: 0, sessionCount: 0, level: 0, inRange: false });
        }
      }
      result.push(week);
    }

    return result;
  }, [data, days]);

  if (weeks.length === 0) {
    return <div className="h-24 flex items-center justify-center text-xs text-[#64748B]">暂无数据</div>;
  }

  return (
    <div>
      {/* 热力图网格 */}
      <div className="overflow-x-auto pb-1">
        <div className="inline-flex flex-col gap-[2px]" style={{ minWidth: "fit-content" }}>
          {["日", "一", "二", "三", "四", "五", "六"].map((dayLabel, rowIdx) => (
            <div key={dayLabel} className="flex items-center gap-[2px]">
              <span className="w-5 text-[8px] text-[#475569] text-right mr-1">
                {rowIdx % 2 === 1 ? dayLabel : ""}
              </span>
              {weeks.map((week: HeatmapCell[], colIdx: number) => {
                const cell = week[rowIdx];
                if (!cell) return <div key={colIdx} className="w-[13px] h-[13px]" />;
                return (
                  <div
                    key={colIdx}
                    className="w-[13px] h-[13px] rounded-[3px] flex-shrink-0 transition-colors"
                    style={{
                      backgroundColor: LEVEL_COLORS[cell.level],
                      opacity: cell.inRange ? 1 : 0,
                    }}
                    title={
                      cell.totalTokens > 0
                        ? `${cell.date} · ${fmtShort(cell.totalTokens)} · ${cell.sessionCount} 条`
                        : `${cell.date} · 无活动`
                    }
                  />
                );
              })}
            </div>
          ))}
        </div>
      </div>

      {/* 图例 */}
      <div className="flex items-center justify-end gap-1 mt-2">
        <span className="text-[9px] text-[#64748B]">较少</span>
        {LEVEL_COLORS.map((color, i) => (
          <div key={i} className="w-[10px] h-[10px] rounded-[2px]" style={{ backgroundColor: color }} />
        ))}
        <span className="text-[9px] text-[#64748B]">较多</span>
      </div>
    </div>
  );
}
