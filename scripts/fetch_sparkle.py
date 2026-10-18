#!/usr/bin/env python3
"""获取固定版本的官方 Sparkle，校验压缩包后保留框架符号链接。"""
from pathlib import Path
import hashlib
import json
import shutil
import subprocess
import tempfile
import urllib.request

VERSION = '2.10.0'
SHA256 = 'c2bf58aa8387266ac179357b1415d6f2635f044da8be41042af32425dae6da0c'
URL = f'https://github.com/sparkle-project/Sparkle/releases/download/{VERSION}/Sparkle-{VERSION}.tar.xz'
ROOT = Path(__file__).resolve().parent.parent


def fetch_sparkle():
    destination = ROOT / 'ThirdParty' / 'Sparkle' / VERSION
    marker = destination / 'provenance.json'
    if marker.exists():
        provenance = json.loads(marker.read_text())
        if provenance != {'version': VERSION, 'url': URL, 'sha256': SHA256}:
            raise SystemExit('Sparkle 来源记录不一致，请先人工检查依赖目录。')
    else:
        destination.parent.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='codex-hud-sparkle-') as temporary:
            archive = Path(temporary) / 'Sparkle.tar.xz'
            with urllib.request.urlopen(URL, timeout=120) as response, archive.open('wb') as output:
                shutil.copyfileobj(response, output)
            if hashlib.sha256(archive.read_bytes()).hexdigest() != SHA256:
                raise SystemExit('Sparkle 下载的 SHA-256 不匹配，拒绝解压。')
            extracted = Path(temporary) / 'unpacked'
            extracted.mkdir()
            subprocess.run(['tar', '-xf', str(archive), '-C', str(extracted)], check=True)
            if destination.exists():
                raise SystemExit('Sparkle 目录已存在但没有来源记录，拒绝覆盖。')
            shutil.copytree(extracted, destination, symlinks=True)
            marker.write_text(json.dumps({'version': VERSION, 'url': URL, 'sha256': SHA256}, indent=2) + '\n')
    subprocess.run(['codesign', '--verify', '--deep', '--strict', str(destination / 'Sparkle.framework')], check=True)
    for executable in ['generate_keys', 'sign_update', 'generate_appcast']:
        subprocess.run(['codesign', '--verify', '--strict', str(destination / 'bin' / executable)], check=True)
    return destination


if __name__ == '__main__':
    print(fetch_sparkle())
