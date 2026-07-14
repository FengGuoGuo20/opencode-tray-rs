//! 各数据源今日贡献列表组件
//!
//! 显示五数据源的今日 token、健康状态徽标和路径，便于对照 WPF 的 SourceStats。

import type { SourceReport } from "../../lib/commands";

interface SourceListProps {
  sources: SourceReport[];
  fmt: (n: number | undefined) => string;
}

/** 状态徽标颜色 */
function statusColor(status: string): string {
  switch (status) {
    case "ok":
      return "#10B981"; // 绿
    case "not_found":
      return "#64748B"; // 灰
    case "error":
      return "#EF4444"; // 红
    default:
      return "#64748B";
  }
}

export default function SourceList({ sources, fmt }: SourceListProps) {
  if (sources.length === 0) {
    return <div className="text-xs text-[#64748B] py-2">暂无数据</div>;
  }

  const maxTokens = Math.max(...sources.map((s) => s.totalTokens), 1);

  return (
    <div className="space-y-1.5">
      {sources.map((s) => {
        const pct = (s.totalTokens / maxTokens) * 100;
        return (
          <div key={s.sourceId} className="flex items-center gap-2 text-xs">
            {/* 状态点 */}
            <div
              className="w-2 h-2 rounded-full flex-shrink-0"
              style={{ backgroundColor: statusColor(s.status) }}
              title={s.statusText}
            />

            {/* 源名 */}
            <div
              className="w-24 truncate text-[#E2E8F0]"
              title={`${s.sourceName}\n${s.path}\n${s.detailText}`}
            >
              {s.sourceName}
            </div>

            {/* 进度条 */}
            <div className="flex-1 h-3 bg-[#1E293B] rounded-full overflow-hidden">
              <div
                className="h-full bg-gradient-to-r from-[#3B82F6] to-[#8B5CF6] rounded-full"
                style={{ width: `${pct}%` }}
              />
            </div>

            {/* Token 数 */}
            <div className="w-16 text-right text-[#94A3B8] font-mono">
              {fmt(s.totalTokens)}
            </div>
          </div>
        );
      })}

      {/* 路径明细（小字，便于诊断） */}
      <div className="mt-2 pt-2 border-t border-[#1E293B] space-y-0.5">
        {sources.map((s) => (
          <div key={s.sourceId} className="flex items-center gap-1.5 text-[10px] text-[#64748B]">
            <span
              className="inline-block w-1.5 h-1.5 rounded-full flex-shrink-0"
              style={{ backgroundColor: statusColor(s.status) }}
            />
            <span className="text-[#94A3B8] flex-shrink-0">{s.sourceName}</span>
            <span className="truncate" title={s.path}>
              {s.path}
            </span>
            {s.status !== "ok" && (
              <span className="text-[#EF4444] flex-shrink-0">{s.statusText}</span>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
