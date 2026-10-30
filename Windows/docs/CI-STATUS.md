# Windows CI 实际状态（2026-10-30）

本轮用户明确授权向公开 WasdTornado/codex-hud 的独立分支上传 Windows 源码、无账号测试和最小 CI。未授权 main、Release、Pages/macOS/Sparkle、付费 runner 或凭据上传。

## 已完成

- 分支 windows-ci-20261030，从公开 main 7b6320246829850f0a3ea0a8629ea401cd0d2efe 建立。用户本地 main 18c8b34f200697c0cd7bfa7f8dfafdbcad486b8a 未改。
- 本地 Git 无可用 HTTPS 身份；改用已连接 GitHub 插件。首个远端提交 65b30b5b0daf4b282b516aff884519c0717f958a，源码树与本地审查树一致。后续修复均在同一分支，最终通过 CI 的代码提交为 c6702ec84148f96faf575c984f7c46324c15ea2c。
- .tools、target、.artifacts、真实账号、私钥和 token 未上传。只包含 Windows/ 与 .github/workflows/windows-ci.yml；现有公开品牌图片复用原 blob。
- 标准 windows-2022 runner，contents:read，分支专用 push 触发，无发布、PR、RDP。公开仓库标准 runner 计算免费；未确认 artifact 存储额度，因此未上传 artifact、未启用缓存 action。
- 实际 Windows Server 2022 10.0.20348，Rust 1.99.1，x86_64-pc-windows-msvc，Python 3.12.10。

## 修复与验证层次

- cmd 包装器失败定位：固定 echo 同样报路径语法错误；解释器路径原为混合分隔符 System32/cmd.exe，改为逐段 PathBuf::join 后，第四轮及后续 Windows 专属 7/7 通过。
- 更新测试启动失败：STATUS_ENTRYPOINT_NOT_FOUND，发生在进入 Rust 测试断言前。Tauri 的对话框要求 Common Controls v6；应用 manifest 不自动传递到独立集成测试 EXE。参照锁定 Tauri 源码 embed_manifest_for_tests，为测试目标加入单独 manifest，保持实际签名测试不变。
- 第七轮：https://github.com/WasdTornado/codex-hud/actions/runs/41034530956 ，测试提交 c6702ec84148f96faf575c984f7c46324c15ea2c。**最终结果 success**。Windows 测试 **29/29、0 failed、0 ignored**：核心 6 + 原生 fake CLI 1 + Python 协议 6 + Windows 专属 7 + 托盘/偏好 6 + 离线更新 3。实际收集 Windows 专属 7 项。随后 cargo build 宿主成功；demo 进程运行 10 秒未退出的冒烟通过。条件性的失败诊断步骤被跳过，不是测试用例跳过。
- 两个修复均获 CI 回归证据：exe/cmd 路径和进程树 7 项通过；manifest 修复后实际插件离线验签 3 项通过，未降低断言或移除测试。
- 已检查 workflow artifacts 列表为空。构建得到 runner 内 Windows/src-tauri/target/debug/codex-hud-windows.exe，没有可下载 artifact、安装包或正式发布。
- Mac 平台无关核心 17/17、宿主 9/9 已复跑通过；Windows 专属用例在 Mac 的 0 tests 不计通过。

## 仍未验收

CI 使用 Windows Server 2022 托管虚拟机，不等同可交互 Windows 11 实机。透明区域跨进程命中、桌面磨砂/黑灰闪烁、任务栏与溢出区、点击/拖动与焦点、DPI/多屏、隐藏重启与真实睡眠唤醒、真实 Windows/WSL 登录隔离、更新下载/安装/重启均未验收。无需真实账号的测试不产生真实额度日志。

不生成或上传分发 artifact，不创建 Release/PR，不改 main、Pages、macOS 或 Sparkle。构建产物仅留在临时 runner；日志属于无账号测试证据。

## 权限与本地同步

早期 create_tree 曾被自动审批拒绝；用户随后明确授权同一公开分支持续修复与 CI 重跑，后续写入已成功。当前不存在待用户补充的上传权限。未创建新凭据、长期规则或扩大安全权限。

Git HTTPS 缺少可用登录，提交通过已连接 GitHub 插件完成；只读 fetch 用于同步，未将任何凭据写入项目。本地 main 保留原值。恢复时核对分支与源码 SHA，避免重复推送。
