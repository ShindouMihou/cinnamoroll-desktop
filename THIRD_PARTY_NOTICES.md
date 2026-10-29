# Third-party material

The character Cinnamoroll belongs to Sanrio. The character artwork is supplied by the user from their personal ChatGPT pet collection and is not covered by the source-code package's MIT license declaration.

Cinnamoroll character references: https://corporate.sanrio.co.jp/en/business-info/brands/cinnamoroll/

This is an independent fan project and is not affiliated with or endorsed by Sanrio.

Direct dependency licenses:

| Component | License |
| --- | --- |
| SDL3 | zlib |
| self_update | MIT |
| zipsign-api / ed25519-dalek | MIT OR Apache-2.0 / BSD-3-Clause |
| ureq | MIT OR Apache-2.0 |
| rustls | MIT OR Apache-2.0 OR ISC |
| tempfile / semver | MIT OR Apache-2.0 |
| sdl3-rs | MIT |
| tiny-skia | BSD-3-Clause |
| Serde / serde_json | MIT OR Apache-2.0 |
| fastrand | MIT OR Apache-2.0 |
| Chrono | MIT OR Apache-2.0 |
| dirs | MIT OR Apache-2.0 |
| windows-rs | MIT OR Apache-2.0 |

`Cargo.lock` records the complete resolved Rust dependency set. `packaging/licenses/` contains upstream license files and a package index; the packagers include those files with the application. This table is a direct-dependency summary.

## Settings typography

Geist font from Google Fonts / Vercel, under the SIL Open Font License 1.1. The font and license are included in assets/ and packaging/licenses/Geist-OFL.txt. fontdue and its Rust dependencies are covered by the accompanying collected license files.

## Dedicated animation strips

Snack, nap and listening source sheets were generated with the built-in image_gen tool from the user-provided Cinnamoroll reference. The prompts are preserved in assets/strips/prompts.json and assets/strips/expansion-prompts.json and assets/strips/reactions-chase-prompts.json and assets/strips/rest-hanging-prompts.json and assets/strips/student-playroom-prompts.json and assets/strips/companion-expansion-prompts.json. The original validated atlas remains unchanged. The prepared sheets are embedded; source sheets are retained for revision.

The Windows SDL3.dll is from the official SDL 3.4.18 MinGW release archive, SHA-256 cd98158c8d025a816f430a600b4e9526524e6219ee884ec72bd498b80fd3bf58.
