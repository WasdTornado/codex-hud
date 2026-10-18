#!/bin/zsh
# 日常直接打开安装版，不重新签名或访问桌面源码。
app="$HOME/Applications/Codex HUD.app"
if [[ ! -d "$app" ]]; then
  print '尚未安装，请先在源码目录运行 python3 scripts/build.py --install。'
  exit 1
fi
open "$app"
