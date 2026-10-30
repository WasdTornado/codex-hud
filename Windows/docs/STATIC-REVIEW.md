# 有限静态复核记录

这是安装工具链之前的历史记录；后续编译与修复结果以 [当前验证记录](VALIDATION.md) 为准。

日期：2026-10-30。范围仅限前轮 Windows 候选源码的确定问题，不扩功能、不安装工具。开始时 `git status --short` 只有 `?? Windows/`，已跟踪文件无 diff；读取的源码与前轮记录一致。没有新启动预览服务、Rust 安装或构建。复核前文本快照保存在忽略目录 `.artifacts/static-review-before/`。

## 已修复的确定问题

1. **Windows API 模块导入错误。** 按 Microsoft `windows-rs` 的 `0.59.0` 标签源码核对：`SetWindowRgn` 在 `Graphics::Gdi`，`SetWindowSubclass/DefSubclassProc/RemoveWindowSubclass` 在 `UI::Shell`。修正导入并启用 `Win32_UI_Shell` feature。这是静态定位的编译阻断，尚未经过编译器验证其他问题。
2. **提前返回时的资源所有权。** 区域 HRGN 和 subclass 的 Box 上下文改为先取得 HWND 后分配，避免 HWND 读取失败漏释放。区域设置失败仍释放；成功交给系统。subclass 注册失败释放上下文，窗口销毁时移除回调并释放。
3. **响应 ID 与 Swift 不一致。** Swift 接受 JSON 数字 `1.0/2.0`，原 Rust `as_u64()` 会忽略。改为数字比较，同时仍拒绝布尔值、字符串和非匹配数值。补充 float-id 和无效 id 固定样例。
4. **`.cmd` 只在部分调用入口校验。** 实际构造 Command 时也执行校验，仅允许固定 `app-server --listen stdio://` 参数；使用 `/d /v:off /s /c`，拒绝解释器元字符、控制字符和有损 Unicode 路径转换；解释器必须是现有绝对路径。空格/中文/括号路径的双层引号表达已有单测源码，尚未在 Windows 执行。
5. **清理失败被吞掉与错误路径无上限。** 将 `wait()` 的内部错误与超时分开报告；Job 绑定失败后的直接子进程回收也有 2 秒上限。正常 Windows 路径先 TerminateJobObject，再限时检查 ActiveProcesses 为 0，最后回收直接子进程；异常仍由 kill-on-close 兜底，不能把兜底视作成功证据。Unix fake CLI 使用 RAII 进程组清理，避免 future 丢弃时只杀直接子进程；成功发送终止后解除守卫，避免回收后重复使用 PID。
6. **取消状态和舍入。** 退出使用 watch `send_replace(true)` 保存取消意图，之后拒绝开始新的刷新；即使当时没有订阅者，也不丢失取消。托盘与菜单数字先 `.round()` 再格式化，避免 Rust 半偶舍入与 Swift/JS 半值向上不一致。状态返回用局部快照明确结束锁借用。

## Swift 语义与有意差异

- `rateLimitsByLimitId` 中存在对象桶时只取 `codex`；仅其他产品有效时返回空，不回退冒用 `rateLimits`。没有有效桶表时才走旧字段。
- `primary/secondary` 只接受对象；布尔/字符串不当数值；分钟必须正整数且小于 Swift Int 上界；未知保留 null。剩余额度限制 0..100，重置时间保持接口秒值；排序稳定，300 分钟、10080 分钟优先。
- 失败保留 windows 和 updatedAt；刷新去重、错误信息固定化，不向 UI 暴露服务端原文。此处是源码对照，不是 Rust 实测通过。
- Windows 候选额外限制 **单行小于 1 MiB、累计不超过 4 MiB**；Swift 只有未处理缓冲上限。候选只处理当前阶段预期响应，忽略乱序/重复响应；不改变初始化后再发送读取请求的契约。
- 25 秒预算现在在准备启动前计时；同步文件系统/操作系统启动调用不能被 Tokio 抢占，截止在恢复异步轮询后生效。清理另有上限：Windows Job 最多 2 秒，直接子进程回收最多 2 秒。这不是“整个系统调用无条件 25 秒内返回”的承诺。

## 模拟和未接通部分

- 网页和未传 `--real-cli` 的原生模式只使用固定 82/47，明确标记演示；不能验证 Windows 登录。
- `fake_cli.py` 的 `child` 模式是进程树验收夹具，**当前没有已执行的孙进程回收测试**。
- Rust 新增的路径、浮点 ID、累计输出上限和归一化边界测试均为待执行源码。当前没有 Rust 编译器，不能以 Node/Swift 结果替代。
- updater 仍为默认关闭的接口：没有 Windows 更新源、公钥、签名 fixture、安装包或安装重启证据。

## 本轮实际执行

- `python3 Windows/scripts/verify-local.py`：通过 JSON/启动隐藏配置、ICO 容器、JS 语法及 Node **3/3**。该脚本不解析或编译 Rust。
- 使用 `/tmp` 模块缓存执行 `python3 scripts/test.py`：原有 Swift **23/23**。
- 没有重新进行网页交互检查：本轮没有修改 UI。前轮 Ego 结果仍只归入网页演示层。
- Windows 构建、Rust 测试、真实 Windows CLI、Job/孙进程、原生窗口/托盘/DPI/唤醒、更新验签安装均未执行。

## 官方来源

- [GDI 0.59.0 源码](https://raw.githubusercontent.com/microsoft/windows-rs/0.59.0/crates/libs/sys/src/Windows/Win32/Graphics/Gdi/mod.rs)
- [Shell 0.59.0 源码](https://raw.githubusercontent.com/microsoft/windows-rs/0.59.0/crates/libs/sys/src/Windows/Win32/UI/Shell/mod.rs)
- [JobObjects 0.59.0 源码](https://raw.githubusercontent.com/microsoft/windows-rs/0.59.0/crates/libs/sys/src/Windows/Win32/System/JobObjects/mod.rs)
- [Tokio Command](https://docs.rs/tokio/latest/tokio/process/struct.Command.html)

Rust 仍未获安装许可，未安装任何新工具或依赖。没有上传、发布、远端 CI；没有更改 macOS、官网、Sparkle 或历史产物。
