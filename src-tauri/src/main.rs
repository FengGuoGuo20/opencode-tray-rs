// 防止 Windows 上运行时出现控制台窗口
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    opencode_tray_rs_lib::run()
}
