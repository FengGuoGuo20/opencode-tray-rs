//! 30日趋势图表组件
//!
//! 使用纯 SVG 绘制柱状图（不依赖第三方图表库，保持小体积）。

import { useMemo } from "react";
import type { DailyUsage } from "../../lib/commands";

interface TrendChartProps {
  data: DailyUsage[];
}

export default function TrendChart({ data }: TrendChartProps) {
  const chart = useMemo(() => {
    if (data.length === 0) return null;

    const width = 760;
    const height = 160;
    const padding = { top: 10, right: 10, bottom: 25, left: 45 };
    const chartW = width - padding.left - padding.right;
    const chartH = height - padding.top - padding.bottom;

    // 计算最大值
    const maxInput = Math.max(...data.map((d) => d.totalInputTokens), 1);
    const maxOutput = Math.max(...data.map((d) => d.totalOutputTokens), 1);
    const maxVal = maxInput + maxOutput;

    // 自动选择单位
    let divisor = 1;
    let unit = "";
    if (maxVal >= 1_000_000_000) { divisor = 1_000_000_000; unit = "B"; }
    else if (maxVal >= 1_000_000) { divisor = 1_000_000; unit = "M"; }
    else if (maxVal >= 1_000) { divisor = 1_000; unit = "K"; }

    const barGroupWidth = chartW / data.length;
    const barWidth = Math.min(barGroupWidth * 0.35, 12);
    const gap = 2;

    // Y轴刻度
    const yTicks = 4;
    const yLabels = Array.from({ length: yTicks + 1 }, (_, i) => {
      const val = (maxVal / yTicks) * (yTicks - i);
      return { y: padding.top + (chartH / yTicks) * i, label: `${(val / divisor).toFixed(1)}${unit}` };
    });

    // 柱状图 bars
    const bars = data.map((d, i) => {
      const x = padding.left + barGroupWidth * i + barGroupWidth / 2;
      const inputH = (d.totalInputTokens / maxVal) * chartH;
      const outputH = (d.totalOutputTokens / maxVal) * chartH;

      return {
        x,
        inputY: padding.top + chartH - inputH,
        inputH,
        outputY: padding.top + chartH - inputH - outputH,
        outputH,
        label: d.date.slice(5), // MM/DD
      };
    });

    return { width, height, padding, yLabels, bars, barWidth, gap, chartH };
  }, [data]);

  if (!chart) {
    return <div className="h-40 flex items-center justify-center text-xs text-[#64748B]">暂无数据</div>;
  }

  const { width, height, padding, yLabels, bars, barWidth, gap } = chart;

  return (
    <svg viewBox={`0 0 ${width} ${height}`} className="w-full h-40">
      {/* Y轴刻度线 + 标签 */}
      {yLabels.map((tick, i) => (
        <g key={i}>
          <line
            x1={padding.left}
            y1={tick.y}
            x2={width - padding.right}
            y2={tick.y}
            stroke="#1E2D45"
            strokeWidth={1}
          />
          <text x={padding.left - 5} y={tick.y + 3} textAnchor="end" fill="#64748B" fontSize={9}>
            {tick.label}
          </text>
        </g>
      ))}

      {/* 柱状图 */}
      {bars.map((bar, i) => (
        <g key={i}>
          {/* 输入 (蓝色) */}
          <rect
            x={bar.x - barWidth - gap / 2}
            y={bar.inputY}
            width={barWidth}
            height={Math.max(bar.inputH, 0.5)}
            fill="#3B82F6"
            rx={1}
          />
          {/* 输出 (橙色) */}
          <rect
            x={bar.x + gap / 2}
            y={bar.outputY}
            width={barWidth}
            height={Math.max(bar.outputH, 0.5)}
            fill="#F59E0B"
            rx={1}
          />
          {/* X轴标签 */}
          {data.length <= 15 || i % 2 === 0 ? (
            <text x={bar.x} y={height - 3} textAnchor="middle" fill="#64748B" fontSize={8}>
              {bar.label}
            </text>
          ) : null}
        </g>
      ))}

      {/* 图例 */}
      <rect x={width - 120} y={5} width={8} height={8} fill="#3B82F6" rx={1} />
      <text x={width - 108} y={12} fill="#94A3B8" fontSize={9}>输入</text>
      <rect x={width - 65} y={5} width={8} height={8} fill="#F59E0B" rx={1} />
      <text x={width - 53} y={12} fill="#94A3B8" fontSize={9}>输出</text>
    </svg>
  );
}
