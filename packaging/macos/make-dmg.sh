#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
ARM=target/aarch64-apple-darwin/release/melody-quack
X86=target/x86_64-apple-darwin/release/melody-quack
APP="dist/Melody Quack.app"
rm -rf dist/dmg "$APP" dist/AppIcon.iconset
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
if [[ -f $ARM && -f $X86 ]]; then
  lipo -create "$ARM" "$X86" -output "$APP/Contents/MacOS/melody-quack"
elif [[ -f $ARM ]]; then
  cp "$ARM" "$APP/Contents/MacOS/melody-quack"
elif [[ -f $X86 ]]; then
  cp "$X86" "$APP/Contents/MacOS/melody-quack"
else
  cp target/release/melody-quack "$APP/Contents/MacOS/melody-quack"
fi
sed "s/__VERSION__/$VERSION/g" packaging/macos/Info.plist > "$APP/Contents/Info.plist"
ICONSET=dist/AppIcon.iconset
mkdir -p "$ICONSET"
for s in 16 32 128 256 512; do
  sips -z $s $s packaging/icon1024.png --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
  sips -z $((s * 2)) $((s * 2)) packaging/icon1024.png --out "$ICONSET/icon_${s}x${s}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/AppIcon.icns"
codesign --force --deep --sign - "$APP"
mkdir -p dist/dmg
cp -R "$APP" dist/dmg/
ln -s /Applications dist/dmg/Applications
hdiutil create -volname "Melody Quack" -srcfolder dist/dmg -ov -format UDZO "dist/Melody-Quack-$VERSION.dmg"
echo "built dist/Melody-Quack-$VERSION.dmg"
