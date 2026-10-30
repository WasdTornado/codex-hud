# 当前实际验证记录

日期：2026-10-30。基线：`18c8b34f200697c0cd7bfa7f8dfafdbcad486b8a`。全部改动在 Windows/，未上传、发布或运行远端 CI。前轮静态复核见 [历史记录](STATIC-REVIEW.md)，以下为获得安装授权后的最新状态。

## 工具链安装

用户已明确同意项目内官方 Rust stable minimal 安装。使用 `https://sh.rustup.rs` 和 `https://static.rust-lang.org`，先读取 stable 清单，再固定清单对应的 **Rust 1.99.1**。

- rustc：`1.99.1 (58cf8f9c3 2026-10-13)`；Cargo：`1.99.1 (5f94df478 2026-08-27)`。
- 安装目标仅 **aarch64-apple-darwin**，组件仅 cargo、rustc、rust-std。未安装 Windows target、MSVC、虚拟机或其他系统工具。
- RUSTUP_HOME：`Windows/.tools/rustup`；CARGO_HOME：`Windows/.tools/cargo`，均为项目内目录。
- 使用 `--no-modify-path --profile minimal`；安装前后 `.zshrc/.zprofile/.profile/.bash_profile/.bashrc` 的存在性及 SHA256 均一致。未修改全局 PATH，也没有 source 安装器生成的 env 文件。
- 官方安装脚本 SHA256：`9502b514362a21096210d498f143b8cdacda365d3dfcf45e8f5541e089ccafc0`。
- stable 清单 SHA256：`fb152be80975c194aa5fc684b2aec3a6e14e2b855efe2faa4e5f8ae6e590f463`。
- 初次 curl/Python HTTPS 请求发生 TLS 中断；保留证书校验并限定 TLS 1.2 后从同一官方地址成功下载。没有使用第三方镜像或关闭证书检查。随后 rustup 官方下载及安装成功。

工具链、Cargo 缓存、构建目录和生成 schema 均被 Windows/.gitignore 忽略；两份 Cargo.lock 保留作为源码依赖锁。

## 1. 本地编译 / 无账号模拟 / 网页预览

以下命令已实际执行，目标均为当前 **Mac ARM64**：

| 命令 | 实际结果 | 能证明什么 |
|---|---|---|
| `sh Windows/scripts/cargo-local.sh test --locked --manifest-path Windows/Cargo.toml` | **16/16**：7 单元 + 9 协议/Unix 进程树 | 归一化、未知/零、分桶、去重、失败保留；真实本地 fake 子进程协议、浮点 ID、噪音、通知、拒绝服务端请求、限长、EOF、超时、取消；Unix 孙进程停止 |
| `sh Windows/scripts/cargo-local.sh check --locked --manifest-path Windows/src-tauri/Cargo.toml` | 通过 | 共享宿主初始化、菜单/状态/更新接口 Rust 类型检查；不检查被 cfg(windows) 排除的分支 |
| `sh Windows/scripts/cargo-local.sh test --locked --manifest-path Windows/src-tauri/Cargo.toml` | **9/9**：3 托盘绘制 + 3 偏好文件 + 3 更新测试 | RGBA 数字舍入、5h 优先、透明角和未知/零/100 区分；偏好替换持久化、写入失败回滚、损坏文件保持隐藏；Tauri mock 配置启动、实际插件离线验签及篡改/版本不符拒绝 |
| `python3 Windows/scripts/verify-local.py` | 通过，含 Node **3/3** | JSON/默认隐藏/图标容器/JS 语法与 UI 几何/手势状态 |
| 原有 `python3 scripts/test.py`（/tmp 模块缓存） | Swift **23/23** | macOS 核心回归；不是 Windows 结果 |

上述基础命令共 **25 项 Rust 测试通过**；最新增加的共用原生夹具使累计达到 **26 项**，见下方专属回归准备记录。没有访问真实账号或打印真实额度。签名测试只有官方公开公钥和固定 `test` payload 的签名，使用 `Update::verify`；没有私钥、网络请求、安装或重启调用。

锁定依赖包括 Tauri **2.12.5**、dialog **2.8.4**、updater **2.14.1**、Tokio **1.54.1**。首次解析后已使用 `--locked` 复跑。rustfmt 不属于 minimal 安装，本轮没有额外安装或声称运行格式化工具。

前轮 ego-browser 交互验证记录：独立任务空间读取双周期 **36×72**、单周期 **36×36**；点击详情、全部演示窗口与重置时间、Esc、未知“—”、零“0”、失败保留和演示隐藏后重载。浅深截图只含演示值，位于 `/tmp/codex-hud-preview-light.png`、`/tmp/codex-hud-preview-dark.png`。预览服务和任务空间已关闭。

## 本轮编译/测试发现及修复

- 初次 Rust 单元 7/7、协议 5/6；EOF 用例失败。独立 Python 复现 macOS 在仅剩未回收僵尸进程的进程组上返回 EPERM。修复为回收直接子进程后，仅用信号 0 确认组消失；仍有组或查询失败则报告清理失败，不假定孙进程已退出。终止信号只发一次，避免回收后 Drop 再使用旧 PID。
- 上述修改首次触发 Rust unsafe 表达式比较的解析错误，已加括号修复，随后所有测试通过。
- 把宿主 builder 抽为可在 Mac 编译检查的共享函数；Mac main 仍只告知使用网页预览，不启动 Windows UI。非 Windows 原生形状/唤醒适配明确返回“不支持”，不以空实现返回成功。
- 读取已锁定 updater 源码发现 pubkey 是反序列化必填字段。加入明确空公钥/空 endpoints 的关闭配置，使插件可以初始化；未配置真实 Windows 源或密钥时 check 仍提前返回。启用签名版本绑定，并明确请求安装后重启（重启未实测）。
- 离线更新测试首次因测试 builder 空 endpoints 失败；补上不会使用的 `.invalid` 地址，同时保留任何网络请求即失败的守卫，之后 3/3 通过。

## 功能核对后的补齐与复验

当前逐项宿主接线、mock/stub 与未配置内容见 [功能对应矩阵](FEATURE-PARITY.md)。本次不是只做文档核对，已完成：

- 偏好更新集中到 `preferences.rs`，持锁写临时文件并替换，只有成功后才修改内存。隐藏、自动更新和位置共用该路径；修复自动更新开关写入失败不回滚的问题。新增 3 项实际本地文件系统测试。
- 移除固定详情高度，前端按内容报告高度，只有详情 WebView 可调用受限尺寸命令；本体页面不再渲染第二份详情。完整日期时间用于最近成功同步。宽度保持 192，浮窗仍为 36×36 / 36×72。
- 启动恢复位置先限制到工作区；Windows 显示器/工作区变化时校正位置而不 show；DPI 改变更新拖动阈值。Win32 分支仍未编译或实测。
- 自动检查在启用及启动时触发，之后每 6 小时检查；仍只通知、安装须手动确认。build.rs 监听更新源/公钥环境变量变化，避免配置改变却复用旧构建。
- `--locked --offline` 复跑核心测试 **16/16**、宿主测试 **9/9**、宿主 `cargo check` 均通过；仍仅目标 aarch64-apple-darwin。Node **3/3** 与 JS/JSON 静态检查通过。原有 Swift **23/23** 是前轮结果，本次未改 Swift、未重复运行。
- Ego 任务空间 28 本轮实测：双周期浮窗 36×72、详情 192×152；单周期浮窗 36×36、详情 192×114；Esc 收起通过。这里是网页布局，不是原生窗口尺寸证据。
- 在同一 Ego 任务空间注入明确的 mock Tauri bridge：带过期提示时，详情实测高 170 → 132，分别向 `resize_details` 发出相同高度；Esc 发送 `close_details` 并隐藏；主浮窗页 expanded=true 时详情仍隐藏且没有尺寸请求。只验证 JS 消息契约，未执行原生宿主。
- 本轮本地服务正常停止，Ego 空间正常 finish。没有未决审批、后台下载或构建。两份 Cargo.lock 未变化，未安装额外工具。

## Windows 专属回归准备（最终补充）

- 在原有核心测试工程添加 `test-fixtures` feature 和 `hud-fake-cli` 原生 Rust 假服务；正常宿主依赖不开启该 feature。假服务不联网、不读登录，使用工作目录中的固定 fixture 配置。
- 新增 `tests/windows_native.rs`，声明 7 项 Windows 专属测试。两类入口均走生产 `discover/read`，测试目录含空格/中文；进程树测试持有准确进程句柄再放行，成功/超时/取消/future 丢弃均检查停止；失败后的测试兜底终止不算通过。
- 现有 `verify-windows.ps1` 已扩充用例收集检查，必须实际收集 7 项再执行，缺 feature 或 0 tests 必须失败。运行后恢复临时 HUD_TEST_PYTHON 环境变量。
- 新增 `verify-hidden-intent.ps1` 人工观察记录器，只启动演示宿主；y/n/s 分别为 passed/failed/not_run，未完成不能通过，不覆盖已有记录、不终止已有 HUD。真实重启/主题/睡眠仍须操作者执行。
- 在 **Mac ARM64** 实际执行 `cargo-local.sh test --locked --offline --features test-fixtures --manifest-path Windows/Cargo.toml`：原有 7 单元 + 9 Python/Unix 协议，以及新增 1 个共用原生 fake CLI 边界测试，合计 **17/17** 通过。新增测试循环覆盖成功、错误、EOF、单行和累计限长、超时。宿主前轮 **9/9** 未改，本机累计已实际通过 **26 项 Rust**。
- Mac 输出 `windows_native: running 0 tests` 是 cfg 排除，**7 项 Windows 专属全部 not_run，不纳入 26 项**。Rust 只验证了非 Windows 代码；Win32 句柄测试分支没有类型检查/链接证据。
- 本机没有 pwsh，未安装。因此两份 PowerShell 只有人工结构复核、UTF-8 BOM/路径/声明数量静态核对；**没有声称 PowerShell 解析或运行通过**。
- 环境要求、一次性执行顺序、每项预期与不覆盖内容见 [WINDOWS-REGRESSION.md](WINDOWS-REGRESSION.md)。没有创建安装包、证书、更新源或发布配置，没有启动 Windows 或远端 CI。

## 2. Windows 构建

**未执行。** 未安装 Windows Rust target、Windows SDK/MSVC，也没有使用远端 CI。Mac 编译与链接的测试二进制不是 Windows EXE。没有生成 Windows 安装包。

Windows 专属代码（CLI cmd/exe 的实际启动、挂起/Job Object/线程恢复、HWND/HRGN、Shell subclass、注册表主题读取）未经过 Windows 编译器与实际运行验证。

## 3. 实际 Windows 验收

**未执行。** 无可交互 Windows 环境，未读取真实 Windows 登录。透明区域跨进程点击、Acrylic/深色、托盘和任务栏、DPI/多屏、睡眠唤醒、原生隐藏持久化、Windows 进程树回收、更新下载/安装/重启均待执行 [验收清单](WINDOWS-ACCEPTANCE.md)。

离线签名拒绝通过不能替代 Windows 安装流程拒绝，也不能替代 Authenticode 或 SmartScreen 验收。

## 既有权限与文件记录

- Library 交接原文 65 行已完整读取；此前 materialize 助手因 Python 3.9 不兼容失败，未裸下载绕过或假称落盘。
- 前轮连接中断后已先核对写入结果，没有重复未知变更。本轮核对后接续同一项目。
- 本机静态预览/ego-browser 的受限运行权限此前获准；本轮安装和 Cargo 获取公开依赖也在用户明确授权范围内完成，没有待用户处理的安装审批。
- 没有修改 Swift、官网、app.json、Sparkle 源/公钥或历史产物。没有 git commit/push、上传、发布、远端 CI、购买服务或安装虚拟机。

## 下一步

用户提供并授权 Windows 环境后，运行 `Windows/scripts/verify-windows.ps1`，先完成构建和无账号平台关卡，再做交互/真实 CLI 验收。当前阶段可复现本地 Rust 测试，不宣称 Windows 版已经完整对齐或可发布。
