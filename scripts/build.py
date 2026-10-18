#!/usr/bin/env python3
"""构建独立原生应用；预览与正式应用使用不同身份，不混入示例数据。"""
from pathlib import Path
import argparse
import base64
import json
import plistlib
import shutil
import subprocess
import time
from make_icon import build_icon
from fetch_sparkle import fetch_sparkle

parser = argparse.ArgumentParser()
parser.add_argument('--install', action='store_true', help='安装到当前用户的应用程序目录')
parser.add_argument('--preview', action='store_true', help='构建独立双额度预览')
parser.add_argument('--output', type=Path, help='独立构建输出目录，默认保留原有位置')
args = parser.parse_args()
if args.install and args.preview:
    parser.error('预览不安装为正式应用')
root = Path(__file__).resolve().parent.parent
config = json.loads((root / 'app.json').read_text())
updates = json.loads((root / 'updates.json').read_text())
if not args.preview:
    expected_feed = f"https://github.com/{updates['repository']}/releases/latest/download/appcast.xml"
    try:
        public_key = base64.b64decode(updates['publicEDKey'], validate=True)
    except (ValueError, TypeError):
        raise SystemExit('EdDSA 公钥格式错误，保留已有构建。')
    if updates['feedURL'] != expected_feed or len(public_key) != 32:
        raise SystemExit('正式更新源或 EdDSA 公钥配置错误，保留已有构建。')
sparkle = fetch_sparkle()
icon_source = root / 'Assets/HUDIcon.icon'
if not (icon_source / 'icon.json').is_file():
    raise SystemExit('缺少原生图标工程 Assets/HUDIcon.icon，保留已有构建。')
name = config['name'] + (' Preview' if args.preview else '')
bundle_id = config['bundleIdentifier'] + ('.preview' if args.preview else '')
output = Path.home() / 'Library/Caches/QuotaOrbNativePreview' if args.preview else root / 'build'
if args.output:
    output = args.output.expanduser().resolve()
app = output / (name + '.app')
if app.exists():
    existing = plistlib.loads((app / 'Contents/Info.plist').read_bytes())
    if existing.get('CFBundleIdentifier') != bundle_id:
        raise SystemExit('目标位置存在其他应用，已停止构建。')
    shutil.rmtree(app)
macos = app / 'Contents/MacOS'
resources = app / 'Contents/Resources'
macos.mkdir(parents=True)
resources.mkdir()
frameworks = app / 'Contents/Frameworks'
frameworks.mkdir()
shutil.copytree(sparkle / 'Sparkle.framework', frameworks / 'Sparkle.framework', symlinks=True)
dev = Path('/Applications/Xcode.app/Contents/Developer')
compiler = dev / 'Toolchains/XcodeDefault.xctoolchain/usr/bin/swiftc'
sdk = dev / 'Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk'
subprocess.run([str(compiler), '-sdk', str(sdk), '-target', 'arm64-apple-macos26.0', '-swift-version', '5', '-parse-as-library', '-O', '-F', str(sparkle), '-framework', 'Sparkle', '-Xlinker', '-rpath', '-Xlinker', '@executable_path/../Frameworks', *map(str, sorted((root / 'Sources').glob('*.swift'))), '-o', str(macos / 'CodexHUD')], check=True)
info = {
    'CFBundleName': name, 'CFBundleDisplayName': config['displayName'] + (' · 预览' if args.preview else ''),
    'CFBundleIdentifier': bundle_id, 'CFBundleExecutable': 'CodexHUD', 'CFBundlePackageType': 'APPL',
    'CFBundleShortVersionString': config['version'], 'CFBundleVersion': config['build'],
    'LSUIElement': True, 'NSHighResolutionCapable': True, 'LSMinimumSystemVersion': config['minimumMacOS'],
    'NSHumanReadableCopyright': 'Copyright © 2026 ' + config['author'], 'QuotaAuthor': config['author'],
}
if not args.preview:
    info.update({
        'SUFeedURL': updates['feedURL'], 'SUPublicEDKey': updates['publicEDKey'],
        'SUEnableAutomaticChecks': True, 'SUScheduledCheckInterval': 21600,
        'SUAutomaticallyUpdate': False, 'SUAllowsAutomaticUpdates': False,
        'SUVerifyUpdateBeforeExtraction': True, 'SUShowReleaseNotes': True,
    })
if args.preview:
    fixture = output / 'dual-preview.json'
    now = time.time()
    fixture.write_text(json.dumps({'windows': [{'minutes': 300, 'remaining': 82, 'resetsAt': now + 7200}, {'minutes': 10080, 'remaining': 47, 'resetsAt': now + 259200}], 'updatedAt': now * 1000, 'loading': False, 'error': None}))
    info['QuotaPreviewFile'] = str(fixture)
info.update(build_icon(icon_source, resources))
(app / 'Contents/Info.plist').write_bytes(plistlib.dumps(info))
for filename in ['LICENSE', 'THIRD_PARTY_NOTICES.md']:
    shutil.copy2(root / filename, resources / filename)
shutil.copy2(sparkle / 'LICENSE', resources / 'Sparkle-LICENSE.txt')
subprocess.run(['codesign', '--force', '--sign', '-', str(app)], check=True, capture_output=True)
subprocess.run(['codesign', '--verify', '--deep', '--strict', str(app)], check=True)
if args.install:
    installed = Path.home() / 'Applications' / app.name
    installed.parent.mkdir(exist_ok=True)
    if installed.exists():
        existing = plistlib.loads((installed / 'Contents/Info.plist').read_bytes())
        if existing.get('CFBundleIdentifier') != bundle_id:
            raise SystemExit('安装位置存在其他应用，已停止。')
        shutil.rmtree(installed)
    shutil.copytree(app, installed, symlinks=True)
    subprocess.run(['codesign', '--verify', '--deep', '--strict', str(installed)], check=True)
    app = installed
print(app)
