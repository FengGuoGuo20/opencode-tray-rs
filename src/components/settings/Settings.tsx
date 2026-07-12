//! 设置面板组件

import { useState, useEffect } from "react";
import {
  getSettings,
  saveSettings,
  type AppSettings,
} from "../../lib/commands";

export default function Settings() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    getSettings().then(setSettings).catch(console.error);
  }, []);

  const handleChange = (key: keyof AppSettings, value: string | number | null) => {
    if (!settings) return;
    setSettings({ ...settings, [key]: value });
    setSaved(false);
  };

  const handleSave = async () => {
    if (!settings) return;
    try {
      await saveSettings(settings);
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error("保存设置失败:", e);
    }
  };

  if (!settings) {
    return <div className="p-5 text-[#64748B]">加载设置中...</div>;
  }

  return (
    <div className="w-full h-full bg-[#0F172A] text-[#E2E8F0] overflow-auto p-5 select-text">
      <h1 className="text-lg font-bold text-[#3B82F6] mb-4">⚙️ 设置</h1>

      <div className="space-y-4 max-w-lg">
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
                onChange={(e) => handleChange("refreshIntervalSecs", parseInt(e.target.value) || 30)}
                className="w-20 px-2 py-1 text-sm bg-[#1E293B] border border-[#334155] rounded text-[#E2E8F0] text-right"
              />
            </div>

            <div className="flex items-center justify-between">
              <label className="text-sm text-[#94A3B8]">悬浮条显示</label>
              <select
                value={settings.trayDisplayMode}
                onChange={(e) => handleChange("trayDisplayMode", e.target.value)}
                className="px-2 py-1 text-sm bg-[#1E293B] border border-[#334155] rounded text-[#E2E8F0]"
              >
                <option value="token_mem">Token + 内存</option>
                <option value="cost_mem">费用 + 内存</option>
                <option value="token_only">仅 Token</option>
                <option value="cost_only">仅费用</option>
                <option value="mem_only">仅内存</option>
              </select>
            </div>

            <div className="flex items-center justify-between">
              <label className="text-sm text-[#94A3B8]">USD→CNY 汇率</label>
              <input
                type="number"
                step={0.1}
                value={settings.usdToCnyRate}
                onChange={(e) => handleChange("usdToCnyRate", parseFloat(e.target.value) || 7.2)}
                className="w-20 px-2 py-1 text-sm bg-[#1E293B] border border-[#334155] rounded text-[#E2E8F0] text-right"
              />
            </div>
          </div>
        </div>

        {/* 数据源路径覆盖 */}
        <div className="p-3 rounded-lg bg-[#111827] border border-[#1E293B]">
          <h2 className="text-sm font-semibold text-[#94A3B8] mb-3">数据源路径（留空使用默认）</h2>

          <div className="space-y-2">
            {([
              ["opencodeDbPath", "OpenCode DB"],
              ["ccswitchDbPath", "CC Switch DB"],
              ["workbuddyDirPath", "WorkBuddy 目录"],
              ["hermesDbPath", "Hermes DB"],
              ["zcodeDbPath", "ZCode DB"],
            ] as const).map(([key, label]) => (
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
          {saved ? "✓ 已保存" : "保存设置"}
        </button>
      </div>
    </div>
  );
}
