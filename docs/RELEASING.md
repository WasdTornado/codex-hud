# 代码同步与版本发布

源码在 main 分支维护；Releases Assets 面向安装和在线升级。日常代码更新与正式版本发布分开进行。

## 日常代码同步

每次完成用户要求的代码更新后，先检查 diff、运行对应测试和构建，再提交并同步到 GitHub。排除 `.gitignore` 中的本机文件；不要使用强制推送。明确要求仅本地处理的改动不上传。

```sh
git status --short
git diff
python3 scripts/test.py
python3 scripts/build.py
git add <本次确认的文件>
git commit -m "描述具体变更"
git push origin main
```

源码提交不自动替换用户安装包。需要向用户分发新版时执行下面的发布流程。

## 准备正式版本

1. 保持 bundle ID 和正式更新源不变，递增 `app.json` 的 version 和 build。
2. 完成核心测试、正式构建及必要界面验证。更新说明使用 UTF-8 文本。
3. 将对应代码提交并同步到 main，记录提交 SHA。版本 tag 必须指向这一提交；已公开的历史标签不移动。
4. 在维护者本机生成正式产物：

```sh
python3 scripts/release.py --notes /绝对路径/更新说明.txt
```

`release.py` 只生成本地文件，不上传。它校验应用版本、身份、更新源及公钥，使用钥匙串中的私钥为 ZIP 和 appcast 签名并验签，再生成 DMG 与 SHA256 清单。不要提交或上传私钥。

## Releases Assets

只上传同一次生成的以下六项：

- `Codex-HUD-<版本>-arm64.dmg`：手动安装包。
- `Codex-HUD-<版本>-arm64.zip`：Sparkle 在线更新包，内含应用，不是源码。
- `appcast.xml`：带签名的更新清单。
- `release-notes.txt`：用户更新说明。
- `SHA256SUMS`：发布附件校验清单。
- `Codex-HUD-<版本>-arm64.dmg.sha256`：DMG 单独校验值。

不额外上传源码 ZIP、源码清单、构建日志或仓库快照。GitHub 自动显示的 Source code (zip/tar.gz) 是平台生成的标签归档，不是手动上传的 Assets，保留平台默认行为。

## 发布与验收

创建对应版本 tag 的 Release，上传完整附件后发布并设为 Latest。固定更新地址为：

`https://github.com/WasdTornado/codex-hud-releases/releases/latest/download/appcast.xml`

发布后回下载全部附件，核对哈希、appcast 版本、ZIP 长度和 EdDSA 签名；在旧版上验证发现新版及用户发起升级。正式安装更新要验证替换和重启后的版本。不要混用不同构建的 ZIP 与签名。

当前使用临时代码签名，尚未进行 Developer ID 签名与 Apple 公证；Sparkle 更新签名不能替代系统公证。
