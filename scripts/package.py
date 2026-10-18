#!/usr/bin/env python3
"""生成本地测试 DMG；不上传、不包含用户登录数据、不进行远端发布。"""
from pathlib import Path
import hashlib
import json
import plistlib
import shutil
import subprocess
import tempfile
import argparse

root = Path(__file__).resolve().parent.parent
parser = argparse.ArgumentParser()
parser.add_argument('--app', type=Path, help='指定已经构建的正式应用')
parser.add_argument('--output', type=Path, help='安装包输出目录')
args = parser.parse_args()
config = json.loads((root / 'app.json').read_text())
app = args.app or root / 'build' / (config['name'] + '.app')
if not app.exists():
    raise SystemExit('请先运行 python3 scripts/build.py。')
info = plistlib.loads((app / 'Contents/Info.plist').read_bytes())
if info.get('CFBundleIdentifier') != config['bundleIdentifier'] or 'QuotaPreviewFile' in info:
    raise SystemExit('仅允许打包当前正式应用，不能打包预览。')
if info.get('CFBundleShortVersionString') != config['version'] or info.get('CFBundleVersion') != config['build']:
    raise SystemExit('构建版本与配置不一致，请重新构建。')
subprocess.run(['codesign', '--verify', '--deep', '--strict', str(app)], check=True)
dist = args.output or root / 'dist'
dist.mkdir(parents=True, exist_ok=True)
dmg = dist / ('Codex-HUD-' + config['version'] + '-arm64.dmg')
if dmg.exists():
    raise SystemExit('该版本安装包已经存在；请增加版本号或将旧包移到其他位置。')
with tempfile.TemporaryDirectory(prefix='codex-hud-package-') as temporary:
    stage = Path(temporary) / 'stage'
    stage.mkdir()
    shutil.copytree(app, stage / app.name, symlinks=True)
    (stage / 'Applications').symlink_to('/Applications')
    (stage / '安装说明.txt').write_text(
        'Codex HUD — wasdTornado\n\n'
        '将 Codex HUD.app 拖入 Applications 后，从应用程序目录启动。\n'
        '运行环境：Apple Silicon（M 系列芯片），macOS 26 或更高版本。\n'
        '请先在自己的电脑安装并登录 Codex 应用或 CLI；安装包不包含任何账号或额度数据。\n'
        '此包为本地测试构建，尚未使用 Developer ID 签名和 Apple 公证。\n'
        '系统可能阻止首次运行；正式分发请使用已签名并公证的版本。\n'
        '应用会每 6 小时检查新版；发现新版后可从菜单确认下载并安装。\n'
        '更新包由 Sparkle 验证 EdDSA 签名；不会自动替你安装。\n', encoding='utf-8')
    subprocess.run(['hdiutil', 'create', '-volname', 'Codex HUD', '-srcfolder', str(stage), '-format', 'UDZO', str(dmg)], check=True)
subprocess.run(['hdiutil', 'verify', str(dmg)], check=True)
digest = hashlib.sha256(dmg.read_bytes()).hexdigest()
dmg.with_suffix('.dmg.sha256').write_text(digest + '  ' + dmg.name + '\n')
print(dmg)
