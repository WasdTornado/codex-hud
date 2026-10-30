"""无需安装依赖的本地检查；不会将静态检查计为 Rust/Windows 构建。"""
import json, subprocess
from pathlib import Path
root=Path(__file__).resolve().parents[1]
for path in [root/'package.json',root/'src-tauri/tauri.conf.json',root/'src-tauri/capabilities/local.json']:
    json.loads(path.read_text())
config=json.loads((root/'src-tauri/tauri.conf.json').read_text())
assert not config['bundle']['active'], '本阶段禁止启用分发打包'
assert all(not w['visible'] for w in config['app']['windows']), '启动须先读取隐藏意图'
assert config['plugins']['updater']['pubkey']=='' and config['plugins']['updater']['endpoints']==[], '预览不得接入真实更新配置'
assert config['plugins']['updater']['requireSignedVersion'], '更新签名必须绑定版本'
assert (root/'src-tauri/icons/icon.ico').read_bytes()[:6]==b'\x00\x00\x01\x00\x01\x00'
subprocess.run(['node',str(root/'tests/ui.test.mjs')],check=True)
for script in ['app.js','geometry.js']:
    subprocess.run(['node','--check',str(root/'ui'/script)],check=True)
compile((root/'tests/fake_cli.py').read_text(),str(root/'tests/fake_cli.py'),'exec')
print('通过：JSON 配置、启动隐藏配置、ICO 容器、JavaScript 语法及 3 项界面逻辑测试。')
print('本脚本不运行 Rust 或 Windows 检查；各层实际结果见 docs/VALIDATION.md。')
