#!/usr/bin/env python3
"""隔离运行真实 Sparkle 更新：临时身份、localhost 源，安装需界面确认。"""
from pathlib import Path
import argparse
import functools
import hashlib
import http.server
import json
import plistlib
import shutil
import subprocess
import tempfile
import threading
import time
import uuid
import xml.etree.ElementTree as ET
from fetch_sparkle import fetch_sparkle

ROOT = Path(__file__).resolve().parent.parent
NS = 'http://www.andymatuschak.org/xml-namespaces/sparkle'
ET.register_namespace('sparkle', NS)


def run(arguments):
    return subprocess.run(list(map(str, arguments)), check=True, capture_output=True, text=True).stdout.strip()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tamper', action='store_true', help='签名后改动包，验证拒绝安装')
    args = parser.parse_args()
    sparkle = fetch_sparkle()
    updates = json.loads((ROOT / 'updates.json').read_text())
    work = Path(tempfile.mkdtemp(prefix='codex-hud-update-fixture-'))
    identity = 'com.wasdtornado.codexhud.fixture.' + uuid.uuid4().hex
    log = work / 'events.jsonl'
    log.touch()
    handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=str(work))
    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), handler)
    port = server.server_address[1]
    threading.Thread(target=server.serve_forever, daemon=True).start()
    binary = work / 'Fixture'
    developer = Path('/Applications/Xcode.app/Contents/Developer')
    run([developer / 'Toolchains/XcodeDefault.xctoolchain/usr/bin/swiftc', '-sdk', developer / 'Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk', '-target', 'arm64-apple-macos26.0', '-swift-version', '5', '-parse-as-library', '-F', sparkle, '-framework', 'Sparkle', '-Xlinker', '-rpath', '-Xlinker', '@executable_path/../Frameworks', ROOT / 'Sources/AppUpdater.swift', ROOT / 'Tests/UpdateFixture.swift', '-o', binary])
    def make_app(directory, version):
        app = work / directory / 'Codex HUD Update Fixture.app'
        (app / 'Contents/MacOS').mkdir(parents=True)
        (app / 'Contents/Frameworks').mkdir()
        shutil.copy2(binary, app / 'Contents/MacOS/Fixture')
        shutil.copytree(sparkle / 'Sparkle.framework', app / 'Contents/Frameworks/Sparkle.framework', symlinks=True)
        info = {
            'CFBundleName': 'Codex HUD Update Fixture', 'CFBundleIdentifier': identity,
            'CFBundleExecutable': 'Fixture', 'CFBundlePackageType': 'APPL',
            'CFBundleVersion': str(version), 'CFBundleShortVersionString': f'0.0.{version}',
            'LSMinimumSystemVersion': '26.0', 'LSUIElement': True,
            'SUFeedURL': f'http://127.0.0.1:{port}/appcast.xml', 'SUPublicEDKey': updates['publicEDKey'],
            'SUEnableAutomaticChecks': True, 'SUScheduledCheckInterval': 21600,
            'SUAutomaticallyUpdate': False, 'SUAllowsAutomaticUpdates': False,
            'SUVerifyUpdateBeforeExtraction': True, 'SUShowReleaseNotes': False,
            'NSAppTransportSecurity': {'NSAllowsLocalNetworking': True}, 'FixtureLog': str(log),
        }
        (app / 'Contents/Info.plist').write_bytes(plistlib.dumps(info))
        run(['codesign', '--force', '--sign', '-', app])
        run(['codesign', '--verify', '--deep', '--strict', app])
        return app
    old = make_app('installed', 1)
    new = make_app('new', 2)
    archive = work / 'fixture-update.zip'
    run(['ditto', '-c', '-k', '--sequesterRsrc', '--keepParent', new, archive])
    signature = run([sparkle / 'bin/sign_update', '--account', updates['keychainAccount'], '-p', archive])
    if args.tamper:
        with archive.open('ab') as output:
            output.write(b'fixture-tamper')
    rss = ET.Element('rss', {'version': '2.0'})
    channel = ET.SubElement(rss, 'channel')
    ET.SubElement(channel, 'title').text = '隔离更新测试'
    item = ET.SubElement(channel, 'item')
    ET.SubElement(item, 'title').text = 'Fixture 0.0.2'
    ET.SubElement(item, f'{{{NS}}}version').text = '2'
    ET.SubElement(item, f'{{{NS}}}shortVersionString').text = '0.0.2'
    ET.SubElement(item, f'{{{NS}}}minimumSystemVersion').text = '26.0'
    ET.SubElement(item, 'enclosure', {'url': f'http://127.0.0.1:{port}/{archive.name}', 'length': str(archive.stat().st_size), 'type': 'application/octet-stream', f'{{{NS}}}edSignature': signature})
    ET.ElementTree(rss).write(work / 'appcast.xml', encoding='utf-8', xml_declaration=True)
    output = (work / 'process.log').open('w')
    process = subprocess.Popen([str(old / 'Contents/MacOS/Fixture')], stdout=output, stderr=output)
    print('隔离测试目录：' + str(work), flush=True)
    success = False
    try:
        deadline = time.monotonic() + 600
        while time.monotonic() < deadline:
            events = [json.loads(line) for line in log.read_text().splitlines() if line]
            if args.tamper:
                success = any(e.get('event') == 'state' and '检查更新暂不可用' in e.get('state', '') for e in events)
            else:
                success = any(e.get('event') == 'launched' and e.get('build') == '2' for e in events)
            if success:
                break
            time.sleep(0.5)
        installed = plistlib.loads((old / 'Contents/Info.plist').read_bytes())['CFBundleVersion']
        gentle = [e for e in events if e['event'] == 'gentle_update_found']
        success = success and bool(gentle) and gentle[0]['visibleWindows'] == 0 and installed == ('1' if args.tamper else '2')
        result = {'success': success, 'tampered': args.tamper, 'installedBuild': installed, 'identity': identity, 'events': events, 'archiveSHA256': hashlib.sha256(archive.read_bytes()).hexdigest()}
        (work / 'result.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
        print(json.dumps(result, ensure_ascii=False, indent=2))
    finally:
        if process.poll() is None:
            process.terminate()
        output.close()
        server.shutdown()
        # 仅清理本次随机测试身份的设置，不读取或改动正式应用设置。
        subprocess.run(['defaults', 'delete', identity], capture_output=True)
    if not success:
        raise SystemExit('隔离更新测试未通过，请检查保留目录中的日志。')


if __name__ == '__main__':
    main()
