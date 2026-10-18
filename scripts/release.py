#!/usr/bin/env python3
"""生成签名更新包与 appcast；只写本地产物，不创建仓库或上传文件。"""
from pathlib import Path
import argparse
import datetime
import email.utils
import hashlib
import json
import plistlib
import re
import subprocess
import sys
import urllib.parse
import xml.etree.ElementTree as ET
from fetch_sparkle import fetch_sparkle

ROOT = Path(__file__).resolve().parent.parent
SPARKLE_NS = 'http://www.andymatuschak.org/xml-namespaces/sparkle'
ET.register_namespace('sparkle', SPARKLE_NS)


def run(command):
    return subprocess.run(list(map(str, command)), check=True, capture_output=True, text=True).stdout.strip()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, help='已完成验证的正式应用')
    parser.add_argument('--notes', type=Path, required=True, help='UTF-8 纯文本更新说明')
    parser.add_argument('--output', type=Path, help='新的产物目录；不会覆盖已有版本')
    args = parser.parse_args()
    config = json.loads((ROOT / 'app.json').read_text())
    updates = json.loads((ROOT / 'updates.json').read_text())
    app = (args.app or ROOT / 'build' / (config['name'] + '.app')).resolve()
    info = plistlib.loads((app / 'Contents/Info.plist').read_bytes())
    for key, expected in [('CFBundleIdentifier', config['bundleIdentifier']), ('CFBundleShortVersionString', config['version']), ('CFBundleVersion', config['build']), ('SUFeedURL', updates['feedURL']), ('SUPublicEDKey', updates['publicEDKey'])]:
        if info.get(key) != expected:
            raise SystemExit(f'应用的 {key} 与正式配置不一致，请重新构建。')
    if 'QuotaPreviewFile' in info or not info.get('SUVerifyUpdateBeforeExtraction'):
        raise SystemExit('拒绝打包预览或未启用解压前验签的应用。')
    if not re.fullmatch(r'[0-9]+(?:\.[0-9]+)*', config['version']):
        raise SystemExit('正式版本号只能由数字和点组成。')
    repo = updates['repository']
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', repo):
        raise SystemExit('发布仓库标识无效。')
    if updates['feedURL'] != f'https://github.com/{repo}/releases/latest/download/appcast.xml':
        raise SystemExit('更新源必须指向配置中发布仓库的 latest appcast。')
    notes = args.notes.read_text(encoding='utf-8').strip()
    if not notes:
        raise SystemExit('更新说明不能为空。')
    sparkle = fetch_sparkle()
    public = run([sparkle / 'bin/generate_keys', '--account', updates['keychainAccount'], '-p'])
    if public != updates['publicEDKey']:
        raise SystemExit('钥匙串公钥与应用公钥不一致，拒绝发布。')
    run(['codesign', '--verify', '--deep', '--strict', app])
    output = (args.output or ROOT / 'dist' / ('v' + config['version'])).resolve()
    output.mkdir(parents=True, exist_ok=False)
    archive = output / f"Codex-HUD-{config['version']}-arm64.zip"
    run(['ditto', '-c', '-k', '--sequesterRsrc', '--keepParent', app, archive])
    signer = sparkle / 'bin/sign_update'
    signature = run([signer, '--account', updates['keychainAccount'], '-p', archive])
    run([signer, '--account', updates['keychainAccount'], '--verify', archive, signature])
    # 只从正式配置和已签名压缩包生成元数据，不接受临时测试地址。
    rss = ET.Element('rss', {'version': '2.0'})
    channel = ET.SubElement(rss, 'channel')
    ET.SubElement(channel, 'title').text = config['name'] + ' 更新'
    ET.SubElement(channel, 'link').text = f'https://github.com/{repo}/releases'
    ET.SubElement(channel, 'description').text = 'Codex HUD 正式版更新'
    ET.SubElement(channel, 'language').text = 'zh-CN'
    item = ET.SubElement(channel, 'item')
    ET.SubElement(item, 'title').text = config['name'] + ' ' + config['version']
    ET.SubElement(item, 'pubDate').text = email.utils.format_datetime(datetime.datetime.now(datetime.timezone.utc))
    ET.SubElement(item, 'description').text = notes
    ET.SubElement(item, f'{{{SPARKLE_NS}}}version').text = config['build']
    ET.SubElement(item, f'{{{SPARKLE_NS}}}shortVersionString').text = config['version']
    ET.SubElement(item, f'{{{SPARKLE_NS}}}minimumSystemVersion').text = config['minimumMacOS']
    ET.SubElement(item, 'enclosure', {
        'url': f"https://github.com/{repo}/releases/download/v{config['version']}/{urllib.parse.quote(archive.name)}",
        'length': str(archive.stat().st_size), 'type': 'application/octet-stream',
        f'{{{SPARKLE_NS}}}edSignature': signature,
    })
    appcast = output / 'appcast.xml'
    ET.indent(rss, space='  ')
    ET.ElementTree(rss).write(appcast, encoding='utf-8', xml_declaration=True)
    run([signer, '--account', updates['keychainAccount'], appcast])
    run([signer, '--account', updates['keychainAccount'], '--verify', appcast])
    subprocess.run([sys.executable, str(ROOT / 'scripts/package.py'), '--app', str(app), '--output', str(output)], check=True)
    (output / 'release-notes.txt').write_text(notes + '\n', encoding='utf-8')
    files = sorted(path for path in output.iterdir() if path.is_file() and not path.name.endswith('.sha256'))
    (output / 'SHA256SUMS').write_text(''.join(hashlib.sha256(path.read_bytes()).hexdigest() + '  ' + path.name + '\n' for path in files))
    print(output)
    print('仅已生成本地发布产物；尚未上传，尚未 Developer ID 签名或 Apple 公证。')


if __name__ == '__main__':
    main()
