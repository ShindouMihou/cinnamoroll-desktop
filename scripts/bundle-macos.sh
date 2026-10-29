#!/bin/sh
set -eu
PROJECT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$PROJECT_DIR"
export MACOSX_DEPLOYMENT_TARGET=${MACOSX_DEPLOYMENT_TARGET:-13.0}
./scripts/cargo.sh build --release --locked --features updater,bundled-sdl "$@"
BUNDLE="$PROJECT_DIR/dist/Cinnamoroll.app"
mkdir -p "$BUNDLE/Contents/MacOS" "$BUNDLE/Contents/Frameworks" "$BUNDLE/Contents/Resources"
cp target/release/cinnamoroll-desktop "$BUNDLE/Contents/MacOS/Cinnamoroll"
SDL_SOURCE=$(otool -L target/release/cinnamoroll-desktop | awk '/libSDL3.*dylib/ {print $1; exit}')
if [ ! -f "$SDL_SOURCE" ]; then
    SDL_SOURCE=$(ls -t target/release/build/sdl3-sys-*/out/lib/libSDL3.0.dylib 2>/dev/null | head -1)
fi
if [ ! -f "$SDL_SOURCE" ]; then
    echo "Cannot find SDL3 at $SDL_SOURCE. Set SDL3_LIB_DIR to its library directory." >&2
    exit 1
fi
cp "$SDL_SOURCE" "$BUNDLE/Contents/Frameworks/libSDL3.0.dylib"
chmod u+w "$BUNDLE/Contents/Frameworks/libSDL3.0.dylib"
install_name_tool -id @rpath/libSDL3.0.dylib "$BUNDLE/Contents/Frameworks/libSDL3.0.dylib"
install_name_tool -change "$SDL_SOURCE" @rpath/libSDL3.0.dylib "$BUNDLE/Contents/MacOS/Cinnamoroll"
otool -l "$BUNDLE/Contents/MacOS/Cinnamoroll" | awk '/cmd LC_RPATH/ {getline; getline; sub(/^ *path /, ""); sub(/ \(offset.*$/, ""); print}' |
    while IFS= read -r SEARCH_PATH; do
        install_name_tool -delete_rpath "$SEARCH_PATH" "$BUNDLE/Contents/MacOS/Cinnamoroll"
    done
install_name_tool -add_rpath @executable_path/../Frameworks "$BUNDLE/Contents/MacOS/Cinnamoroll"
cp packaging/Info.plist "$BUNDLE/Contents/Info.plist"
VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1)
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $VERSION" "$BUNDLE/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion $VERSION" "$BUNDLE/Contents/Info.plist"
ICONSET="$PROJECT_DIR/work/Cinnamoroll.iconset"
mkdir -p "$ICONSET"
DYLD_LIBRARY_PATH="$(dirname "$SDL_SOURCE")${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}" target/release/cinnamoroll-desktop --icon "$PROJECT_DIR/work/icon.png"
for ICON_SIZE in 16 32 128 256 512; do
    sips -z "$ICON_SIZE" "$ICON_SIZE" "$PROJECT_DIR/work/icon.png" --out "$ICONSET/icon_${ICON_SIZE}x${ICON_SIZE}.png" >/dev/null
    DOUBLE_SIZE=$((ICON_SIZE * 2))
    sips -z "$DOUBLE_SIZE" "$DOUBLE_SIZE" "$PROJECT_DIR/work/icon.png" --out "$ICONSET/icon_${ICON_SIZE}x${ICON_SIZE}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o "$BUNDLE/Contents/Resources/Cinnamoroll.icns"
cp README.md LICENSE THIRD_PARTY_NOTICES.md "$BUNDLE/Contents/Resources/"
mkdir -p "$BUNDLE/Contents/Resources/docs"
cp -R docs/. "$BUNDLE/Contents/Resources/docs/"
if [ -d packaging/licenses ]; then
    cp -R packaging/licenses "$BUNDLE/Contents/Resources/"
fi
codesign --force --deep --sign - "$BUNDLE"
touch "$BUNDLE"
printf 'Built %s\n' "$BUNDLE"
