# Codex HUD Windows 平台验证预览

这是独立的本地开发目录，**不是已验收的 Windows 发行版**。作者 wasdTornado，沿用仓库 [MIT 许可](../LICENSE) 与原图标；现有 Swift 客户端、官网、版本和 Sparkle 配置保持原状。

## 当前路线与关卡

候选采用 Tauri 2 + Rust 后台 + 小型 Win32 适配。已比较 WPF：原生透明窗和托盘可以实现，但无法在现有 Mac 上预览 WPF，且桌面磨砂仍需要额外平台适配；本次只开发 Tauri 一套。**Windows CLI、WebView2 + 形状裁剪 + Acrylic、动态托盘等关卡尚未实测通过，因此不能称路线已验证可行。**

先阅读 [逐项功能对应](docs/FEATURE-PARITY.md)、[能力矩阵](docs/CAPABILITIES.md)、[实际验证记录](docs/VALIDATION.md)和 [Windows 验收清单](docs/WINDOWS-ACCEPTANCE.md)。所有测试层次独立记录。

后续已做一次 [有限静态复核](docs/STATIC-REVIEW.md)，修复源码中可确定的问题；这不替代 Rust 编译与 Windows 验收。

## 无工具安装的本地检查

```sh
python3 Windows/scripts/verify-local.py
python3 Windows/scripts/preview.py
```

预览仅绑定 `http://127.0.0.1:4173`，使用固定 82/47 演示值，不连接 CLI。浏览器检查使用 ego-browser。预览中的隐藏偏好只属于浏览器演示，不证明 Windows 托盘或宿主持久化有效。退出服务使用 Ctrl+C。

## 项目内 Rust 验证

```sh
sh Windows/scripts/cargo-local.sh test --locked --manifest-path Windows/Cargo.toml
sh Windows/scripts/cargo-local.sh check --locked --manifest-path Windows/src-tauri/Cargo.toml
sh Windows/scripts/cargo-local.sh test --locked --manifest-path Windows/src-tauri/Cargo.toml
```

用户已授权安装官方 Rust **1.99.1 stable minimal**，仅放在 `Windows/.tools/{rustup,cargo}`。脚本通过局部环境变量和绝对路径使用工具，不修改全局 PATH/shell，也不自动安装组件。已生成两份 Cargo.lock，后续使用 `--locked`。当前仅安装 `aarch64-apple-darwin` 目标；Mac 上核心 **16 项**、宿主绘制/偏好持久化/离线验签 **9 项**已通过，不能计为 Windows 构建。

核心测试包含额度归一化、状态、真实子进程 fake CLI 的超时/取消/噪音/服务器请求、Unix 孙进程清理。macOS 使用 `/usr/bin/python3`；Windows 设置 `HUD_TEST_PYTHON` 为已有 Python 的绝对路径。它们不访问真实账号。宿主测试使用 Tauri mock runtime 和官方公开公钥/签名向量，只调用验签，不联网、不安装。见 [第三方测试向量声明](THIRD_PARTY_NOTICES.md)。

Windows 专属无账号回归脚本与 7 项判定见 [执行说明](docs/WINDOWS-REGRESSION.md)。可在 Mac 额外使用 `--features test-fixtures` 测试共用 Rust 假服务；其通过不能替代 Windows 专属用例。

## Windows 本地构建与运行（未执行）

建议先以 Windows 11 x64 验证；Windows 10、Windows ARM64 不在已验证支持矩阵内。需要现有 Rust MSVC、Visual Studio C++ Build Tools、WebView2。脚本不安装工具、不改执行策略、不跑远端 CI：

```powershell
& .\Windows\scripts\verify-windows.ps1
.\Windows\src-tauri\target\debug\codex-hud-windows.exe
```

默认原生宿主也只显示标注的演示值。真实读取必须由用户在 Windows 手动选择：

```powershell
cargo run --manifest-path .\Windows\Cargo.toml --bin cli_probe -- --allow-real-cli
.\Windows\src-tauri\target\debug\codex-hud-windows.exe --real-cli
```

只发现绝对 PATH 目录内原生 `codex.exe`（优先）或 `codex.cmd`，不静默使用 WSL，不读取或复制凭据。`.cmd` 路径中含 `% ! & | < > ^ "` 或换行时拒绝执行，提示使用 `.exe`，避免命令解释器展开。空格和 Unicode 仍需 Windows 测试。探针省略真实额度和服务器原文。

## 实现边界

- Rust 后台负责 5 分钟刷新、去重、25 秒截止、取消和成功时间。隐藏 WebView 不负责调度。
- `.cmd`/`.exe` 在 Windows 先挂起创建，放入不可继承的 Job Object，设置 kill-on-close 后恢复线程；正常清理主动终止 Job 并限时确认活动进程归零。错误和超时单独报告。仍需验证孙进程、异常启动和退出时清理。
- 36×36 / 36×72 逻辑尺寸，5h 优先，沿外缘分区；未知显示“—”，零值显示“0”。详情列出实际返回的全部窗口。
- Windows 窗口区域使用 SetWindowRgn；没有把 HTTRANSPARENT 或整窗忽略鼠标当作形状外跨进程穿透的证明。
- 浅色请求 Acrylic，深色移除材质并使用不透明石墨。网页没有伪造桌面模糊，磨砂性能/点击不变黑仍是平台关卡。
- 隐藏意图在 `%LOCALAPPDATA%` 的独立应用目录中保存；demo/native 分开，仅保存偏好、不缓存真实额度。失败保留内存里的上次成功数据。
- 托盘菜单及数字绘制、WM_POWERBROADCAST 唤醒、左右停靠已写入候选源码，共享代码已在 Mac 编译，Windows 专属分支尚待 Windows 编译和实测。

## 更新边界

默认未配置 Windows 更新源，也未启用安装包分发。配置中的空公钥和空 endpoints 是明确关闭状态，使插件可以初始化，不是假公钥或可用发布源。`HUD_WINDOWS_UPDATE_URL`（HTTPS）及 `HUD_WINDOWS_UPDATE_PUBLIC_KEY` 为编译时配置，未设置就明确告知“尚未配置”，不会访问 macOS 更新地址。自动检查只更新托盘状态；用户手动检查并确认后才调用 Tauri 更新插件下载、验签、安装并请求安装后重启。已在 Mac 离线测试实际插件的签名验证，但下载、Windows 安装与重启仍须单独验收。当前没有生成密钥、安装包或有效更新。

更新签名不同于 Windows Authenticode 代码签名，不代表 SmartScreen 已消除。没有购买证书或服务的授权。
