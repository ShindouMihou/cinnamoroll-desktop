# Development

The desktop companion is a native Rust/SDL3 application. Use Rust 1.89 or later, a C/C++ toolchain, and CMake for bundled SDL builds.

## Run locally

On macOS, install the Xcode command-line tools and SDL3, then run:

```sh
brew install sdl3
./scripts/cargo.sh run --features updater
```

The wrapper uses a local Rust toolchain if present, otherwise the system `cargo`. The updater reads the public channel in `packaging/update-channel.json`; private signing keys never belong in this checkout.

On Windows, install Rust MSVC, Visual Studio C++ Build Tools, and CMake:

```powershell
cargo run --features bundled-sdl,updater
```

## Make a portable app

```sh
./scripts/bundle-macos.sh
```

The Mac bundle builds SDL3 from source with a macOS 13 deployment target, includes the dynamic library, and applies an ad-hoc signature. It builds for the current architecture. A separate Intel build is needed for Intel Macs.

```powershell
./scripts/bundle-windows.ps1
```

This builds SDL3 from source and packages `Cinnamoroll.exe` with `SDL3.dll`. A native Windows launch check is available at `scripts/test-windows-package.ps1` inside the package.

For a Windows GNU cross-build from macOS, install MinGW-w64 and the Rust `x86_64-pc-windows-gnu` target, then run `./scripts/bundle-windows-cross.sh`. This is useful for development; the release workflow builds and smoke-tests Windows natively.

## Check a change

```sh
./scripts/cargo.sh fmt --check
./scripts/cargo.sh clippy --all-targets --features updater -- -D warnings
./scripts/cargo.sh test --features updater
```

Tests cover behavior pacing, movement and bounds, drag anchors, games, animation preparation, settings compatibility, update signatures, and update-folder rollback. Actual mixed-DPI dragging, media detection, accessibility support, and update/relaunch behavior also need native platform checks.

## Layout

| Area | Responsibility |
| --- | --- |
| `src/app/` | Pet state, input, movement, and command coordination |
| `src/behavior/` | Weighted idle states, durations, and cooldowns |
| `src/art/` | Sprite strips, frame composition, props, and attachment anchors |
| `src/companion/` | Study, touch interactions, toys, cozy corners, and walking home |
| `src/pocket/` | Direct interaction menu |
| `src/preferences/` | Native settings and control pages |
| `src/scene/` | Toy and bed windows, placement, and dragging |
| `src/native/` | Platform bridges, tray, startup registration, and window order |
| `src/updates/` | Checks, signatures, idle policy, validation, and installation |
| `assets/strips/` | Source artwork, prompts, and prepared runtime strips |
| `packaging/` | Bundle metadata, public update channel, and license copies |

The supplied atlas remains available for directional artwork. Dedicated strips use eight frames and prepared runtime cells of 320 × 224 pixels. Walking advances with actual travelled distance, and accessories use per-frame attachment anchors. Keep new animations consistent with existing scale, ground alignment, stroke color, and pacing.

## Local data

Preferences are under `~/Library/Application Support/cinnamoroll-desktop/` on macOS and `%APPDATA%/cinnamoroll-desktop/` on Windows. Launch-at-login registration is read from the OS. Update caches and check timestamps live beside preferences, outside the app installation. Sticker exports use `Pictures/Cinnamoroll` with a Documents fallback.

## Useful checks and previews

```sh
./scripts/cargo.sh run --release -- --gallery work/gallery
./scripts/cargo.sh run --release -- --settings-gallery work/settings
./scripts/cargo.sh run --release -- --pocket-gallery work/pocket
./scripts/cargo.sh run --release -- --play-gallery work/play
./scripts/cargo.sh run --release -- --smoke-seconds 5
./scripts/cargo.sh run --release -- --startup-status
./scripts/cargo.sh run --release --features updater -- --update-config
```

Generated previews under `work/` are not source artifacts. README images under `docs/assets/` use actual renderer output and retain the approved character artwork.
