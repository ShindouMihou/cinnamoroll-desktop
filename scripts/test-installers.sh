#!/bin/sh
set -eu
[ "$#" = 1 ] || { printf '%s\n' 'Usage: test-installers.sh SIGNED_MAC_ZIP' >&2; exit 1; }
ARCHIVE=$(CDPATH= cd -- "$(dirname -- "$1")" && pwd)/$(basename -- "$1")
ASSET=$(basename -- "$ARCHIVE")
VERSION=$(printf '%s' "$ASSET" | sed -E 's/^Cinnamoroll-([0-9]+\.[0-9]+\.[0-9]+)-.*$/\1/')
INSTALLER=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)/install.sh
FIXTURE=$(mktemp -d)
trap 'rm -rf "$FIXTURE"' EXIT
DESTINATION="$FIXTURE/Applications with spaces"
CHECKSUMS="$FIXTURE/SHA256SUMS"
HASH=$(shasum -a 256 "$ARCHIVE" | awk '{print $1}')
printf '%s  %s\n' "$HASH" "$ASSET" > "$CHECKSUMS"
"$INSTALLER" --version "$VERSION" --archive "$ARCHIVE" --checksums "$CHECKSUMS" --install-dir "$DESTINATION" --no-launch
SENTINEL="$DESTINATION/Cinnamoroll.app/preserve-on-failure.txt"
printf keep > "$SENTINEL"
printf '%064d  %s\n' 0 "$ASSET" > "$CHECKSUMS"
if "$INSTALLER" --version "$VERSION" --archive "$ARCHIVE" --checksums "$CHECKSUMS" --install-dir "$DESTINATION" --no-launch; then
    printf 'The installer accepted a bad checksum.\n' >&2; exit 1
fi
[ -f "$SENTINEL" ]
printf '%s  %s\n' "$HASH" "$ASSET" > "$CHECKSUMS"
"$INSTALLER" --version "$VERSION" --archive "$ARCHIVE" --checksums "$CHECKSUMS" --install-dir "$DESTINATION" --no-launch
[ ! -e "$SENTINEL" ]
python3 - "$FIXTURE/unsafe.zip" <<'PY'
import sys, zipfile
with zipfile.ZipFile(sys.argv[1], 'w') as archive:
    archive.writestr('../outside.txt', 'must not escape')
PY
UNSAFE_HASH=$(shasum -a 256 "$FIXTURE/unsafe.zip" | awk '{print $1}')
printf '%s  %s\n' "$UNSAFE_HASH" "$ASSET" > "$CHECKSUMS"
if "$INSTALLER" --version "$VERSION" --archive "$FIXTURE/unsafe.zip" --checksums "$CHECKSUMS" --install-dir "$DESTINATION" --no-launch; then
    printf 'The installer accepted an unsafe archive.\n' >&2; exit 1
fi
[ ! -e "$FIXTURE/outside.txt" ]
printf 'Mac installer: install, replace, checksum rejection, and traversal checks passed.\n'
