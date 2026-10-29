#!/bin/sh
set -eu
PROJECT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$PROJECT_DIR"
command -v x86_64-w64-mingw32-gcc >/dev/null
SDK="$PROJECT_DIR/.tools/sdl3-windows"
ARCHIVE="$SDK/SDL3-devel-3.4.18-mingw.zip"
mkdir -p "$SDK"
if [ ! -f "$ARCHIVE" ]; then
    curl --fail --location https://github.com/libsdl-org/SDL/releases/download/release-3.4.18/SDL3-devel-3.4.18-mingw.zip --output "$ARCHIVE"
fi
printf '%s  %s\n' cd98158c8d025a816f430a600b4e9526524e6219ee884ec72bd498b80fd3bf58 "$ARCHIVE" | shasum -a 256 --check
unzip -p "$ARCHIVE" SDL3-3.4.18/x86_64-w64-mingw32/lib/libSDL3.dll.a > "$SDK/libSDL3.dll.a"
unzip -p "$ARCHIVE" SDL3-3.4.18/x86_64-w64-mingw32/bin/SDL3.dll > "$SDK/SDL3.dll"
CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc SDL3_LIB_DIR="$SDK" ./scripts/cargo.sh build --release --locked --target x86_64-pc-windows-gnu --features updater "$@"
DESTINATION="$PROJECT_DIR/dist/Cinnamoroll-Windows"
mkdir -p "$DESTINATION"
cp target/x86_64-pc-windows-gnu/release/cinnamoroll-desktop.exe "$DESTINATION/Cinnamoroll.exe"
cp "$SDK/SDL3.dll" README.md LICENSE THIRD_PARTY_NOTICES.md "$DESTINATION/"
cp packaging/Windows-Readme.txt "$DESTINATION/Read-me-first.txt"
cp scripts/test-windows-package.ps1 "$DESTINATION/Check-Windows.ps1"
cp -R packaging/licenses docs "$DESTINATION/"
printf 'Built %s\n' "$DESTINATION"
