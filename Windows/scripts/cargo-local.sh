#!/bin/sh
# 只使用项目内已获授权安装的工具链，不修改全局 PATH 或 shell 配置。
set -eu
project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
export CARGO_HOME="$project_dir/.tools/cargo"
export RUSTUP_HOME="$project_dir/.tools/rustup"
export RUSTC="$CARGO_HOME/bin/rustc"
export RUSTDOC="$CARGO_HOME/bin/rustdoc"
if [ ! -x "$CARGO_HOME/bin/cargo" ]; then
  echo '项目内 Rust 尚未安装；此脚本不会自动下载或安装。' >&2
  exit 2
fi
exec "$CARGO_HOME/bin/cargo" "$@"
