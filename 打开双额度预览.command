#!/bin/zsh
# 预览有独立身份和明显的数据标识，不修改真实账号数据。
app="$HOME/Library/Caches/QuotaOrbNativePreview/Codex HUD Preview.app"
if [[ ! -d "$app" ]]; then
  print '尚未构建预览，请先运行 python3 scripts/build.py --preview。'
  exit 1
fi
open "$app"
