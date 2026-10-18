# 开源来源与署名

Codex HUD 由 **wasdTornado** 维护，原生界面、交互及独立 Swift 核心在此项目中整理。

额度读取协议及窗口转换逻辑源自 [dedsec1911/codex-meter](https://github.com/dedsec1911/codex-meter)，原本地来源版本 v0.2.0，MIT License，Copyright (c) 2026 dedsec1911。原始许可证保留于 LICENSE。

本应用不包含上游 Electron 界面、Node 依赖、安装包或运行时。使用用户已安装且已登录的 Codex CLI，通过本地 app-server stdio 接口读取额度。Codex CLI 属于外部运行依赖，不随本应用分发。

Codex HUD 为独立社区工具，与 OpenAI 无隶属或官方背书关系。

## Sparkle 2.10.0

从 1.1.0 起，应用内嵌 [Sparkle](https://github.com/sparkle-project/Sparkle) 2.10.0（`Sparkle.framework` 及更新辅助组件），用于应用更新检查、签名校验与安装。Sparkle 主体采用 MIT License；其版权声明包括 Andy Matuschak、Elgato Systems GmbH、Kornel Lesiński、Mayur Pawashe、C.W. Betts、Petroules Corporation 与 Big Nerd Ranch。

完整许可证保留于 `ThirdParty/Sparkle/2.10.0/LICENSE`，构建时复制为应用资源中的 `Sparkle-LICENSE.txt`。该文件还保留 bsdiff / bspatch、sais-lite、Ed25519 实现及 SUSignatureVerifier 的各自版权与许可条款；这些第三方条款不被本项目的作者署名或 MIT 许可替代。

Sparkle 发行包中的开发与签名工具用于本机构建发布流程，不作为用户运行依赖。EdDSA 私钥保留于作者的 macOS Keychain，不随应用、更新包或源码分发。
