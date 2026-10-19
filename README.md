# Gnirehtet GUI: reverse tethering for Android over USB

[![Latest release](https://img.shields.io/github/v/release/sawongam/gnirehtet-gui)](https://github.com/sawongam/gnirehtet-gui/releases/latest)
[![CI](https://github.com/sawongam/gnirehtet-gui/actions/workflows/ci.yml/badge.svg)](https://github.com/sawongam/gnirehtet-gui/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/sawongam/gnirehtet-gui)](LICENSE)

Use your computer's internet connection on your Android phone through a USB
cable. No root, no Wi-Fi hotspot, nothing to configure on the phone. Runs on
Linux, Windows and macOS.

This is a desktop app for [gnirehtet](https://github.com/Genymobile/gnirehtet),
the reverse tethering tool by Genymobile. Plug the phone in, click **Connect**,
accept the VPN prompt on the phone, and it is online. The app handles the parts
of gnirehtet that usually happen in a terminal: starting the relay, installing
the Android client, setting up the adb tunnel, and cleaning up when you
disconnect or unplug.

<p align="center">
  <img src="docs/screenshots/connected-light.png" alt="Gnirehtet connected to a phone, light theme" width="45%">
  &nbsp;
  <img src="docs/screenshots/connected-dark.png" alt="Gnirehtet connected to a phone, dark theme" width="45%">
</p>

It uses the unmodified gnirehtet **v2.5.1** relay and Android client; this project
adds the desktop interface and the process management around them.

The upstream relay source is also vendored in `crates/gnirehtet-relay`, because
upstream ships no prebuilt relay for macOS or ARM. On those platforms the build
compiles it from that copy instead of downloading it.

## When it's useful

- Your desktop is on wired Ethernet and there's no Wi-Fi for the phone.
- The phone has no SIM, no data plan, or weak signal where you are.
- You're testing an app on a real device and want it on the same network as
  your computer, including its VPN.
- You want to save mobile data while the phone is plugged in anyway.

## Features

- Detects connected phones and explains what to do when one is unauthorized,
  offline or not accessible (Linux udev permissions).
- Installs the gnirehtet client on the phone automatically on first use.
- Shows the connection state as it happens: starting, waiting for the phone's
  VPN prompt, connected, interrupted.
- Restores the USB tunnel automatically when the cable is plugged back in.
- Stops the VPN on the phone and the relay on the computer when you disconnect
  or quit.
- Optional DNS servers, routes and relay port; light and dark themes; a log view
  for troubleshooting.

## Install

Download the installer for your system from the
[Releases](https://github.com/sawongam/gnirehtet-gui/releases) page:

| System | File |
|--------|------|
| Debian, Ubuntu | `.deb` |
| Fedora, openSUSE | `.rpm` |
| Other Linux | `.AppImage` |
| Windows | `.msi` or `-setup.exe` |
| macOS | `.dmg` |

The app also needs **adb** (Android platform-tools). See the
[installation guide](docs/INSTALL.md) for step-by-step instructions for each
system and for preparing the phone.

## Usage

1. Enable **USB debugging** on the phone (Settings → About phone → tap *Build
   number* seven times, then Settings → Developer options → USB debugging).
2. Connect the phone with a USB cable and accept the *Allow USB debugging?*
   prompt on the phone.
3. Open Gnirehtet, select the phone and click **Connect**.
4. Accept the *Connection request* (VPN) prompt on the phone.

The phone now uses the computer's connection. Click **Disconnect** to stop.
If something goes wrong, see [Troubleshooting](docs/TROUBLESHOOTING.md).

## Limitations

These come from gnirehtet itself:

- IPv4 only: IPv6 traffic is not relayed.
- TCP and UDP only (no ICMP, so `ping` does not work on the phone).
- One phone at a time in this app.

## FAQ

**Is this the same as USB tethering?**
No, it's the opposite direction. Android's built-in USB tethering shares the
*phone's* mobile data with the computer. Reverse tethering shares the
*computer's* connection with the phone.

**Does the phone need root?**
No. The Android client uses the standard VPN API, which is why the phone asks
you to accept a connection request. Android 5.0 or newer is enough.

**Do I need to install anything on the phone?**
Only USB debugging has to be enabled. The app installs the small gnirehtet
client on the phone the first time you connect.

**Does it work without the command line?**
Yes, that's the point of this app. You still need adb installed on the
computer, but you never have to run it yourself.

## Building from source

```bash
git clone https://github.com/sawongam/gnirehtet-gui.git
cd gnirehtet-gui/apps/desktop
npm install
npm run setup        # fetches the pinned gnirehtet relay and APK (checksum-verified)
npm run tauri dev    # run in development
npm run tauri build  # build installers into target/release/bundle/
```

Prerequisites (Rust 1.88, Node.js 20.19+, system libraries) and the project layout
are described in the [development guide](docs/DEVELOPMENT.md).

## Documentation

- [Installation](docs/INSTALL.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Development](docs/DEVELOPMENT.md)
- [Contributing](CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)

## License

Apache License 2.0; see [LICENSE](LICENSE) and [NOTICE](NOTICE).

gnirehtet is developed by [Genymobile](https://github.com/Genymobile/gnirehtet)
and is also licensed under Apache 2.0. This project is not affiliated with
Genymobile.
