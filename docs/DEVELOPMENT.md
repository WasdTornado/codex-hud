# 开发说明

## 环境

Apple Silicon、macOS 26+、Xcode（`/Applications/Xcode.app`）和 Python 3。构建目标为 arm64，使用 AppKit 原生窗口与材质。Sparkle 下载由 `scripts/fetch_sparkle.py` 固定版本及 SHA256，第三方二进制不提交到 Git。

## 结构

| 路径 | 职责 |
| --- | --- |
| `Sources/QuotaOrb.swift` | 浮窗、材质、交互、动画与菜单 |
| `Sources/MenuBarStatus.swift` | 菜单栏数字环与状态 |
| `Sources/AppUpdater.swift` | Sparkle 检查与更新提示 |
| `Sources/QuotaModels.swift` | 额度窗口及快照 |
| `Sources/QuotaClient.swift` | CLI 发现、协议、超时及窗口归一化 |
| `Sources/QuotaStore.swift` | 请求去重、刷新与失败保留 |
| `Tests/` | 不访问真实账号的协议和状态测试 |
| `Assets/HUDIcon.icon` | 构建所需的原生图标工程 |
| `app.json` | 应用身份、版本和作者 |
| `updates.json` | 公开更新地址、公钥及钥匙串账户名 |

## 常用命令

```sh
python3 scripts/test.py
python3 scripts/build.py
python3 scripts/build.py --preview
python3 scripts/package.py
```

正式构建输出到 `build/`，DMG 输出到 `dist/`。预览输出到当前用户缓存目录，并使用独立身份；预览不连接真实账号和正式更新源。安装正式版前先退出旧应用，再运行 `python3 scripts/build.py --install`。

## 验证

1.1.3 增加菜单项“显示桌面悬浮窗”，使用 `desktopHUDHidden` 偏好保存选择。隐藏时收起详情并停止动画，菜单栏与额度刷新继续工作；详情与停靠操作暂时禁用。恢复时保留窗口位置；重建布局、外观变化、唤醒或重启不得擅自重新显示。

核心测试验证额度归一化、请求超时、取消、刷新间隔与失败状态。它不证明原生材质、拖动或动画表现。界面修改需要另外检查单周期、双周期、深浅模式、点击展开、Esc 收起及拖动。跨机器安装、Gatekeeper 和 Apple 公证也应单独验收。

`python3 scripts/test_sparkle_update.py` 使用随机测试身份和 localhost 更新源，更新安装需在界面操作；`--tamper` 测试篡改包拒绝。该测试需维护者钥匙串签名私钥，普通贡献者运行核心测试和构建即可。

## 贡献

代码注释使用中文。提交只包含本次相关改动。禁止提交账号凭据、更新签名私钥、本机额度、日志、缓存、历史安装包或机器路径截图。正式图标工程属于源码构建资源，需要随代码提交。

自行分发分支时请配置自己的应用身份、更新源和签名密钥；仓库中的公钥不提供维护者私钥。
