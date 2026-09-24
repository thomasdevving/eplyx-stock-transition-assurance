# Milestone 17: release binaries and install experience

External developers install one verified `eplyx` binary and use the existing `init`, `doctor`, `preflight`, `search`, `dashboard` and `reproduce` workflow in their own Solana project. They need no Eplyx checkout, Rust, Cargo, Node or npm. Analytical semantics are unchanged: the release binary is the same engine and embedded dashboard validated in Milestones 8–16.

## Platforms and artifacts

| Platform | Target | Runner | Archive |
|---|---|---|---|
| macOS Apple Silicon | `aarch64-apple-darwin` | `macos-14` | `eplyx-v<version>-darwin-arm64.tar.gz` |
| Linux x86_64 (glibc ≥ 2.35) | `x86_64-unknown-linux-gnu` | `ubuntu-22.04` | `eplyx-v<version>-linux-x86_64.tar.gz` |
| Windows x86_64 | `x86_64-pc-windows-msvc` | `windows-2022` | `eplyx-v<version>-windows-x86_64.zip` |

- Each archive contains exactly one member, `eplyx` or `eplyx.exe`, with no owner names or extended attributes.
- A release also publishes `SHA256SUMS` (`sha256sum` format, covering the archives and both installers), `eplyx-release.json`, `install.sh` and `install.ps1`.
- The manifest is schema 1: version, tag, commit, and per artifact the platform, target, file, binary, size and sha256.
- macOS Intel and Linux ARM64 are not built or claimed. Other platforms can build from source.

## Runtime dependencies

None for Eplyx itself:
- Dashboard assets are embedded at compile time.
- HTTPS uses rustls, so there is no OpenSSL.
- The candidate program comes from the validated package bytes.
- The only process the binary spawns is itself, as the isolated offline VM worker, besides an optional `git` for run metadata and the platform's browser opener.

No code path in `eplyx` reads the source tree at runtime. The build-from-a-copy acceptance below and the release workflow's no-checkout install jobs check this.

## Version identity

`engine/build.rs` embeds the source commit (`EPLYX_BUILD_COMMIT` in CI, otherwise `git rev-parse HEAD`) and the target triple. It embeds no timestamp. The semantic version (`Cargo.toml`) is the only compatibility identity.
- `eplyx --version` prints version, commit, target and engine schema versions.
- `eplyx -V` prints the version only.
- `eplyx version --json` adds platform, OS, architecture, package, invariant, search, run-metadata, reproduction and dashboard-index schema versions.

None of these need a project, config, store or network, and they write nothing.

## Installers

`install.sh` (POSIX sh) and `install.ps1` behave the same way:
1. Detect the platform. The sh installer maps Rosetta shells to arm64, and `EPLYX_PLATFORM` overrides detection. Anything outside the supported list is rejected.
2. Download `SHA256SUMS`, then select exactly one line naming this platform's archive; `EPLYX_VERSION` pins the version.
3. Download the archive and verify its SHA-256. A mismatch aborts, and nothing is installed.
4. Require the archive to contain exactly one member, the binary. Anything else is rejected: extra files, `./` or directory prefixes, `../` traversal, a missing binary, or an unreadable archive. After extraction the file must be a regular file, not a link.
5. Run only that verified binary's `--version`, to prove it runs on this system.
6. Copy it into `~/.local/bin` or `%LOCALAPPDATA%\Programs\eplyx\bin`, or `EPLYX_INSTALL_DIR` (which must be absolute).
7. Print, never apply, the PATH command.

Downloads use HTTPS only (`curl --proto =https --tlsv1.2`). `EPLYX_DOWNLOAD_BASE` may point at another https:// release directory, or at an absolute `file://` mirror for offline installs and tests. Temporary files are always removed. The installers need no sudo or Administrator rights, change no profile, PATH or registry entry, and install no service. They send no telemetry, store no credentials and never configure an RPC. Uninstall by deleting the binary. Release signing is deferred; checksum verification is mandatory.

## Doctor

`eplyx doctor` is a checklist, and exits `2` when any item fails.
- **Eplyx:** the installed binary, `eplyx.toml`, the local run store and the RPC. An RPC error never echoes the URL.
- **Your project:** the candidate `.so` and package validation.

It points to `eplyx init` and `cargo build-sbf` where needed. It states that Eplyx itself needs no Rust, Node or checkout. `--offline` skips only the RPC check.

## Release workflow

`.github/workflows/release.yml` runs on a `v*` tag, and on manual dispatch without publishing.
1. **validate** (ubuntu-22.04):
   - tag = `v` + workspace version
   - `make fmt-check`, `make lint`, `make test` with pinned `cargo-build-sbf` 4.4.0
   - `npm run check:frontend`, `test:service`, `test:release`, `test:dashboard`
2. **build** (native macOS, Linux and Windows runners):
   - CLI, dashboard and release-CLI tests
   - release build with `EPLYX_BUILD_COMMIT`
   - archive
   - `scripts/release/smoke.sh` on the binary extracted from that archive
   - installer tests: `test-installers.sh` on Unix, `test-install-ps1.ps1` with the real zip on Windows
3. **assemble**: `SHA256SUMS` and the manifest from the exact uploaded archives, then `sha256sum -c`.
4. **install-acceptance** (three native runners, **no checkout**):
   - the real installer from the assembled bundle
   - `--version`, `version --json`, `init`, `runs`, `doctor --offline`
   - the dashboard answering on loopback
   - a tampered archive rejected
   - uninstall by deletion
5. **publish** (tags only, after all of the above): re-verify checksums, then `gh release create --verify-tag` with the exact tested bundle. Nothing is rebuilt after testing.

## Smoke test

`scripts/release/smoke.sh <archive>` extracts the binary from the archive and runs it outside the repository:
- archive layout
- `--version` and `version --json` with proxies pointing at a closed port and no files written
- `init`, `runs` and `doctor --offline` guidance in a clean project
- `doctor --offline` passing on a real checked-in package
- the Milestone 16 fixture store through `runs`, `show` and the dashboard: four runs, embedded assets, foreign Host rejected, no RPC secret in responses
- an unchanged archive digest at the end

## Compatibility

Release and source-built binaries share `engine/src/local_store.rs` and the dashboard, so `.eplyx/` formats are identical.
- The Milestone 16 fixture store opens with `runs`, `show` and the dashboard.
- A full Milestone 15/16 store reproduces offline with the release binary.
- Canonical artifact references stay forward-slash.
- On Windows, verbatim `\\?\C:\…` canonical paths keep their equality checks but are shown, and passed to `git`, in plain form.
- `.gitattributes` stops checkouts from rewriting line endings in pinned evidence and fixtures.
