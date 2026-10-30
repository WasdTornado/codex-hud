# Windows 能力矩阵

2026-10-30（已补充 Rust 编译阶段），本机 macOS，仓库基线 `18c8b34f200697c0cd7bfa7f8dfafdbcad486b8a`。仅本地开发，无远端 CI、上传、发布、虚拟机安装。

“已有源码”与“能力通过”分开：下表的 Windows 项目全部尚未取得运行证据。Tauri 2 是候选实现，不是可行性已经通过的结论。当前已完成 Mac ARM64 的 Rust 核心编译及宿主共享代码检查；Windows 专属分支没有编译。

| 能力 | 候选实现/文档依据 | 本地证据 | Windows 构建 | 实际 Windows 验收 |
|---|---|---|---|---|
| CLI 原生 exe/cmd 发现 | PATH 绝对目录，exe 优先；固定 cmd 参数、危险字符拒绝 | Mac 路径/引号拒绝单测通过；未执行 cmd | 未执行 | 未执行；native/WSL 必须分开 |
| JSONL 初始化和额度读取 | initialize → initialized → account/rateLimits/read | Swift 23 项通过；Rust fake CLI 9 项通过（含 Unix 进程树） | 未执行 | 未执行 |
| 超时/取消/进程树 | 25 秒总截止；Windows 挂起 → Job Object → 恢复主线程；kill-on-close | Mac 超时/取消/future 丢弃/孙进程停止测试通过；Windows Job 未测 | 未执行 | 必测包装器/孙进程/取消/启动失败 |
| 额度归一化 | 完全按 Swift 分桶、primary/secondary、时长优先规则移植 | Rust 归一化与边界测试通过 | 未执行 | 真实账号未验证 |
| 透明置顶形状 | Tauri transparent/alwaysOnTop/shadow=false + SetWindowRgn | 仅 API 文档与候选源码 | 未执行 | WebView2、裁剪边缘、跨进程点击待测 |
| 36×36 / 36×72 | CSS 逻辑尺寸；宿主 LogicalSize，区域按 DPI 换算 | Ego DOM 实测两种尺寸 | 未执行 | 100/125/150/200% DPI 待测 |
| 外缘分区/未知/零 | 同一 UI 模块用于原生和预览 | Node 3 项通过；Ego 读到未知“—”、零“0” | 未执行 | 托盘和实际 WebView2 待测 |
| 点击/拖动/Esc | 手势阈值；原生 start_dragging；捕获丢失取消 | Node 手势状态测试、Ego 点击与 Esc 通过 | 未执行 | 原生指针捕获、拖动尾随点击待测 |
| 浅色磨砂/深色石墨 | 浅色 Acrylic、深色 clear effects + 不透明背景 | Ego 明暗截图，仅网页色彩证据 | 未执行 | 不变黑、性能、透明区域待测 |
| 动态数字托盘 | RGBA 圆环+数字，按 SystemUsesLightTheme 选颜色 | Rust RGBA 绘制 3 项通过；不证明真实托盘 | 未执行 | 0/99/100/未知、overflow、高 DPI 待测 |
| 隐藏持久化 | 宿主偏好先保存再显隐；demo/native 独立 | 3 项本地宿主偏好文件测试通过；Windows 重启保持隐藏未测 | 未执行 | 重启/唤醒/主题切换不得显示 |
| 刷新与失败保留 | Rust Tokio 后台；前端只订阅；成功时间单独保存 | Swift/Rust 状态测试、Ego 演示失败保留通过 | 未执行 | 真实隐藏刷新和登录失效待测 |
| 系统唤醒 | WM_POWERBROADCAST / PBT_APMRESUMEAUTOMATIC | 源码；不使用 tauri://resumed | 未执行 | 睡眠、休眠、唤醒风暴待测 |
| 更新签名/安装重启 | Tauri updater 独立 HTTPS 配置，默认关闭；用户确认后安装 | Tauri mock 离线验签 3 项通过（公开向量）；没有安装测试 | 未执行 | 篡改拒绝、旧版升级与重启未测 |

完整的宿主调用链、模拟替身和未实现交付项见 [逐项功能对应](FEATURE-PARITY.md)。

## 路线比较

Tauri 2 可将同一界面用于 Mac 静态预览，Rust 能独立持有子进程和刷新任务；代价是透明 WebView2 与 Win32 区域/材质组合有平台风险，需要先验收。WPF 有原生透明非矩形窗口和成熟托盘接入，但在当前 Mac 不可运行 WPF UI，磨砂也不能直接依赖旧 DWM Blur API（Windows 8 起该 API 不再产生预期效果）。此次不并行开发两套。

本轮不以 CSS backdrop-filter 模拟“桌面磨砂通过”，不以 setIgnoreCursorEvents 或 HTTRANSPARENT 声称跨进程点击穿透，不以 JS 隐藏页定时器声称后台刷新可靠。若原生关卡失败，先记录失败与版本环境，再决定是否调整路线。

## 官方依据（仅文档/源码，不是测试结果）

- [Codex App Server](https://learn.chatgpt.com/docs/app-server)：逐行协议、初始化与 account/rateLimits/read；实际兼容性仍以 Windows CLI 版本为准。
- [Rust Command](https://doc.rust-lang.org/std/process/struct.Command.html)、[npm bin](https://docs.npmjs.com/cli/v11/configuring-npm/package-json/#bin)：exe 与 cmd 包装的区别。
- [Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects)、[Process Creation Flags](https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags)：进程树、挂起创建与清理。
- [Tauri WindowConfig](https://v2.tauri.app/reference/config/#windowconfig)、[backgroundThrottling](https://v2.tauri.app/reference/config/#backgroundthrottling)：尺寸、透明、Windows 后台限速限制。
- [SetWindowRgn](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowrgn)：窗口区域、DPI 坐标与 HRGN 归属。成功后区域由系统持有。
- [WM_NCHITTEST](https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-nchittest)：HTTRANSPARENT 不能作为跨进程穿透证明。
- [GetSystemMetrics](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getsystemmetrics)：拖动阈值。
- [Tauri Tray](https://v2.tauri.app/reference/javascript/api/namespacetray/#settitle)：Windows 不支持托盘标题，应改绘制图标。
- [Window effects](https://docs.rs/tauri/latest/tauri/window/enum.Effect.html)、[window-vibrancy](https://github.com/tauri-apps/window-vibrancy)：Acrylic/Blur 的系统版本与拖动性能限制。
- [WM_POWERBROADCAST](https://learn.microsoft.com/en-us/windows/win32/power/wm-powerbroadcast)：Windows 唤醒消息。
- [WPF AllowsTransparency](https://learn.microsoft.com/en-us/dotnet/api/system.windows.window.allowstransparency)、[DWM Blur](https://learn.microsoft.com/en-us/windows/win32/dwm/blur-ovw)：替代路线与磨砂限制。
- [Tauri updater](https://v2.tauri.app/plugin/updater/)、[Windows code signing](https://v2.tauri.app/distribute/sign/windows/)：更新验签与系统签名的区别。
- [Windows prerequisites](https://v2.tauri.app/start/prerequisites/#windows)、[Cross compilation caveats](https://v2.tauri.app/distribute/windows-installer/#build-windows-apps-on-linux-and-macos)：Mac 检查/交叉产物不等同 Windows 构建环境和实际交互验收。
