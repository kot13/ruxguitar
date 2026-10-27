#!/usr/bin/env sh
# Build a release binary and pack it as a macOS .app bundle.
#
# Usage (from anywhere):
#   ./packaging/macos-bundle.sh
#   ./packaging/macos-bundle.sh --install
#   ./packaging/macos-bundle.sh --open
#
# Output: dist/Ruxguitar.app (existing bundle is replaced).
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"

lsregister=/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister

open_after=0
install_after=0
for arg in "$@"; do
  case "$arg" in
    --open) open_after=1 ;;
    --install) install_after=1 ;;
    -h | --help)
      printf '%s\n' "Usage: packaging/macos-bundle.sh [--install] [--open]"
      exit 0
      ;;
    *)
      printf 'macos-bundle.sh: unknown option: %s\n' "$arg" >&2
      exit 1
      ;;
  esac
done

if [ "$(uname -s)" != "Darwin" ]; then
  printf 'macos-bundle.sh: macOS only\n' >&2
  exit 1
fi

version=$(awk -F'"' '/^version = / { print $2; exit }' Cargo.toml)
bundle_id="com.ruxguitar.app"
app_name="Ruxguitar"
dist="$root/dist"
bundle="$dist/$app_name.app"
macos_dir="$bundle/Contents/MacOS"
resources_dir="$bundle/Contents/Resources"

printf 'Building release binary...\n'
cargo build --release --locked

bin="$root/target/release/ruxguitar"
if [ ! -x "$bin" ]; then
  bin=""
  for f in $(ls -t "$root/target/release/deps"/ruxguitar-* 2>/dev/null); do
    case "$f" in *.d) ;; *)
      bin=$f
      break
    esac
  done
fi
if [ ! -x "$bin" ]; then
  printf 'macos-bundle.sh: release binary not found under target/release\n' >&2
  exit 1
fi

rm -rf "$bundle"
mkdir -p "$macos_dir" "$resources_dir"
cp "$bin" "$macos_dir/ruxguitar-bin"
cat >"$macos_dir/ruxguitar" <<'EOF'
#!/bin/sh
DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
exec "$DIR/ruxguitar-bin" "$@"
EOF
chmod +x "$macos_dir/ruxguitar" "$macos_dir/ruxguitar-bin"

icon_work="$root/packaging/.icon-build"
iconset="$icon_work/AppIcon.iconset"
rm -rf "$icon_work"
mkdir -p "$iconset"
swift "$root/packaging/render-macos-icon.swift" "$iconset"
iconutil -c icns "$iconset" -o "$resources_dir/AppIcon.icns"

cat >"$bundle/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleExecutable</key>
  <string>ruxguitar</string>
  <key>CFBundleIdentifier</key>
  <string>$bundle_id</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>$app_name</string>
  <key>CFBundleDisplayName</key>
  <string>$app_name</string>
  <key>CFBundleIconFile</key>
  <string>AppIcon</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>$version</string>
  <key>CFBundleVersion</key>
  <string>$version</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>NSPrincipalClass</key>
  <string>NSApplication</string>
  <key>CFBundleDocumentTypes</key>
  <array>
    <dict>
      <key>CFBundleTypeName</key>
      <string>Guitar Pro document</string>
      <key>CFBundleTypeRole</key>
      <string>Viewer</string>
      <key>LSHandlerRank</key>
      <string>Alternate</string>
      <key>LSItemContentTypes</key>
      <array>
        <string>com.arobas-music.guitarpro.document</string>
      </array>
      <key>CFBundleTypeExtensions</key>
      <array>
        <string>gp</string>
      </array>
    </dict>
    <dict>
      <key>CFBundleTypeName</key>
      <string>Guitar Pro 3 document</string>
      <key>CFBundleTypeRole</key>
      <string>Viewer</string>
      <key>LSHandlerRank</key>
      <string>Alternate</string>
      <key>LSItemContentTypes</key>
      <array>
        <string>com.arobas-music.guitarpro3.document</string>
      </array>
      <key>CFBundleTypeExtensions</key>
      <array>
        <string>gp3</string>
      </array>
    </dict>
    <dict>
      <key>CFBundleTypeName</key>
      <string>Guitar Pro 4 document</string>
      <key>CFBundleTypeRole</key>
      <string>Viewer</string>
      <key>LSHandlerRank</key>
      <string>Alternate</string>
      <key>LSItemContentTypes</key>
      <array>
        <string>com.arobas-music.guitarpro4.document</string>
      </array>
      <key>CFBundleTypeExtensions</key>
      <array>
        <string>gp4</string>
      </array>
    </dict>
    <dict>
      <key>CFBundleTypeName</key>
      <string>Guitar Pro 5 document</string>
      <key>CFBundleTypeRole</key>
      <string>Viewer</string>
      <key>LSHandlerRank</key>
      <string>Alternate</string>
      <key>LSItemContentTypes</key>
      <array>
        <string>com.arobas-music.guitarpro5.document</string>
      </array>
      <key>CFBundleTypeExtensions</key>
      <array>
        <string>gp5</string>
      </array>
    </dict>
    <dict>
      <key>CFBundleTypeName</key>
      <string>Guitar Pro 6 document</string>
      <key>CFBundleTypeRole</key>
      <string>Viewer</string>
      <key>LSHandlerRank</key>
      <string>Alternate</string>
      <key>LSItemContentTypes</key>
      <array>
        <string>com.arobas-music.guitarpro6.document</string>
      </array>
      <key>CFBundleTypeExtensions</key>
      <array>
        <string>gpx</string>
      </array>
    </dict>
    <dict>
      <key>CFBundleTypeName</key>
      <string>Guitar Pro 6 Tablature</string>
      <key>CFBundleTypeRole</key>
      <string>Viewer</string>
      <key>LSHandlerRank</key>
      <string>Alternate</string>
      <key>CFBundleTypeExtensions</key>
      <array>
        <string>gpx</string>
      </array>
    </dict>
  </array>
  <key>UTImportedTypeDeclarations</key>
  <array>
    <dict>
      <key>UTTypeIdentifier</key>
      <string>com.arobas-music.guitarpro.document</string>
      <key>UTTypeDescription</key>
      <string>Guitar Pro document</string>
      <key>UTTypeConformsTo</key>
      <array>
        <string>public.data</string>
      </array>
      <key>UTTypeTagSpecification</key>
      <dict>
        <key>public.filename-extension</key>
        <array>
          <string>gp</string>
        </array>
      </dict>
    </dict>
    <dict>
      <key>UTTypeIdentifier</key>
      <string>com.arobas-music.guitarpro3.document</string>
      <key>UTTypeDescription</key>
      <string>Guitar Pro 3 document</string>
      <key>UTTypeConformsTo</key>
      <array>
        <string>public.data</string>
      </array>
      <key>UTTypeTagSpecification</key>
      <dict>
        <key>public.filename-extension</key>
        <array>
          <string>gp3</string>
        </array>
      </dict>
    </dict>
    <dict>
      <key>UTTypeIdentifier</key>
      <string>com.arobas-music.guitarpro4.document</string>
      <key>UTTypeDescription</key>
      <string>Guitar Pro 4 document</string>
      <key>UTTypeConformsTo</key>
      <array>
        <string>public.data</string>
      </array>
      <key>UTTypeTagSpecification</key>
      <dict>
        <key>public.filename-extension</key>
        <array>
          <string>gp4</string>
        </array>
      </dict>
    </dict>
    <dict>
      <key>UTTypeIdentifier</key>
      <string>com.arobas-music.guitarpro5.document</string>
      <key>UTTypeDescription</key>
      <string>Guitar Pro 5 document</string>
      <key>UTTypeConformsTo</key>
      <array>
        <string>public.data</string>
      </array>
      <key>UTTypeTagSpecification</key>
      <dict>
        <key>public.filename-extension</key>
        <array>
          <string>gp5</string>
        </array>
      </dict>
    </dict>
    <dict>
      <key>UTTypeIdentifier</key>
      <string>com.arobas-music.guitarpro6.document</string>
      <key>UTTypeDescription</key>
      <string>Guitar Pro 6 document</string>
      <key>UTTypeConformsTo</key>
      <array>
        <string>public.data</string>
      </array>
      <key>UTTypeTagSpecification</key>
      <dict>
        <key>public.filename-extension</key>
        <array>
          <string>gpx</string>
        </array>
      </dict>
    </dict>
  </array>
</dict>
</plist>
EOF

# Bind Info.plist into the bundle signature so Launch Services reads document types reliably.
codesign -s - --force --deep "$bundle" >/dev/null

printf 'Created %s\n' "$bundle"

install_bundle() {
  dest=$1
  rm -rf "$dest"
  mv "$bundle" "$dest"
  bundle=$dest
  "$lsregister" -f -R "$dest"
  printf 'Installed %s\n' "$dest"
}

if [ "$install_after" -eq 1 ]; then
  install_bundle "/Applications/$app_name.app"
else
  printf 'Install: %s --install\n' "$0"
  printf '  (moves dist/Ruxguitar.app to /Applications/Ruxguitar.app)\n'
fi

if [ "$open_after" -eq 1 ]; then
  open "$bundle"
fi
