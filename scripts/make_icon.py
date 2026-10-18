#!/usr/bin/env python3
"""使用苹果资源编译器生成原生图标栈，避免旧式透明图标被套入浅色底板。"""
from pathlib import Path
import os
import plistlib
import subprocess
import tempfile


def build_icon(document, resources):
    document, resources = Path(document), Path(resources)
    if not (document / 'icon.json').is_file():
        raise SystemExit('缺少原生图标工程 Assets/HUDIcon.icon。')
    developer = Path('/Applications/Xcode.app/Contents/Developer')
    environment = dict(os.environ, DEVELOPER_DIR=str(developer))
    resources.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='codex-hud-icon-') as temporary:
        partial_info = Path(temporary) / 'icon-info.plist'
        subprocess.run([
            str(developer / 'usr/bin/actool'), str(document),
            '--compile', str(resources), '--platform', 'macosx',
            '--minimum-deployment-target', '26.0', '--app-icon', document.stem,
            '--output-partial-info-plist', str(partial_info),
            '--output-format', 'human-readable-text',
        ], check=True, env=environment, stdout=subprocess.DEVNULL)
        return plistlib.loads(partial_info.read_bytes())


if __name__ == '__main__':
    import sys
    build_icon(sys.argv[1], sys.argv[2])
