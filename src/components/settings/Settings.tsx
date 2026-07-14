//! 设置面板组件

import { useState, useEffect } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  getSettings,
  saveSettings,
  type AppSettings,
} from "../../lib/commands";

export default function Settings() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [saved, setSaved] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getSettings().then(setSettings).catch(console.error);
  }, []);

  const handleClose = () => {
    const win = getCurrentWindow();
    win.hide();
  };

  const handleChange = (
    key: keyof AppSettings,
    value: string | number | boolean | null
  ) => {
    if (!settings) return;
    setSettings({ ...settings, [key]: value });
    setSaved(false);
  };

  const handleSave = async () => {
    if (!settings) return;
    try {
      setError(null);
      await saveSettings(settings);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error("保存设置失败:", e);
      setError(String(e));
    }
  };

  if (!settings) {
    return (
      <div className="w-full h-full bg-[#0F172A] text-[#64748B] flex items-center justify-center">
        加载设置中...
      </div>
    );
  }

  return (
    <div className="flex flex-col w-full h-full bg-[#0F172A] text-[#E2E8F0] overflow-hidden select-text">
      {/* 拖拽标题栏 */}
      <div
        className="flex-shrink-0 flex items-center justify-between px-5 py-2.5 border-b border-[#1E293B]"
        data-tauri-drag-region
        style={{ WebkitAppRegion: "drag" } as React.CSSProperties}
      >
        <h1 className="text-lg font-bold text-[#3B82F6] pointer-events-none">⚙️ 设置</h1>
        <button
          onClick={handleClose}
          className="w-6 h-6 flex items-center justify-center rounded hover:bg-[#1E293B] text-[#64748B] hover:text-white transition-colors"
          style={{ WebkitAppRegion: "no-drag" } as React.CSSProperties}
        >
          ✕
        </button>
      </div>

      <div className="flex-1 overflow-auto p-5">
        <div className="space-y-4 max-w-lg mx-auto">
          {/* 通用设置 */}
          <div className="p-3 rounded-lg bg-[#111827] border border-[#1E293B]">
            <h2 className="text-sm font-semibold text-[#94A3B8] mb-3">通用</h2>

            <div className="space-y-3">
              <div className="flex items-center justify-between">
                <label className="text-sm text-[#94A3B8]">刷新间隔（秒）</label>
                <input
                  type="number"
                  min={5}
                  max={300}
                  value={settings.refreshIntervalSecs}
                  onChange={(e) =>
                    handleChange("refreshIntervalSecs", parseInt(e.target.value) || 30)
                  }
                  className="w-20 px-2 py-1 text-sm bg-[#1E293B] border border-[#334155] rounded text-[#E2E8F0] text-right"
                />
              </div>

              <div className="flex items-center justify-between">
                <label className="text-sm text-[#94A3B8]">悬浮条显示</label>
                <select
                  value={
                    settings.trayDisplayMode === "cost_mem" ||
                    settings.trayDisplayMode === "cost_only"
                      ? "token_mem"
                      : settings.trayDisplayMode
                  }
                  onChange={(e) => handleChange("trayDisplayMode", e.target.value)}
                  className="px-2 py-1 text-sm bg-[#1E293B] border border-[#334155] rounded text-[#E2E8F0]"
                >
                  <option value="token_mem">Token + 内存</option>
                  <option value="token_only">仅 Token</option>
                  <option value="mem_only">仅内存</option>
                </select>
              </div>

              <div className="flex items-center justify-between">
                <label className="text-sm text-[#94A3B8]">开机自启</label>
                <button
                  type="button"
                  role="switch"
                  aria-checked={!!settings.startWithWindows}
                  onClick={() =>
                    handleChange("startWithWindows", !settings.startWithWindows)
                  }
                  className={`relative w-11 h-6 rounded-full transition-colors ${
                    settings.startWithWindows ? "bg-[#3B82F6]" : "bg-[#334155]"
                  }`}
                >
                  <span
                    className={`absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white transition-transform ${
                      settings.startWithWindows ? "translate-x-5" : "translate-x-0"
                    }`}
                  />
                </button>
              </div>
              <p className="text-[11px] text-[#64748B] -mt-1">
                开启后写入当前用户启动项，下次登录 Windows 时自动运行
              </p>
            </div>
          </div>

          {/* 数据源路径覆盖 */}
          <div className="p-3 rounded-lg bg-[#111827] border border-[#1E293B]">
            <h2 className="text-sm font-semibold text-[#94A3B8] mb-3">
              数据源路径（留空使用默认）
            </h2>

            <div className="space-y-2">
              {(
                [
                  ["opencodeDbPath", "OpenCode DB"],
                  ["ccswitchDbPath", "CC Switch DB"],
                  ["workbuddyDirPath", "WorkBuddy 目录"],
                  ["hermesDbPath", "Hermes DB"],
                  ["zcodeDbPath", "ZCode DB"],
                ] as const
              ).map(([key, label]) => (
                <div key={key} className="flex items-center gap-2">
                  <label className="text-xs text-[#64748B] w-28 shrink-0">{label}</label>
                  <input
                    type="text"
                    placeholder="默认路径"
                    value={settings[key] ?? ""}
                    onChange={(e) => handleChange(key, e.target.value || null)}
                    className="flex-1 px-2 py-1 text-xs bg-[#1E293B] border border-[#334155] rounded text-[#E2E8F0]"
                  />
                </div>
              ))}
            </div>
          </div>

          {/* 保存按钮 */}
          <button
            onClick={handleSave}
            className={`w-full py-2 rounded-lg text-sm font-medium transition-colors ${
              saved
                ? "bg-[#22C55E] text-white"
                : "bg-[#3B82F6] text-white hover:bg-[#2563EB]"
            }`}
          >
            {saved ? "✓ 已保存并生效" : "保存设置"}
          </button>
          {error && (
            <p className="text-xs text-red-400 text-center">{error}</p>
          )}
          <p className="text-[11px] text-[#64748B] text-center">
            保存后立即生效：刷新间隔、显示模式、路径覆盖、开机自启
          </p>
        </div>
      </div>
    </div>
  );
}
