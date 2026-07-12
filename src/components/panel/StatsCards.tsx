//! 统计卡片组件

import type { UsageStats } from "../../lib/commands";

interface StatsCardsProps {
  today: UsageStats | null;
  week: UsageStats | null;
  month: UsageStats | null;
  fmt: (n: number | undefined) => string;
  fmtCost: (n: number | undefined) => string;
}

interface CardData {
  label: string;
  stats: UsageStats | null;
  color: string;
}

export default function StatsCards({ today, week, month, fmt, fmtCost }: StatsCardsProps) {
  const cards: CardData[] = [
    { label: "今日", stats: today, color: "#3B82F6" },
    { label: "本周", stats: week, color: "#8B5CF6" },
    { label: "本月", stats: month, color: "#F59E0B" },
  ];

  return (
    <div className="grid grid-cols-3 gap-3">
      {cards.map(({ label, stats, color }) => (
        <div
          key={label}
          className="p-3 rounded-lg bg-[#111827] border border-[#1E293B] hover:border-[#334155] transition-colors"
        >
          <div className="flex items-center gap-1.5 mb-2">
            <div className="w-2 h-2 rounded-full" style={{ backgroundColor: color }} />
            <span className="text-xs font-medium text-[#94A3B8]">{label}</span>
          </div>
          <div className="text-xl font-bold text-[#E2E8F0] mb-1">
            {fmt(stats?.inputTokens ? stats.inputTokens + (stats.outputTokens ?? 0) : 0)}
            <span className="text-xs font-normal text-[#64748B] ml-1">tokens</span>
          </div>
          <div className="flex flex-col gap-0.5 text-xs text-[#64748B]">
            <div className="flex justify-between">
              <span>输入</span>
              <span className="text-[#94A3B8]">{fmt(stats?.inputTokens)}</span>
            </div>
            <div className="flex justify-between">
              <span>输出</span>
              <span className="text-[#94A3B8]">{fmt(stats?.outputTokens)}</span>
            </div>
            <div className="flex justify-between">
              <span>缓存读</span>
              <span className="text-[#94A3B8]">{fmt(stats?.cacheReadTokens)}</span>
            </div>
            <div className="flex justify-between">
              <span>缓存写</span>
              <span className="text-[#94A3B8]">{fmt(stats?.cacheWriteTokens)}</span>
            </div>
            <div className="flex justify-between">
              <span>推理</span>
              <span className="text-[#94A3B8]">{fmt(stats?.reasoningTokens)}</span>
            </div>
            <div className="flex justify-between border-t border-[#1E293B] pt-0.5 mt-0.5">
              <span>费用</span>
              <span className="text-[#F59E0B]">{fmtCost(stats?.costUsd)}</span>
            </div>
            <div className="flex justify-between">
              <span>会话</span>
              <span className="text-[#94A3B8]">{stats?.sessions ?? 0}</span>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}
