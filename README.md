# Codex HUD

作者：**wasdTornado** · 原生 macOS Codex 额度悬浮工具 · MIT License

[下载最新版](https://github.com/WasdTornado/codex-hud-releases/releases/latest) · [开发说明](docs/DEVELOPMENT.md) · [发布流程](docs/RELEASING.md)

Codex HUD 将剩余额度放在桌面和菜单栏，支持每 5 小时与每周额度。单周期显示小球，双周期显示竖版胶囊，5h 在上、周在下；点击展开详情。

## 安装

1. 在 Releases 下载 `Codex-HUD-<版本>-arm64.dmg`。
2. 将 Codex HUD.app 拖入应用程序目录并打开。
3. 请先安装并登录自己的 Codex 应用或 CLI。

需要 **Apple Silicon（M 系列）和 macOS 26+**。当前版本为 **1.1.2（build 9）**。当前安装包使用临时签名，尚未进行 Developer ID 签名和 Apple 公证，首次打开可能被系统拦截。

## 功能与操作

- 36 × 36 pt 单额度小球、36 × 72 pt 双额度胶囊；环内显示数字，外缘显示额度进度。
- 跟随系统深浅外观：深色使用不透明石墨色，浅色小球使用原生磨砂。
- 点击展开详情，点击外部或按 Esc 收起；拖动调整位置，菜单支持左右停靠。
- 菜单栏以 22 × 22 pt 数字环显示额度，双额度时优先显示 5h。
- 每 5 分钟刷新额度，启动、手动刷新与唤醒时立即请求；失败保留上次成功数据并提示状态。
- Sparkle 每 6 小时检查新版，发现后提示，由用户确认下载安装；也可点击菜单“检查更新…”。

快捷键：⌘R 刷新、⌘I 详情、⌘[ 靠左、⌘] 靠右、⌘Q 退出。快捷键在应用获得键盘焦点时生效。应用驻留菜单栏，不占 Dock；暂无独立设置页面。

## 构建与测试

安装 Xcode（默认路径 `/Applications/Xcode.app`，SDK 支持 macOS 26）及 Python 3：

```sh
git clone https://github.com/WasdTornado/codex-hud-releases.git
cd codex-hud-releases
python3 scripts/test.py
python3 scripts/build.py
```

产物为 `build/Codex HUD.app`。构建脚本自动获取固定版本 Sparkle 2.10.0，并验证 SHA256 和官方代码签名；图标由 Xcode actool 编译。Python 仅用于开发。

退出旧应用后，可执行 `python3 scripts/build.py --install` 安装到当前用户应用程序目录。预览使用独立应用身份，不连接真实账号：

```sh
python3 scripts/build.py --preview
open "$HOME/Library/Caches/QuotaOrbNativePreview/Codex HUD Preview.app"
```

预览中的 82 / 47 为示例数据。详细结构与验证边界见[开发说明](docs/DEVELOPMENT.md)。

## 数据与更新

应用通过本机 Codex CLI 的 app-server stdio 接口读取额度，不将账号数据上传到本项目。显示的是 Codex 接口返回的额度，不代表 ChatGPT 所有模型的聊天额度。账号凭据由 Codex 自身管理，不随安装包或源码分发。

更新使用 Sparkle EdDSA 验签，私钥只保留在维护者钥匙串中；仓库仅保存公开更新源和公钥。1.1.0 及以上版本支持在线更新，更早版本需手动安装一次。

## 开源与发布

本仓库保存应用源码、测试、构建脚本与原生图标工程。后续代码修改经验证后同步到仓库；面向用户的 Releases Assets 仅放 DMG、在线更新 ZIP、appcast、更新说明和校验文件，不再单独上传源码 ZIP。GitHub 自动提供的 Source code 归档由平台生成，随对应 tag 的内容变化。

历史 v1.1.0 / v1.1.2 标签创建时仓库仅含 README，因此其自动源码归档不包含后续导入的应用代码。当前完整源码位于 main；后续发布的版本标签应指向对应源码提交。

项目采用 MIT 许可。版权、协议来源与第三方条款见 [LICENSE](LICENSE) 和 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。Codex HUD 是独立社区工具，与 OpenAI 无隶属或官方背书关系。
