#!/bin/sh
set -eu
PROJECT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$PROJECT_DIR"
if [ "$#" -ne 4 ]; then
    echo 'Usage: scripts/sign-update.sh TARGET INPUT_ZIP PRIVATE_KEY PUBLIC_KEY' >&2
    exit 1
fi
TARGET=$1
case "$TARGET" in
    aarch64-apple-darwin|x86_64-apple-darwin) REQUIRED='Cinnamoroll.app/Contents/MacOS/Cinnamoroll' ;;
    x86_64-pc-windows) REQUIRED='Cinnamoroll-Windows/Cinnamoroll.exe' ;;
    *) echo 'Unsupported update target.' >&2; exit 1 ;;
esac
PRIVATE_DIR=$(CDPATH= cd -- "$(dirname -- "$3")" && pwd -P)
case "$PRIVATE_DIR/" in "$PROJECT_DIR/"*) echo 'Keep the private signing key outside the project.' >&2; exit 1 ;; esac
command -v zipsign >/dev/null
unzip -Z1 "$2" | rg -Fx "$REQUIRED" >/dev/null
VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -1)
mkdir -p dist/releases
ASSET="dist/releases/Cinnamoroll-$VERSION-$TARGET.zip"
if [ -e "$ASSET" ]; then
    echo "Refusing to overwrite $ASSET" >&2
    exit 1
fi
cp "$2" "$ASSET"
zipsign sign zip "$ASSET" "$3"
zipsign verify zip "$ASSET" "$4"
printf 'Prepared signed asset: %s\nPublish with tag: v%s\n' "$ASSET" "$VERSION"
