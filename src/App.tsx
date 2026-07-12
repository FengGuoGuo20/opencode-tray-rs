import { getCurrentWindow } from "@tauri-apps/api/window";

function App() {
  const openPanel = async () => {
    // 通过 Tauri command 打开主面板窗口
    await getCurrentWindow().emit("open-panel");
  };

  return (
    <div className="flex items-center justify-center h-screen w-screen bg-transparent select-none">
      <div
        className="flex items-center gap-1.5 px-2 py-0.5 cursor-pointer"
        onDoubleClick={openPanel}
      >
        <span className="text-[11px] font-bold text-[#E2E8F0] font-mono">
          🔵 <span id="token-text">0</span>
        </span>
        <span className="text-[11px] font-bold text-[#F59E0B] font-mono">·</span>
        <span className="text-[11px] font-bold text-[#E2E8F0] font-mono" id="mem-text">
          0%
        </span>
      </div>
    </div>
  );
}

export default App;
