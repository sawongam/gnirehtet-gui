# Changelog

All notable changes to this project are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the
project follows [Semantic Versioning](https://semver.org/).

## [Unreleased]

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

[Unreleased]: https://github.com/sawongam/gnirehtet-gui/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/sawongam/gnirehtet-gui/releases/tag/v0.1.0
