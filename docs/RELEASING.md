# Packing a little release

The release workflow builds Apple Silicon Mac, Intel Mac, and Windows x64 packages, runs native checks, signs ZIPs with zipsign, verifies the extracted app, and creates a **draft** GitHub Release. Publishing remains an explicit final step.

## Channel and keys

`packaging/update-channel.json` is public. It names the GitHub repository and contains a 32-byte Ed25519 public key encoded as 64 hex characters. `build.rs` embeds those values; release bundles enable the optional updater transport. Build-time `CINNAMOROLL_UPDATE_REPOSITORY` and `CINNAMOROLL_UPDATE_PUBLIC_KEY` can override them for separate test channels.

The private key stays outside the checkout. Keep an offline backup: losing it means existing installations cannot trust future releases signed by a new key. The GitHub Actions secret `CINNAMOROLL_UPDATE_SIGNING_KEY` holds the base64 encoding of the raw 64-byte zipsign keypair. The workflow decodes it into the runner’s temporary directory, checks its public half, signs/verifies the packages, and removes the temporary key file even when a step fails. No private key belongs in source, artifacts, logs, or release assets.

Generating a new channel, if needed for a fork:

```sh
cargo install zipsign --version 0.2.1 --locked
zipsign gen-key /secure/location/update-private.key /secure/location/update-public.key
chmod 600 /secure/location/update-private.key
```

Update the public JSON and configure the matching repository secret before building. Do not replace the established public key casually; installed versions still trust the previous key.

## Before the first draft

1. Review the README, screenshots, source-code license, and third-party notices.
2. Confirm Cargo.toml’s version and `docs/releases/vVERSION.md` agree.
3. Commit reviewed source and create the corresponding `vVERSION` tag, only with the project owner’s explicit authorization.
4. Push the source and tag through the project’s Git/GitHub approval process.
5. Configure the signing secret through that same approval process.
6. Dispatch **Pack a little release** on the main branch, with the existing tag as its input.

The workflow accepts only a tag matching Cargo.toml and checks out that tag. Third-party Actions are pinned to commit SHAs, build jobs have read-only repository access, and only the final draft job can write release contents. That job refuses to overwrite a published release. Pull-request checks never receive the signing key.

## Local preparation

Install Python 3.11+, Rust, CMake, and the native C/C++ toolchain, then:

```sh
python3 scripts/collect-licenses.py
./scripts/bundle-macos.sh
python3 scripts/prepare-release.py --target aarch64-apple-darwin --key /secure/location/update-private.key
```

On Windows:

```powershell
python scripts/collect-licenses.py
./scripts/bundle-windows.ps1
python scripts/prepare-release.py --target x86_64-pc-windows --key C:\secure\update-private.key
```

Use `x86_64-apple-darwin` on an Intel Mac. The packager checks the executable’s embedded version, repository, public key, and enabled transport. It signs the final filename, verifies the signature, checks ZIP integrity, extracts it, and asks the extracted executable for its version. Mac code signatures are verified too. It refuses to overwrite an existing asset.

`--skip-launch-check` supports local Windows cross-build preparation, but does not count as native validation. Do not publish an untested cross-build in place of the workflow’s Windows package.

`--source-only` makes a curated source ZIP excluding work files, caches, Git metadata, local build output, and private-key files. `scripts/sign-update.sh` remains available for signing an already-packaged ZIP. Never rename a signed archive: the signature includes its final filename.

## Release contract

| Platform | Signed asset | Archive root |
| --- | --- | --- |
| Apple Silicon | `Cinnamoroll-VERSION-aarch64-apple-darwin.zip` | `Cinnamoroll.app/` |
| Intel Mac | `Cinnamoroll-VERSION-x86_64-apple-darwin.zip` | `Cinnamoroll.app/` |
| Windows x64 | `Cinnamoroll-VERSION-x86_64-pc-windows.zip` | `Cinnamoroll-Windows/` |

The tag must be `vVERSION`, using a stable semantic version newer than the installed version. The draft also includes a source ZIP and SHA256SUMS. Checksums provide a download-integrity check; the embedded public-key signature is the update-authenticity check.

## Review, then publish

Confirm all three platform jobs passed and the draft has every expected asset. Install the portable packages on their intended computers and exercise dragging, settings, the pocket, media reactions, and startup registration. Validate a real update and relaunch between two versions before depending on unattended updates. OS-signing/notarization remains separate from archive signing.

Publish the reviewed draft only with explicit approval. Drafts and prereleases are not offered to the stable updater. If no stable release exists yet, the app shows that the first release is being prepared.

## One-line installers

`install.sh` and `install.ps1` select only the latest published stable release. They verify the exact asset’s SHA-256 against SHA256SUMS, reject unsafe archive paths, confirm the extracted app’s version, and stage replacement beside the destination. Failed replacement restores the previous app. Preferences are not stored in that folder and remain unchanged.

Windows defaults to `%LOCALAPPDATA%\Programs\Cinnamoroll` so installations and idle updates work without UAC, and adds a per-user Start Menu shortcut. Mac defaults to `/Applications`, falling back to `~/Applications` if the system folder is not writable. A running copy in the target location must be closed before reinstalling. Startup registration remains an explicit app setting on fresh installations.

Offline test options take a local archive, checksums file, and explicit version. Use `--no-launch` on Mac or `-NoLaunch -NoShortcuts` on Windows for isolated installation checks. The checksum protects the initial download’s integrity; HTTPS GitHub hosting is the bootstrap trust source, and subsequent app updates additionally require the embedded Ed25519 signature.
