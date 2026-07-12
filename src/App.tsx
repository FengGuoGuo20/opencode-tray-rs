import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import FloatingBar from "./components/floating-bar/FloatingBar";
import Panel from "./components/panel/Panel";
import Settings from "./components/settings/Settings";

type View = "floating-bar" | "main" | "settings";

function App() {
  const [view, setView] = useState<View>("floating-bar");

  useEffect(() => {
    // 根据当前窗口标签决定渲染哪个组件
    const currentWindow = getCurrentWindow();
    const label = currentWindow.label;
    if (label === "main") {
      setView("main");
    } else if (label === "settings") {
      setView("settings");
    } else {
      setView("floating-bar");
    }
  }, []);

  switch (view) {
    case "main":
      return <Panel />;
    case "settings":
      return <Settings />;
    default:
      return <FloatingBar />;
  }
}

export default App;
