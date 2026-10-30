# 交接功能对应核对

最新结果（2026-10-30）：[Windows CI 41034530956](https://github.com/WasdTornado/codex-hud/actions/runs/41034530956) **成功**，代码提交 `c6702ec84148f96faf575c984f7c46324c15ea2c`。`x86_64-pc-windows-msvc` 宿主构建、29/29 无账号测试（Windows 专属 7/7、0 ignored）和 10 秒演示进程冒烟通过。真实交互与账号验收未执行。完整层次及历史失败修复见 [CI-STATUS.md](CI-STATUS.md)。

以下保留 CI 前本地阶段的历史核对，涉及“Windows 未执行/未编译”的旧描述以以上结果为准；交互、磨砂、托盘、DPI、真实睡眠和安装验收仍未通过。

2026-10-30，当前最终源码。这里的“已实现”仅指存在真实宿主调用链，**不代表已通过 Windows 编译或交互验收**。Tauri + Win32 仍为待平台关卡验证的候选路线。验证层次与命令见 [VALIDATION.md](VALIDATION.md)。

## 已实现、待 Windows 验证

| 交接功能 | 当前真实调用链 | 已取得的证据 / 尚缺的证据 |
|---|---|---|
| 原生 CLI / 登录隔离 | `main::refresh` → `client::discover/read`；仅显式 `--real-cli` 启用；exe 优先、受控 cmd、无 WSL 回退 | Mac fake CLI / 路径字符串测试；Windows exe/cmd、原生登录与 WSL 隔离未测 |
| 协议与归一化 | initialize → initialized → account/rateLimits/read；按 Swift 选择 codex 桶、primary/secondary；未知保留 null | Mac Rust 与原有 Swift 测试通过；真实 Windows CLI 兼容性未测 |
| 超时、取消、限长、清理 | 25 秒协议截止、单行 <1 MiB / 总量 ≤4 MiB；watch 取消；退出先取消；Windows Job 挂起绑定/恢复/终止/查询活动数，Drop 关闭句柄兜底 | Mac 超时、预取消、中途取消、future 丢弃、Unix 孙进程停止通过；Windows Job 分支未编译。同步 OS 调用不可抢占，清理另有最多 2+2 秒预算，不能承诺所有返回严格 ≤25 秒 |
| 启动 / 5 分钟 / 手动 / 唤醒刷新与去重 | Rust Tokio 后台调度；托盘/详情调用同一 refresh；Windows WM_POWERBROADCAST 触发同一路径；Snapshot::begin 拒绝并发重复请求 | 去重/失败状态单测通过；真实隐藏后台、Windows 睡眠唤醒未测；不依赖 WebView 计时 |
| 失败保留与恢复 | Snapshot::finish 仅成功覆盖额度和成功时间；失败保留数据并显示异常，成功清除额度错误 | 单元、fake CLI、网页演示均有证据；真实登录失效/网络恢复未测 |
| 单周期球 / 双周期胶囊与设计 | Rust 按窗口数量设置 36×36 / 36×72；同一 UI 绘制外缘和数字；5h 上、周下，无百分号、无中间横线 | Ego DOM、Node 几何通过；Windows WebView2、DPI 与真实外观未测 |
| 详情全部窗口 / 重置 / 最近成功时间 / 刷新 | UI 使用宿主快照生成所有行、完整本地日期时间；详情按钮 invoke refresh；ResizeObserver → resize_details，宿主固定宽 192、按内容高度调整 | 网页尺寸通过；mock bridge 收到真实 UI 发出的高度请求；真实宿主窗口尺寸/区域未测。浮窗本体不扩宽 |
| 点击展开、收起、Esc、动画 | pointer 状态机 → toggle_details / start_drag；Esc → close_details；宿主 show/focus，延迟 hide 配合 CSS 动画；支持减少动态效果 | Node 与 Ego 点击/Esc 通过；原生焦点、动画裁剪、捕获丢失、触摸未测 |
| 拖动、左右停靠、多屏位置 | start_dragging；clamp_and_dock 使用工作区；保存位置，启动先夹回工作区；WM_DISPLAYCHANGE / WM_SETTINGCHANGE 校正位置；DPI 事件更新形状与拖动阈值 | 共享宿主 Mac 类型检查；Windows 拖动、迁屏、拔屏、系统阈值和工作区消息未测 |
| 透明、置顶、无外框 / 阴影 | Tauri 窗口配置 + SetWindowRgn，尺寸/DPI 变化重建区域 | 候选源码及官方 API 依据；透明区域跨进程点击、HRGN、WebView2 组合均未验证 |
| 浅色磨砂 / 深色石墨 / 跟随系统 | ThemeChanged → appearance + 快照；浅色 Acrylic，深色移除效果并用不透明石墨；托盘单独读取系统任务栏外观 | 网页颜色预览；桌面磨砂、无黑灰闪烁、系统切换未测，不把 CSS 当桌面磨砂 |
| 动态托盘数字 | tray::image 生成 RGBA → tray.set_icon；5h 优先，数字在环内、未知与零分开，错误/更新标识 | 3 项绘制测试通过；Windows 任务栏、溢出区、DPI、Explorer 重启未测 |
| 全部托盘菜单 | refresh / visible / details / left / right / update / automatic / about / quit 都连接宿主函数；隐藏时禁用详情和停靠但仍可刷新和查看状态 | 宿主编译检查；实际 Windows 菜单触发、系统托盘生命周期未测 |
| 隐藏持久化 | Preferences::update 先成功写盘才提交状态；demo/native 分目录；坏文件默认隐藏；刷新/主题/唤醒/工作区改变不调用 show | 3 项实际本地文件系统测试通过；浏览器持久化是另一个演示实现，不能替代 Windows 重启/唤醒验收 |
| Windows 更新检查、确认、安装与重启调用 | 手动菜单/命令 → schedule_update → updates::check → updater.check；发现版本通知托盘；手动确认后 download_and_install，restart_after_install(true) | **真实调用代码已接通，不只有离线验签 stub**；但源/公钥未配置，下载/安装/重启从未执行 |
| 自动更新开关 | 偏好事务持久化；启用后立即检查，启动时检查，后台每 6 小时检查；只通知，不自动安装；检查并发去重 | 保存/回滚测试与宿主编译；调度到真实更新源和用户取消安装未测 |
| 品牌、作者、许可与平台隔离 | 原图标、wasdTornado、MIT、公开测试向量声明；独立 Windows/ 和独立更新配置 | 本地文件检查；现有 Swift、官网、app.json、Sparkle 未改 |

## 仅模拟、测试替身或明确不支持

- 网页预览及未传 `--real-cli` 的宿主使用固定 82 / 47，标注“演示”。它们不能证明真实额度读取。
- fake CLI 用固定 JSONL 验证协议；浏览器 mock `__TAURI__` 仅验证消息契约（高度与 Esc），不执行 Rust / Win32。
- 更新测试的 Tauri mock runtime 使用官方公开公钥和固定 `test` payload。实际调用插件验签，拒绝篡改与版本不符；网络请求设为立即失败，**没有执行下载、安装或重启**。
- Mac main 明确拒绝运行 Windows 原生 UI；非 Windows shape / install_wake 明确返回不支持。非 Windows 拖动阈值和托盘外观常量仅用于编译，不作为 Windows 能力证据。

## 未实现 / 未配置的交付项

- Windows 更新服务地址、发布公钥、签名包和版本清单均未配置/生成。构建脚本已监听两个配置环境变量变化，但没有可实际安装的更新。
- Windows 安装包分发保持 `bundle.active=false`；没有安装器配置验收、安装包产物、Authenticode 签名或 SmartScreen 结果。没有触碰 macOS 的更新源/密钥。
- Windows 专属无账号套件已准备：[执行顺序与判定](WINDOWS-REGRESSION.md)，含 7 个 exe/cmd、空格/中文路径、超时/取消/孙进程、噪声与限长用例，以及 hidden intent 人工记录脚本。**全部 Windows 专属用例未执行**；Mac 收集为 0 tests 不计通过。Job 嵌套/绑定失败和 OS 挂起阶段故障注入、跨进程透明命中仍未自动化覆盖。
- 本地首轮 Windows 编译修复、平台关卡、真实账号与安装升级验收均未执行；不因此把功能矩阵标成“全面对齐”或“稳定版”。

## 真正需要的下一步

提供并授权可交互 Windows 环境，优先已有 Rust MSVC、C++ Build Tools、WebView2、Python 的 Windows 11 x64。先按 [无账号回归说明](WINDOWS-REGRESSION.md) 运行现有脚本，构建并执行已经准备的 Windows 专属用例，再完成 hidden intent 人工记录与其他平台关卡，最后由用户在原生 Windows 登录 CLI 后验收真实读取。环境若缺工具，先报告具体缺项，不自动安装。

现在无需再批准 Mac Rust，也无需购买证书、开通更新服务或上传文件。更新分发配置应在 Windows 平台关卡通过后另行确定；当前禁止发布的边界继续有效。
