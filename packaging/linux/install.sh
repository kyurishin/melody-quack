#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
PREFIX="${PREFIX:-$HOME/.local}"
install -Dm755 melody-quack "$PREFIX/bin/melody-quack"
install -Dm644 melody-quack.png "$PREFIX/share/icons/hicolor/256x256/apps/melody-quack.png"
install -Dm644 melody-quack.desktop "$PREFIX/share/applications/melody-quack.desktop"
echo "installed to $PREFIX. run: melody-quack"
