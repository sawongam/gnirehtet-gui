# Installation

Setting up takes three steps:

1. [Install the app](#1-install-the-app)
2. [Install adb](#2-install-adb)
3. [Prepare the phone](#3-prepare-the-phone)

The phone needs Android 5.0 or newer. It does **not** need to be rooted.

The app ships with the gnirehtet relay and the Android client, so nothing else
needs to be downloaded. The client is installed on the phone automatically the
first time you connect.

## 1. Install the app

Download the file for your system from the
[latest release](https://github.com/sawongam/gnirehtet-gui/releases/latest).

### Debian, Ubuntu, Linux Mint, Pop!_OS

```bash
sudo apt install ./gnirehtet-gui-*-linux-x64.deb
```

`apt` also installs `adb`, which the package recommends. Launch **Gnirehtet**
from the applications menu, or run `gnirehtet-gui`.

To remove it: `sudo apt remove gnirehtet-gui`.

### Fedora, openSUSE, RHEL

```bash
sudo dnf install ./gnirehtet-gui-*-linux-x64.rpm       # Fedora / RHEL
sudo zypper install ./gnirehtet-gui-*-linux-x64.rpm    # openSUSE
```

### Arch Linux, Manjaro, EndeavourOS

The [`gnirehtet-gui-bin`](https://aur.archlinux.org/packages/gnirehtet-gui-bin)
AUR package installs the release build, along with `adb` (`android-tools`):

```bash
yay -S gnirehtet-gui-bin       # or paru, or any AUR helper
```

Without an AUR helper:

```bash
git clone https://aur.archlinux.org/gnirehtet-gui-bin.git
cd gnirehtet-gui-bin
makepkg -si
```

To remove it: `sudo pacman -R gnirehtet-gui-bin`.

### Other Linux distributions (AppImage)

```bash
chmod +x gnirehtet-gui-*-linux-x64.AppImage
./gnirehtet-gui-*-linux-x64.AppImage
```

If the AppImage does not start and mentions FUSE, install your distribution's
`fuse2` / `libfuse2` package, or run it with `--appimage-extract-and-run`.

On any distribution you can also build from source (see [DEVELOPMENT.md](DEVELOPMENT.md)).

### Windows 10 and 11

Run `gnirehtet-gui-<version>-windows-x64-setup.exe`. The app uses Microsoft Edge
WebView2, which Windows 10 and 11 normally already have; if it is missing, the
installer downloads it.

The installer is not code-signed. If SmartScreen shows *Windows protected your
PC*, click **More info → Run anyway**.

### macOS 10.15 or newer

Download `gnirehtet-gui-<version>-macos-apple-silicon.dmg` for Apple Silicon
(M1 and later) or `gnirehtet-gui-<version>-macos-intel.dmg` for Intel Macs.
Open it and drag **gnirehtet-gui** to *Applications*.

The app is not notarized. The first time, right-click it and choose **Open**, or
run:

```bash
xattr -dr com.apple.quarantine /Applications/gnirehtet-gui.app
```

macOS builds are produced by CI but are less tested than Linux and Windows.

## 2. Install adb

The app uses `adb` (from Android *platform-tools*) to talk to the phone. It finds
adb on your `PATH`, then in the default Android SDK location
(`~/Android/Sdk`, `~/Library/Android/sdk` or `%LOCALAPPDATA%\Android\Sdk`,
or `$ANDROID_HOME`). To use a specific adb, set the `ADB` environment variable to
its full path.

| System | Command |
|--------|---------|
| Debian, Ubuntu | `sudo apt install adb` |
| Fedora | `sudo dnf install android-tools` |
| Arch Linux | `sudo pacman -S android-tools android-udev` |
| openSUSE | `sudo zypper install android-tools` |
| macOS (Homebrew) | `brew install android-platform-tools` |
| Windows (winget) | `winget install Google.PlatformTools` |

Otherwise, download
[SDK Platform-Tools](https://developer.android.com/tools/releases/platform-tools)
and add the extracted `platform-tools` folder to your `PATH`.

Check that it works by running `adb version` in a new terminal. If Android Studio
is installed, adb is already on the system.

### Linux: USB permissions

If the app lists the phone as **No permission**, your user is not allowed to
access the USB device. Install the Android udev rules and reconnect the phone:

| System | Package |
|--------|---------|
| Debian, Ubuntu | `sudo apt install android-sdk-platform-tools-common` |
| Arch Linux | `sudo pacman -S android-udev` |
| Fedora | usually covered by `android-tools` |

On some distributions you also need to add yourself to the `plugdev` or
`adbusers` group (`sudo usermod -aG plugdev $USER`) and log out and back in.

### Windows: USB driver

Most phones work with the driver Windows installs automatically. If `adb devices`
does not show the phone, install your manufacturer's USB driver or the
[Google USB Driver](https://developer.android.com/studio/run/win-usb).

## 3. Prepare the phone

1. Open **Settings → About phone** and tap **Build number** seven times to
   unlock the developer options. (On some phones it is under *Software
   information*.)
2. Open **Settings → System → Developer options** and turn on **USB debugging**.
3. Connect the phone with a USB cable that carries data (some charging cables
   do not).
4. Unlock the phone and accept **Allow USB debugging?**. Tick *Always allow from
   this computer* so it is not asked again.

Some manufacturers add extra switches:

- **Xiaomi / Redmi / POCO:** also turn on **Install via USB** (requires signing
  in to a Mi account) and **USB debugging (Security settings)**.
- **OPPO / realme / OnePlus:** turn off **Permission monitoring** if the client
  cannot be installed.
- **Huawei:** turn on **Allow ADB debugging in charge only mode**.

## Connecting

1. Open Gnirehtet. Once the phone is detected, the window shows **Ready to
   connect**. With more than one phone plugged in, pick one above the status.
2. Click **Connect**. The first time, the gnirehtet client is installed on the
   phone.
3. On the phone, accept the **Connection request** prompt. This is Android's
   standard confirmation for VPN apps; gnirehtet uses a local VPN to route the
   phone's traffic through the USB cable.
4. The status changes to **Connected**. A key icon in the phone's status bar
   shows that the VPN is active.

Click **Disconnect** to stop, or quit the app: both stop the VPN on the phone.
If the cable is unplugged while connected, the app shows **Connection
interrupted** and resumes by itself when the phone is plugged back in.

Having trouble? See [TROUBLESHOOTING.md](TROUBLESHOOTING.md).

## Advanced settings

The **Preferences** view has three connection options, all optional:

| Option | Default | Description |
|--------|---------|-------------|
| Relay port | `31416` | Local TCP port for the relay. Change it if another program uses 31416. |
| DNS servers | `8.8.8.8` | Comma-separated IPv4 addresses the phone uses for DNS. |
| Routes | `0.0.0.0/0` | Comma-separated IPv4 CIDR ranges to send through the computer; everything else uses the phone's own connection. |

Environment variables, read at startup:

| Variable | Purpose |
|----------|---------|
| `ADB` | Path to the `adb` executable. |
| `GNIREHTET_BIN` | Path to a `gnirehtet` relay executable to use instead of the bundled one. |
| `GNIREHTET_APK` | Path to a `gnirehtet.apk` to install instead of the bundled one. |

The paths the app ends up using are written to the first line of the **Log**.
