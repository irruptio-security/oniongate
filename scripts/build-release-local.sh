#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

target="$(rustc -vV | awk '/^host:/{print $2}')"
extension=""
if [[ "$target" == *windows* ]]; then extension=".exe"; fi
case "$target" in
  *apple-darwin) bundles="app,dmg" ;;
  *windows*) bundles="nsis" ;;
  *linux*) bundles="appimage,deb,rpm" ;;
  *)
    echo "Unsupported release host: $target" >&2
    exit 1
    ;;
esac

echo "==> Building unsigned local release bundle for $target"
npm run deps
cargo build \
  --manifest-path src-tauri/Cargo.toml \
  --release \
  --bin oniongate-helper \
  --target "$target"

source_path="src-tauri/target/$target/release/oniongate-helper$extension"
destination="src-tauri/binaries/oniongate-helper-$target$extension"
test -f "$source_path"
cp "$source_path" "$destination"
if [[ "$target" != *windows* ]]; then chmod 755 "$destination"; fi

CI=true npm run tauri -- build \
  --target "$target" \
  --bundles "$bundles" \
  --config src-tauri/tauri.release.conf.json \
  --no-sign

if [[ "$target" == *apple-darwin ]]; then
  app_bundle="$(printf '%s\n' src-tauri/target/"$target"/release/bundle/macos/*.app | head -n 1)"
  test -x "$app_bundle/Contents/MacOS/oniongate-helper"
  test -x "$app_bundle/Contents/MacOS/oniongate"
fi

echo "==> Local release bundle verified (unsigned)"
case "$target" in
  *apple-darwin)
    printf '    DMG: %s\n' src-tauri/target/"$target"/release/bundle/dmg/*.dmg
    ;;
  *windows*)
    printf '    EXE: %s\n' src-tauri/target/"$target"/release/bundle/nsis/*.exe
    ;;
  *linux*)
    printf '    AppImage: %s\n' src-tauri/target/"$target"/release/bundle/appimage/*.AppImage
    printf '    DEB: %s\n' src-tauri/target/"$target"/release/bundle/deb/*.deb
    printf '    RPM: %s\n' src-tauri/target/"$target"/release/bundle/rpm/*.rpm
    ;;
esac
