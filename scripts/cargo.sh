#!/bin/sh
set -eu
PROJECT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
if [ -x "$PROJECT_DIR/.tools/cargo/bin/cargo" ]; then
    export CARGO_HOME="$PROJECT_DIR/.tools/cargo"
    export RUSTUP_HOME="$PROJECT_DIR/.tools/rustup"
    export PATH="$CARGO_HOME/bin:$PATH"
fi
if [ -z "${SDL3_LIB_DIR:-}" ] && [ "$(uname -s)" = Darwin ]; then
    for SDL_DIR in /opt/homebrew/opt/sdl3/lib /usr/local/opt/sdl3/lib; do
        if [ -f "$SDL_DIR/libSDL3.dylib" ]; then
            export SDL3_LIB_DIR="$SDL_DIR"
            break
        fi
    done
fi
exec cargo "$@"
