//! 模型分布列表组件

import type { ModelUsage } from "../../lib/commands";

interface ModelListProps {
  models: ModelUsage[];
  fmt: (n: number | undefined) => string;
  fmtCost: (n: number | undefined) => string;
}

export default function ModelList({ models, fmt, fmtCost }: ModelListProps) {
  if (models.length === 0) {
    return <div className="text-xs text-[#64748B] py-2">暂无数据</div>;
  }

  // 计算最大 token 用于进度条
  const maxTokens = Math.max(
    ...models.map((m) => m.inputTokens + m.outputTokens),
    1
  );

  return (
    <div className="space-y-1.5">
      {models.map((m, i) => {
        const total = m.inputTokens + m.outputTokens;
        const pct = (total / maxTokens) * 100;

        return (
          <div key={i} className="flex items-center gap-2 text-xs">
            {/* 模型名 */}
            <div className="w-48 truncate text-[#E2E8F0]" title={m.model}>
              {m.model}
            </div>

            {/* 进度条 */}
            <div className="flex-1 h-3 bg-[#1E293B] rounded-full overflow-hidden">
              <div
                className="h-full bg-gradient-to-r from-[#3B82F6] to-[#8B5CF6] rounded-full"
                style={{ width: `${pct}%` }}
              />
            </div>

            {/* Token 数 */}
            <div className="w-16 text-right text-[#94A3B8] font-mono">{fmt(total)}</div>

            {/* 费用 */}
            <div className="w-16 text-right text-[#F59E0B] font-mono">{fmtCost(m.costUsd)}</div>

            {/* 会话数 */}
            <div className="w-10 text-right text-[#64748B]">{m.sessions}次</div>
          </div>
        );
      })}
    </div>
  );
}
