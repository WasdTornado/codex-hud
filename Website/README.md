# Codex HUD 官网

定稿采用原版 B 夜航布局。纯静态 HTML / CSS / JavaScript，无依赖、无构建步骤；通过 GitHub Pages 托管。

官网：https://wasdtornado.github.io/codex-hud/

## 预览

在项目根目录运行：

```sh
python3 -m http.server 56613 --bind 127.0.0.1 --directory Website
```

打开 http://127.0.0.1:56613/ 。网站根目录就是 Website，部署时仅使用此目录内容。

## 内容与交互

- index.html：唯一官网入口，功能、安装与更新、常见问题。
- site.css：夜航样式、毛玻璃吸顶导航、移动端、下载提示。
- quota-animation.js：首次进入时，数字和圆环从 0 增长至 82%，红→黄→绿；减少动态效果时直接展示结果。
- demo.js：单／双额度、浅／深色、点击查看详情与 Esc 收起，均为演示数据。
- download.js：保留浏览器原生附件下载，同时显示可关闭的 Star 邀请与安装说明。不检查、不自动操作用户的 Star 状态。
- assets/icon.png：现有应用图标。

A、C、方案总览、设计切换条及旧精修样式已从官网移除；历史方案备份在项目 build/website-concepts-backup，不属于网站内容。

## 下载与版本维护

当前官网直接下载已核实发布的 v1.1.3 Apple Silicon DMG，要求 macOS 26+。按钮旁明确展示版本，不自动推断用户芯片。浏览器自行处理下载与进度，网站不声称下载已完成。

安装包直链：https://github.com/WasdTornado/codex-hud/releases/download/v1.1.3/Codex-HUD-1.1.3-arm64.dmg

发布新版后，应同步修改 index.html 内的版本文案、安装包文件名、直链、大小及 SHA256SUMS 链接，并实测下载。当前采用固定版本的附件直链，不依赖 GitHub API，避免未登录限流导致用户无法下载。不要将带版本的文件名直接拼到 releases/latest/download 后面，否则新版本移除旧文件名时会失效。

首次打开说明如实注明：当前应用未进行 Developer ID 签名与公证。下载文件托管于 GitHub，访问受用户网络环境影响。Star 入口仅由用户主动点击后打开项目仓库，不阻挡下载，也不请求账号授权。

## 本地验证

通过 ego-browser 验证：实际点击下载并保存 DMG、SHA256 与 GitHub 发布值一致、提示开关与 Esc/键盘焦点、安装帮助、手机与桌面布局、吸顶导航、额度演示。验证下载及截图保存在 build/website-checks，不属于官网内容。

## 发布方式

main 分支的 Website/ 是官网源码；gh-pages 分支根目录是发布产物，GitHub Pages 从 gh-pages / (root) 发布。发布时同步 Website/ 的文件到 gh-pages 并保留 .nojekyll，不复制原生应用源码、build/ 或本机验证文件。主分支更新不会自动替换 gh-pages，改动官网后需要同步发布分支。

每次更新安装包版本时，先确认 Release 附件可下载，再同步官网和根 README 的下载入口。发布完成后验证线上 HTML、CSS、JavaScript、图标、安装包下载及 Star 邀请。
