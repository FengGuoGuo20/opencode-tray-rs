# 参与贡献

🎉 感谢你对 OpenCodeTray-RS 的关注！欢迎以任何形式参与贡献——提交 Bug、建议功能、改进文档、提交代码。

## 🐛 提交 Issue

- **Bug 报告**：请使用 Bug 报告模板，附上复现步骤、系统环境、日志（如有）。
- **功能建议**：描述你希望的功能、使用场景以及期望的效果。
- 提交前请先搜索是否已有相关 Issue，避免重复。

## 🔧 开发流程

### 环境准备

参见 [README.md](./README.md#-快速开始) 的「前置要求」和「安装依赖」。

### 开发步骤

1. **Fork** 本仓库到你的 GitHub 账号
2. **克隆**到你本地：
   ```bash
   git clone https://github.com/<你的用户名>/opencode-tray-rs.git
   cd opencode-tray-rs
   ```
3. **创建功能分支**（不要直接在 `main` 上开发）：
   ```bash
   git checkout -b feature/your-feature-name
   ```
4. **安装依赖**：
   ```bash
   pnpm install
   ```
5. **启动开发模式**：
   ```bash
   pnpm tauri dev
   ```
6. **编码 → 测试 → 提交**
7. **推送并提交 Pull Request**

### 分支命名约定

| 前缀 | 用途 | 示例 |
|------|------|------|
| `feature/` | 新功能 | `feature/heatmap-zoom` |
| `fix/` | Bug 修复 | `fix/tray-icon-flicker` |
| `docs/` | 文档改进 | `docs/update-readme` |
| `refactor/` | 代码重构 | `refactor/db-helper` |

## 📝 提交规范

使用 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>(<scope>): <description>
```

| type | 说明 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `docs` | 文档变更 |
| `style` | 代码格式（不影响功能） |
| `refactor` | 重构（既不是新功能也不是修 Bug） |
| `perf` | 性能优化 |
| `test` | 测试相关 |
| `chore` | 构建/工具变更 |

**示例**：
```
feat(panel): add model distribution pie chart
fix(tray): resolve tooltip not updating on Windows
docs: add screenshot to README
```

## 🧪 提交 PR 前的检查清单

- [ ] 代码能通过编译：`pnpm tauri build` 成功
- [ ] 没有引入新的编译警告（Rust `cargo check`）
- [ ] 没有硬编码的密钥、Token、个人信息
- [ ] 提交信息符合提交规范
- [ ] 如果是 UI 变更，附上截图
- [ ] 更新了相关文档（如有必要）

## 🏗️ 项目架构概览

```
前端 (src/)  ←→  Tauri IPC  ←→  后端 (src-tauri/src/)
                                     │
                    ┌────────────────┼────────────────┐
                    │                │                │
               commands/        services/          db/
            (对外暴露命令)   (5大数据源适配器)   (SQLite 访问)
```

- **新增数据源**：在 `src-tauri/src/services/` 下新建模块，实现统一 trait
- **新增命令**：在 `src-tauri/src/commands/` 下注册，并在 `lib.rs` 中挂载
- **新增 UI 组件**：在 `src/components/` 下按模块归类

## 💬 交流

- GitHub Issues — Bug 报告 & 功能建议
- GitHub Discussions — 问题讨论 & 使用交流

## 📄 许可证

贡献的代码将遵循项目的 [MIT License](./LICENSE)。

---

再次感谢你的贡献！每一份 PR 都让这个项目更好 ❤️
