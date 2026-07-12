//! 悬浮条组件
//!
//! 透明悬浮窗口，显示 token 数 + 内存百分比。
//! 监听后端 usage-updated / memory-updated 事件实时更新。

import { useState, useEffect } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { getTrayDisplay, showPanel } from "../../lib/commands";

export default function FloatingBar() {
  const [tokenText, setTokenText] = useState("0");
  const [memPercent, setMemPercent] = useState(0);
  const [_costText, setCostText] = useState("$0");

  // 初始加载
  useEffect(() => {
    getTrayDisplay()
      .then((data) => {
        setTokenText(data.tokenText);
        setMemPercent(data.memPercent);
        setCostText(data.costText);
      })
      .catch(console.error);
  }, []);

  // 监听后端推送
  useEffect(() => {
    let unlisteners: UnlistenFn[] = [];

    async function setup() {
      const u1 = await listen<{
        tokenText: string;
        memPercent: number;
        costUsd: number;
      }>("usage-updated", (event) => {
        setTokenText(event.payload.tokenText);
        setMemPercent(event.payload.memPercent);
        // 格式化费用
        const cost = event.payload.costUsd;
        if (cost >= 1) setCostText(`$${cost.toFixed(2)}`);
        else if (cost >= 0.01) setCostText(`$${cost.toFixed(3)}`);
        else if (cost > 0) setCostText(`$${cost.toFixed(4)}`);
        else setCostText("$0");
      });

      const u2 = await listen<{ memPercent: number }>("memory-updated", (event) => {
        setMemPercent(event.payload.memPercent);
      });

      unlisteners = [u1, u2];
    }

    setup();
    return () => unlisteners.forEach((u) => u());
  }, []);

  // 双击打开面板
  const handleDoubleClick = async () => {
    await showPanel();
  };

  // 拖拽：使用 Tauri startDragging
  const handleMouseDown = (e: React.MouseEvent) => {
    if (e.button === 0) {
      getCurrentWindow().startDragging();
    }
  };

  return (
    <div
      className="flex items-center justify-center h-screen w-screen bg-transparent select-none cursor-grab active:cursor-grabbing"
      onMouseDown={handleMouseDown}
      onDoubleClick={handleDoubleClick}
    >
      <div className="flex items-center gap-1.5 px-3 py-1 rounded-full bg-[rgba(15,23,42,0.85)] border border-[rgba(51,65,85,0.6)] backdrop-blur-sm shadow-lg">
        <span className="text-[11px] font-bold text-[#3B82F6] font-mono">🔵</span>
        <span className="text-[11px] font-bold text-[#E2E8F0] font-mono">
          {tokenText}
        </span>
        <span className="text-[11px] font-bold text-[#F59E0B] font-mono">·</span>
        <span className="text-[11px] font-bold text-[#E2E8F0] font-mono">
          {memPercent.toFixed(0)}%
        </span>
      </div>
    </div>
  );
}
