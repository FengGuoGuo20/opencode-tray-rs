//! 悬浮条组件
//!
//! 透明悬浮窗口，显示 token / 费用 / 内存（按设置模式）。
//! 监听后端 usage-updated / memory-updated / settings-updated 事件实时更新。

import { useState, useEffect } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  getTrayDisplay,
  getSettings,
  showPanel,
  showSettings,
  type AppSettings,
  type UsageUpdatedPayload,
  type MemoryUpdatedPayload,
} from "../../lib/commands";

type DisplayMode = "token_mem" | "token_only" | "mem_only" | string;

function normalizeMode(mode?: string | null): DisplayMode {
  if (mode === "token_only" || mode === "mem_only" || mode === "token_mem") {
    return mode;
  }
  // 兼容旧的费用模式，统一回退到 Token + 内存
  return "token_mem";
}

export default function FloatingBar() {
  const [tokenText, setTokenText] = useState("0");
  const [memPercent, setMemPercent] = useState(0);
  const [mode, setMode] = useState<DisplayMode>("token_mem");

  // 初始加载
  useEffect(() => {
    Promise.all([getTrayDisplay(), getSettings()])
      .then(([data, settings]) => {
        setTokenText(data.tokenText);
        setMemPercent(data.memPercent);
        setMode(normalizeMode(data.displayMode || settings.trayDisplayMode));
      })
      .catch(console.error);
  }, []);

  // 监听后端推送
  useEffect(() => {
    let unlisteners: UnlistenFn[] = [];

    async function setup() {
      const u1 = await listen<UsageUpdatedPayload>("usage-updated", (event) => {
        setTokenText(event.payload.tokenText);
        setMemPercent(event.payload.memPercent);
        if (event.payload.displayMode) {
          setMode(normalizeMode(event.payload.displayMode));
        }
      });

      const u2 = await listen<MemoryUpdatedPayload>("memory-updated", (event) => {
        setMemPercent(event.payload.memPercent);
      });

      const u3 = await listen<AppSettings>("settings-updated", (event) => {
        if (event.payload.trayDisplayMode) {
          setMode(normalizeMode(event.payload.trayDisplayMode));
        }
      });

      unlisteners = [u1, u2, u3];
    }

    setup();
    return () => unlisteners.forEach((u) => u());
  }, []);

  // 双击打开面板
  const handleDoubleClick = async () => {
    await showPanel();
  };

  // 拖拽：仅本次会话有效；下次启动/显示会重新贴到右下角
  const handleMouseDown = async (e: React.MouseEvent) => {
    if (e.button !== 0) return;
    try {
      await getCurrentWindow().startDragging();
    } catch (err) {
      console.error("拖拽悬浮条失败:", err);
    }
  };

  // 右键打开设置
  const handleContextMenu = async (e: React.MouseEvent) => {
    e.preventDefault();
    await showSettings();
  };

  const memText = `${memPercent.toFixed(0)}%`;

  let primary = tokenText;
  let secondary: string | null = memText;
  switch (mode) {
    case "token_only":
      primary = tokenText;
      secondary = null;
      break;
    case "mem_only":
      primary = memText;
      secondary = null;
      break;
    case "token_mem":
    default:
      primary = tokenText;
      secondary = memText;
      break;
  }

  return (
    <div
      className="flex items-center justify-center h-screen w-screen bg-transparent select-none cursor-grab active:cursor-grabbing"
      style={{ background: "transparent", backgroundColor: "transparent" }}
      onMouseDown={handleMouseDown}
      onDoubleClick={handleDoubleClick}
      onContextMenu={handleContextMenu}
      title="双击打开面板 · 右键打开设置 · 拖动可移动（重启后回到右下角）"
    >
      <div className="flex items-center gap-1 px-1 bg-transparent">
        <span className="inline-block w-[5px] h-[5px] rounded-full bg-[#3B82F6] mr-[2px]" />
        <span className="text-[11px] font-bold text-[#E2E8F0] font-mono drop-shadow-[0_1px_2px_rgba(0,0,0,0.85)]">
          {primary}
        </span>
        {secondary !== null && (
          <>
            <span className="text-[11px] font-bold text-[#F59E0B] font-mono drop-shadow-[0_1px_2px_rgba(0,0,0,0.85)]">
              ·
            </span>
            <span className="text-[11px] font-bold text-[#E2E8F0] font-mono drop-shadow-[0_1px_2px_rgba(0,0,0,0.85)]">
              {secondary}
            </span>
          </>
        )}
      </div>
    </div>
  );
}
