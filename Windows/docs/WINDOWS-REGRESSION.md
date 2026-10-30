# Windows 无账号回归执行说明

状态：标准 Windows Server 2022 runner 已实际执行本套自动回归，**Windows 专属 7/7 通过、0 ignored**；详情见 [CI-STATUS.md](CI-STATUS.md)。hidden intent 人工交互记录仍未执行。Mac 编译或模拟不能替代 Windows 证据。脚本不安装工具、不调整执行策略、不触发 CI、不上传结果，不需要 Codex CLI 或账号。

## 最小环境

- 可交互 Windows 11 x64 桌面（本轮优先目标，不代表 Windows 10/ARM64 已受支持）。
- 已有 Rust/Cargo MSVC 工具链，建议与本地记录一致的 1.99.1；已有 Visual Studio C++ Build Tools 和 Windows SDK。锁文件保持不变，缺项先报告，不自动安装。
- 已有可运行的 Python 3（当前夹具在 3.9 验证）；`python` 必须解析到真实程序，不能是 Microsoft Store 占位入口。
- 已有 WebView2 Runtime，用于宿主运行；PowerShell 5.1 或 7。两份 ps1 保存为 UTF-8 BOM，兼容 PowerShell 5.1 中文读取。
- 首次获取锁定的公开 Cargo 依赖可能需要网络；测试服务本身不联网。已有完整缓存时可给手动 Cargo 命令加 `--offline`。不需要 Node、真实 Codex、证书或更新服务来执行本套无账号测试。

## 一次性执行顺序

1. 在本地 Windows 检出/复制项目后记录 OS build、CPU、Rust/Cargo、SDK、Python、WebView2 版本及源码 diff 摘要。不得把真实额度、令牌或凭据写入日志。
2. 在项目根目录运行 `& .\Windows\scripts\verify-windows.ps1`。脚本先确认 `windows_native` 实际收集到 **7 个**测试，再执行共用协议、Windows 专属、宿主文件持久化/托盘绘制/离线验签，最后构建宿主。任一步失败即停止。需要执行策略变更时停下报告，不绕过策略。
3. 检查各段结果：failed=0、ignored=0 且 Windows 专属确实执行 7 项，才能记录自动部分通过。Mac 上 `windows_native: running 0 tests`、feature 未开启、平台跳过或没有运行都属于 **not_run**，不计通过。
4. 执行 `& .\Windows\scripts\verify-hidden-intent.ps1`，按真实观察完成隐藏意图检查。脚本只启动未附加 `--real-cli` 的演示宿主；输入 y/n/s 分别记录 passed/failed/not_run。未实际睡眠或切换主题须填 s。默认记录在忽略目录 `.artifacts/windows-hidden-intent.json`，存在旧记录时需另指定 `-OutputPath`，不会覆盖。
5. 最后按 [WINDOWS-ACCEPTANCE.md](WINDOWS-ACCEPTANCE.md) 完成透明/磨砂、数字托盘、拖动、DPI、多屏与真实唤醒等平台关卡。全部关卡通过后再考虑真实账号验收；此回归脚本不会自动启动真实 CLI。

已有 HUD 进程须由用户自行退出；脚本不会结束用户已有进程。人工脚本中断时需要自行退出本轮演示宿主并恢复系统外观。测试仅操作演示偏好目录，不复制或删除用户登录数据。

## 自动 Windows 专属用例与判定

`tests/windows_native.rs` 使用 `test-fixtures` feature 构建的 Rust 假 CLI，不依赖真实 Codex。每个实际工作目录包含空格和中文。exe 模式复制为 codex.exe；cmd 模式通过真实 codex.cmd 包装调用同目录 fixture.exe，使用生产发现/启动路径和固定 app-server 参数。

| 测试名后缀（共同前缀 windows_） | 覆盖 | 通过判定 | 当前结果 |
|---|---|---|---|
| exe_cmd_success_noise_and_tree_cleanup | exe/cmd、空格/Unicode 路径、浮点与乱序 ID、字符串/布尔 ID、通知、噪声、服务器请求拒绝 | 固定 82/47 且排序正确；parent/child 两个已打开进程句柄均 signaled | CI 通过；交互未验收 |
| exe_cmd_timeout_tree_cleanup | exe/cmd 假服务挂起与孙进程 | 返回超时，且两个句柄确认退出；测试用 8 秒截止，不声称测了默认 25 秒 SLA | CI 通过；交互未验收 |
| exe_cmd_cancellation_tree_cleanup | 假服务与孙进程启动后取消 | 返回取消且两进程退出 | CI 通过；交互未验收 |
| exe_cmd_aborted_future_tree_cleanup | 丢弃读取 future，Job Drop 兜底 | JoinHandle 确认取消且两进程退出 | CI 通过；交互未验收 |
| exe_cmd_precancel_never_spawns | 请求开始前已有取消意图 | 返回取消、ready.json 未生成 | CI 通过；交互未验收 |
| exe_cmd_errors_and_output_limits | exe/cmd 错误、EOF、单行 1 MiB、累计 >4 MiB | 明确错误类别，且不泄漏固定 secret-token 文本；不得变成超时蒙混通过 | CI 通过；交互未验收 |
| cmd_metacharacters_are_rejected_without_execution | cmd 文件名 & / % / ! / ^ | 现有绝对路径也被拒绝，未运行假 CLI | CI 通过；交互未验收 |

进程树测试先等待 fake CLI 报告父/孙 PID，再打开进程对象句柄，最后放行协议，避免成功进程太快退出或 PID 重用造成误判。只查询这些测试句柄。断言失败后的 TerminateProcess 是测试环境清理，不算产品清理通过；不使用进程名批量终止、不杀真实 Codex。测试异常时 ReadTask Drop 取消读取。若 Job 绑定失败，测试必须失败，不能跳过。

共用 `native_fixture_protocol_and_boundaries` 实际启动同一个 Rust 假服务，验证成功/错误/EOF/单行/累计限制/超时。它在 Mac 已通过，但没有经过 cmd 或 Windows Job。原有 Python 协议套件继续保留，没有另起第二套生产客户端。

## hidden intent 的自动与交互边界

- 自动：宿主 `preferences` 的三项文件系统测试在 Windows 重新执行，检查隐藏/开关/位置保存替换、写入失败不修改内存、坏文件或读取失败默认隐藏。核心状态测试检查刷新去重和失败保留。这些不证明可见窗口行为。
- 人工脚本七步：隐藏 → 隐藏时手动刷新 → 隐藏等待至少六分钟 → 退出重启 → 实际主题切换并恢复 → 实际睡眠唤醒 → 用户主动显示、详情/Esc、隐藏退出。
- 人工结果只表示操作者确认，没有自动窗口枚举或图像判定。等待六分钟不单独证明刷新成功；后台同步仍需对应时间/事件证据。部分未执行时脚本以不完整状态结束，不输出“全部通过”。

## 尚不覆盖

本套件不证明真实账号读取、跨进程透明命中、磨砂性能、任务栏溢出区、Explorer 重启、所有 DPI/多屏组合、Job 嵌套/系统策略绑定失败、OS 创建挂起阶段故障注入、安装升级与 SmartScreen。失败情况需按实际错误补充，不能在 Mac 上伪造平台通过。无 Windows 环境时停留在“准备完成、未执行”，不继续扩大安装或发布范围。
