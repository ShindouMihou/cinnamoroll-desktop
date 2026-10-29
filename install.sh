#!/bin/sh
set -eu

REPOSITORY=ShindouMihou/cinnamoroll-desktop
VERSION=
ARCHIVE=
CHECKSUMS=
INSTALL_DIR=/Applications
LAUNCH=1

fail() { printf 'Cinnamoroll: %s\n' "$*" >&2; exit 1; }
usage() { printf '%s\n' 'Usage: install.sh [--version VERSION] [--install-dir DIRECTORY] [--no-launch]' 'Offline verification: --archive ZIP --checksums SHA256SUMS --version VERSION'; }
while [ "$#" -gt 0 ]; do
    case "$1" in
        --version|--install-dir|--archive|--checksums)
            [ "$#" -ge 2 ] || fail "Missing value for $1"
            case "$1" in
                --version) VERSION=$2 ;;
                --install-dir) INSTALL_DIR=$2 ;;
                --archive) ARCHIVE=$2 ;;
                --checksums) CHECKSUMS=$2 ;;
            esac
            shift 2 ;;
        --no-launch) LAUNCH=0; shift ;;
        --help) usage; exit 0 ;;
        *) fail "Unknown option: $1" ;;
    esac
done

[ "$(uname -s)" = Darwin ] || fail 'This installer is for macOS. On Windows, use install.ps1.'
[ "$(sw_vers -productVersion | cut -d. -f1)" -ge 13 ] || fail 'macOS 13 or later is required.'
if [ "$(sysctl -n hw.optional.arm64 2>/dev/null || printf 0)" = 1 ]; then
    TARGET=aarch64-apple-darwin
else
    TARGET=x86_64-apple-darwin
fi
TEMP=$(mktemp -d)
STAGE=
BACKUP=
DESTINATION=
cleanup() {
    if [ -n "$BACKUP" ] && [ -d "$BACKUP/Cinnamoroll.app" ] && [ ! -e "$DESTINATION" ]; then
        if ! mv "$BACKUP/Cinnamoroll.app" "$DESTINATION"; then
            printf 'Your previous app is preserved at %s\n' "$BACKUP/Cinnamoroll.app" >&2
        fi
    fi
    [ -z "$STAGE" ] || rm -rf "$STAGE"
    rm -rf "$TEMP"
}
trap cleanup EXIT
trap 'exit 1' HUP INT TERM

fetch() {
    curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \
        --connect-timeout 15 --max-time 180 --max-filesize 134217728 "$1" -o "$2"
}
if [ -z "$VERSION" ]; then
    [ -z "$ARCHIVE" ] || fail 'Offline installation needs --version.'
    fetch "https://api.github.com/repos/$REPOSITORY/releases/latest" "$TEMP/release.json" || fail 'No published release is available yet. Check the Releases page.'
    VERSION=$(plutil -extract tag_name raw -o - "$TEMP/release.json")
fi
VERSION=${VERSION#v}
printf '%s' "$VERSION" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' || fail 'A stable VERSION such as 0.1.1 is required.'
ASSET="Cinnamoroll-$VERSION-$TARGET.zip"
BASE="https://github.com/$REPOSITORY/releases/download/v$VERSION"
if [ -z "$ARCHIVE" ]; then
    ARCHIVE="$TEMP/$ASSET"
    printf 'Fetching Cinnamoroll %s...\n' "$VERSION"
    fetch "$BASE/$ASSET" "$ARCHIVE" || fail 'Could not download the app.'
fi
if [ -z "$CHECKSUMS" ]; then
    CHECKSUMS="$TEMP/SHA256SUMS"
    fetch "$BASE/SHA256SUMS" "$CHECKSUMS" || fail 'Could not download release checksums.'
fi
[ -f "$ARCHIVE" ] && [ -f "$CHECKSUMS" ] || fail 'The archive or checksums file is missing.'
EXPECTED=$(awk -v asset="$ASSET" '$2 == asset { value=$1; count++ } END { if(count != 1) exit 1; print value }' "$CHECKSUMS") || fail 'The release does not have exactly one checksum for this app.'
printf '%s' "$EXPECTED" | grep -Eq '^[0-9a-fA-F]{64}$' || fail 'Invalid release checksum.'
ACTUAL=$(shasum -a 256 "$ARCHIVE" | awk '{print $1}')
[ "$(printf '%s' "$EXPECTED" | tr A-F a-f)" = "$ACTUAL" ] || fail 'Checksum mismatch. Nothing was installed.'
unzip -Z1 "$ARCHIVE" | awk '/(^\/|(^|\/)\.\.($|\/)|\\|:)/ { bad=1 } END { exit bad }' || fail 'The archive contains unsafe paths.'
unzip -Z -l "$ARCHIVE" | awk '/^l/ { bad=1 } END { exit bad }' || fail 'The archive contains unsupported symbolic links.'
if [ "$INSTALL_DIR" = /Applications ] && [ ! -w /Applications ]; then
    INSTALL_DIR="$HOME/Applications"
    printf 'Using ~/Applications so automatic updates can stay prompt-free.\n'
fi
mkdir -p "$INSTALL_DIR"
[ -w "$INSTALL_DIR" ] || fail 'Choose a writable Applications directory with --install-dir.'
INSTALL_DIR=$(CDPATH= cd -- "$INSTALL_DIR" && pwd -P)
DESTINATION="$INSTALL_DIR/Cinnamoroll.app"
for pid in $(pgrep -x Cinnamoroll 2>/dev/null || true); do
    RUNNING=$(ps -p "$pid" -o comm= 2>/dev/null || true)
    [ "$RUNNING" != "$DESTINATION/Contents/MacOS/Cinnamoroll" ] || fail 'Quit the installed Cinnamoroll before running this installer again.'
done
STAGE=$(mktemp -d "$INSTALL_DIR/.cinnamoroll-install.XXXXXX")
unzip -q "$ARCHIVE" -d "$STAGE"
APP="$STAGE/Cinnamoroll.app"
[ -f "$APP/Contents/MacOS/Cinnamoroll" ] || fail 'The download is missing Cinnamoroll.app.'
codesign --verify --deep --strict "$APP" || fail 'The app failed its code-signature check.'
[ "$("$APP/Contents/MacOS/Cinnamoroll" --version)" = "Cinnamoroll $VERSION" ] || fail 'The app does not match the release version.'
if [ -e "$DESTINATION" ]; then
    [ -f "$DESTINATION/Contents/MacOS/Cinnamoroll" ] || fail 'The destination contains an unrecognized app; it was left untouched.'
    BACKUP=$(mktemp -d "$INSTALL_DIR/.cinnamoroll-previous.XXXXXX")
    mv "$DESTINATION" "$BACKUP/Cinnamoroll.app"
fi
mv "$APP" "$DESTINATION" || fail 'Could not install the new app. Restoring the previous copy.'
[ -z "$BACKUP" ] || rm -rf "$BACKUP"
BACKUP=
printf 'Cinnamoroll %s is ready at %s\n' "$VERSION" "$DESTINATION"
if [ "$LAUNCH" = 1 ]; then
    if pgrep -x Cinnamoroll >/dev/null 2>&1; then
        printf 'Quit the running copy, then open the installed app when ready.\n'
    else
        open "$DESTINATION"
    fi
fi
