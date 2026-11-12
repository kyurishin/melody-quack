#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
ARCH=$(uname -m)
BIN=target/release/melody-quack
NAME="melody-quack-$VERSION-linux-$ARCH"
mkdir -p dist
rm -rf "dist/$NAME" dist/AppDir
mkdir -p "dist/$NAME"
cp "$BIN" packaging/linux/melody-quack.desktop packaging/linux/melody-quack.png packaging/linux/install.sh README.md "dist/$NAME/"
chmod +x "dist/$NAME/install.sh"
tar -C dist -czf "dist/$NAME.tar.gz" "$NAME"
echo "built dist/$NAME.tar.gz"
if [[ -n "${APPIMAGETOOL:-}" ]]; then
  mkdir -p dist/AppDir/usr/bin
  cp "$BIN" dist/AppDir/usr/bin/melody-quack
  cp packaging/linux/melody-quack.desktop packaging/linux/melody-quack.png dist/AppDir/
  ln -s usr/bin/melody-quack dist/AppDir/AppRun
  ARCH=$ARCH "$APPIMAGETOOL" dist/AppDir "dist/Melody-Quack-$VERSION-$ARCH.AppImage"
  echo "built dist/Melody-Quack-$VERSION-$ARCH.AppImage"
fi
