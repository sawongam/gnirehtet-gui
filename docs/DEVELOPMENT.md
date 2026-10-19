# Development

## Prerequisites

- **Rust 1.88.** `rust-toolchain.toml` pins it, so [rustup](https://rustup.rs)
  installs it automatically.
- **Node.js 20.19+ or 22.12+** with npm.
- **adb**, and an Android phone for manual testing.
- **System libraries for Tauri:**

| System | Command |
|--------|---------|
| Debian, Ubuntu | `sudo apt install build-essential curl file libwebkit2gtk-4.1-dev librsvg2-dev libssl-dev` |
| Fedora | `sudo dnf install webkit2gtk4.1-devel librsvg2-devel openssl-devel && sudo dnf group install c-development` |
| Arch Linux | `sudo pacman -S --needed base-devel webkit2gtk-4.1 librsvg openssl` |
| macOS | `xcode-select --install` |
| Windows | [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) ("Desktop development with C++") and WebView2 (preinstalled on Windows 10/11) |

For more detail, see Tauri's [prerequisites guide](https://v2.tauri.app/start/prerequisites/).

## First build

```bash
git clone https://github.com/sawongam/gnirehtet-gui.git
cd gnirehtet-gui/apps/desktop
npm install
npm run setup
npm run tauri dev
```

`npm run setup` runs `scripts/fetch-upstream.mjs`. It places the pinned gnirehtet
v2.5.1 relay and APK where the build expects them, after verifying their
SHA-256 checksums. Downloads are cached in `target/upstream-cache`. The build
fails without these files, because Tauri checks for the bundled relay at compile
time.

The script accepts these options:

| Option | Effect |
|--------|--------|
| `--target <triple>` | Prepare the relay for another Rust target, e.g. `x86_64-pc-windows-msvc`. |
| `--from-source` | Build the relay from `crates/gnirehtet-relay` instead of downloading it. This is automatic for targets without an upstream binary. |
| `--force` | Overwrite existing files. |

## Project layout

```
apps/desktop/                 Tauri application
  src/                        SvelteKit frontend
    lib/app.svelte.ts         application state and connection phases
    lib/api.ts                typed wrappers around the Tauri commands
    lib/components/           views; ui/ holds the small component library
    routes/+page.svelte       main window
  src-tauri/                  Rust backend
    src/session.rs            session lifecycle (relay, client, cleanup)
    src/commands.rs           Tauri commands
    tauri.conf.json           bundle and security configuration
crates/
  gnirehtet-adb/              adb client
  gnirehtet-controller/       relay process supervisor
  gnirehtet-relay/            vendored upstream relay (do not modify)
  gnirehtet-cli/              upstream command-line tool
resources/                    gnirehtet.apk (fetched, not committed)
scripts/fetch-upstream.mjs    prepares the bundled relay and APK
docs/                         documentation
```

For how the pieces fit together, see [ARCHITECTURE.md](ARCHITECTURE.md).

## Checks

Run these before sending a change. CI runs the same ones.

```bash
# Rust (from the repository root)
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Frontend (from apps/desktop)
npm run check   # svelte-check and TypeScript
npm test        # vitest
```

The Rust tests do not need a phone. They use small shell scripts that stand in
for `adb` and the relay, so they run on Linux and macOS; on Windows the
process-level tests are skipped. To also test against the real relay binary
(the path must be absolute):

```bash
GNIREHTET_TEST_RELAY="$PWD/apps/desktop/src-tauri/binaries/gnirehtet-relay-x86_64-unknown-linux-gnu" \
  cargo test -p gnirehtet-controller
```

## Manual test checklist

Before a release, go through these with a real phone:

- [ ] First connection on a phone without the client: it installs, the VPN
      prompt appears, the status reaches **Connected**, and pages load on the
      phone.
- [ ] **Disconnect**: the VPN key icon disappears from the phone.
- [ ] Unplug while connected: **Connection interrupted**. Plug back in: the
      tunnel is restored and the phone is online again.
- [ ] Quit while connected: the confirmation dialog appears, and the VPN stops
      on the phone.
- [ ] Unauthorized phone (revoke USB debugging authorizations): a clear message
      appears, and the phone becomes ready after accepting the prompt.
- [ ] Two phones connected: a device can be selected and connected.
- [ ] `gnirehtet relay` already running in a terminal: **Connect** reports that
      the port is in use and leaves the other relay alone.
- [ ] adb missing from `PATH`: the setup checklist explains how to install it.
- [ ] Custom DNS servers and port from Settings are applied.

## Building installers

```bash
cd apps/desktop
npm run tauri build
```

Bundles are written to `target/release/bundle/`. Build each platform on that
platform, or use the release workflow.

On Arch Linux and other distributions with very recent toolchains, the AppImage
step can fail with `failed to run linuxdeploy`. linuxdeploy's bundled `strip`
does not understand newer ELF sections. Build with `NO_STRIP=true npm run tauri
build`. This does not affect the `.deb` and `.rpm`, and CI builds on Ubuntu
22.04, where the problem does not occur.

## Releasing

Pull requests run the **CI** workflow. Every push to `master` runs CI again and
then the **Release** workflow, which publishes a release only when the app
version has no `v<version>` tag yet. To release:

1. In a pull request, bump the version everywhere it appears:
   - `apps/desktop/src-tauri/tauri.conf.json`
   - `apps/desktop/src-tauri/Cargo.toml` (run `cargo check` to update
     `Cargo.lock`)
   - `apps/desktop/package.json` (run `npm install` to update
     `package-lock.json`)
2. Move the *Unreleased* entries in `CHANGELOG.md` under a new
   `## [x.y.z]` heading. That section becomes the release notes.
3. Run the manual test checklist, then merge.

`node scripts/release-info.mjs` checks steps 1 and 2 locally. CI runs the same
check. After the merge, the Release workflow builds the Linux, Windows and
macOS installers, checks they are all present, attaches `SHA256SUMS`, creates
the tag and publishes the release. If a build fails, fix it and merge again,
or re-run the workflow. The leftover draft is replaced.

## Updating gnirehtet

The relay and APK are pinned to upstream v2.5.1. To move to a new upstream
release:

1. Update `VERSION`, the archive checksums and `APK_SHA256` in
   `scripts/fetch-upstream.mjs`.
2. Replace `crates/gnirehtet-relay/src` with the new upstream `relay-rust/src`
   and update its `Cargo.toml` version.
3. Check `crates/gnirehtet-adb` against upstream `adb_monitor.rs` and the
   `main.rs` command functions for protocol changes, such as new intent extras.
4. Run the manual test checklist.

Keep `crates/gnirehtet-relay` identical to upstream. Send fixes to
[Genymobile/gnirehtet](https://github.com/Genymobile/gnirehtet).
