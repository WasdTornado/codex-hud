# Windows 测试向量与依赖声明

应用沿用项目根目录 MIT 许可和 Codex HUD 原图标，作者 wasdTornado。

`src-tauri/tests/fixtures/update.json` 中的公钥和签名来自 Tauri 官方 `tauri-plugin-updater 2.14.1` 的 `src/updater.rs::offline_tests`，payload 固定为 `test`。这些是公开测试向量，没有私钥。测试使用实际插件的 `Update::verify`，不调用安装；其许可为 MIT OR Apache-2.0，此处按 MIT 保留声明。

运行依赖由两个 Cargo.lock 固定版本；未把第三方源码、构建缓存或二进制打包进源码目录。未生成对外安装包。

## Tauri MIT 许可

MIT License

Copyright (c) 2017 - Present Tauri Apps Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.