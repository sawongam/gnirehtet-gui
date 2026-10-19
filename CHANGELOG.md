# Changelog

All notable changes to this project are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [1.0.0]

First stable release.

### Added

- Link to the project's GitHub page under **Preferences → About**.
- The installers include the `LICENSE` and `NOTICE` files.

### Changed

- Redesigned main window: a single connection screen with a phone illustration
  that shows the current state, one primary action at a time, and a phone
  picker that appears only when more than one phone is plugged in, replacing
  the Phones list. The log and Preferences open as slide-in panels instead of
  separate pages.
- The adb, relay and APK paths in use are written to the first line of the log
  instead of a Diagnostics section in Preferences.
- Building from source uses pnpm instead of npm.
- Release files are named after the system they are for, such as
  `gnirehtet-gui-<version>-windows-x64-setup.exe` and
  `gnirehtet-gui-<version>-macos-apple-silicon.dmg`, and the release notes start
  with a Downloads table.
- Windows is shipped as the `-setup.exe` installer only; the `.msi` and the
  macOS `.app.tar.gz` archives are no longer published. Checksums are in
  `SHA256SUMS.txt`.

## [0.1.0]

First public release.

### Added

- Desktop app for Linux, Windows and macOS that shares the computer's internet
  connection with an Android phone over USB, using gnirehtet v2.5.1.
- Device list with guidance for unauthorized, offline and permission-less
  (Linux udev) devices.
- One-click connect: starts the relay, installs the Android client when needed,
  creates the adb reverse tunnel and starts the VPN.
- Live connection status: waiting for the VPN prompt, connected, interrupted,
  relay stopped.
- Automatic tunnel restore when the phone is plugged back in.
- Clean shutdown: disconnecting or quitting stops the VPN on the phone and the
  relay on the computer; a relay not started by the app is never touched.
- Settings for relay port, DNS servers and routes; light, dark and system
  themes; log view with copy; diagnostics showing the adb, relay and APK in use.
- `ADB`, `GNIREHTET_BIN` and `GNIREHTET_APK` environment overrides.
- Installers: `.deb`, `.rpm`, AppImage, `.msi`, NSIS `.exe` and `.dmg`, built by
  the release workflow.

[Unreleased]: https://github.com/sawongam/gnirehtet-gui/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/sawongam/gnirehtet-gui/compare/v0.1.0...v1.0.0
[0.1.0]: https://github.com/sawongam/gnirehtet-gui/releases/tag/v0.1.0
