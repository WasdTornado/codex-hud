#!/usr/bin/env python3
"""编译并运行原生核心测试，不读取真实账号。"""
from pathlib import Path
import subprocess
root = Path(__file__).resolve().parent.parent
build = root / 'build/tests'
build.mkdir(parents=True, exist_ok=True)
dev = Path('/Applications/Xcode.app/Contents/Developer')
compiler = dev / 'Toolchains/XcodeDefault.xctoolchain/usr/bin/swiftc'
sdk = dev / 'Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk'
sources = [root / 'Sources' / name for name in ['QuotaModels.swift', 'QuotaClient.swift', 'QuotaStore.swift']]
sources.append(root / 'Tests/QuotaCoreTests.swift')
binary = build / 'quota-tests'
subprocess.run([str(compiler), '-sdk', str(sdk), '-swift-version', '5', '-parse-as-library', *map(str, sources), '-o', str(binary)], check=True)
subprocess.run([str(binary), str(root / 'Tests/FakeCodex.py')], check=True)
