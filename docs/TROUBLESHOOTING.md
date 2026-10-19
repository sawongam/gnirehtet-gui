# Troubleshooting

When something fails, the app shows a message explaining what to do. The
**Log** view has the full detail, including the relay's own output. Use the
**Copy** button there when you report a problem.

## The app says "adb was not found"

adb is not installed, or it is not where the app looks. Install it (see
[INSTALL.md](INSTALL.md#2-install-adb)) and restart the app. If adb is installed
in an unusual place, start the app with the `ADB` environment variable set to
its full path. For example, on Linux:

```bash
ADB=/opt/platform-tools/adb gnirehtet-gui
```

The first line of the **Log** shows the adb, relay and client APK paths the
app is using.

## The app keeps saying "Plug in your phone"

- Check that **USB debugging** is enabled on the phone.
- Try a different cable or port. Many cables only carry power.
- Set the phone's USB mode to *File transfer* instead of *Charging only*. Some
  phones only expose adb in that mode.
- Run `adb devices` in a terminal. If the phone is not listed there either, the
  problem is between the phone and adb (driver, cable, udev rules), not the app.
- On Windows, install the manufacturer's USB driver.

## "No permission" (Linux)

Your user cannot open the USB device. Install the udev rules, then unplug and
reconnect the phone (see [INSTALL.md](INSTALL.md#linux-usb-permissions)).

## "Not authorized"

Unlock the phone and accept **Allow USB debugging?**. If no prompt appears:

1. On the phone, open **Developer options → Revoke USB debugging authorizations**.
2. Unplug and reconnect the cable, then accept the new prompt.
3. If it still does not appear, run `adb kill-server` and reconnect.

## "Offline"

Unlock the phone, or unplug and reconnect it. If it stays offline, run
`adb kill-server` in a terminal; the app starts adb again automatically.

## "Could not install the client on the phone"

- Unlock the phone. Some phones ask for confirmation before installing over USB.
- **Xiaomi / Redmi / POCO:** enable **Install via USB** in developer options.
- **OPPO / realme / OnePlus:** disable **Permission monitoring**.
- If a different gnirehtet client is already installed (for example, one you
  built yourself, signed with another key), uninstall it first:
  `adb uninstall com.genymobile.gnirehtet`.

## Stuck on "Waiting for the phone"

The client started on the phone, but the VPN has not been accepted yet.

- Unlock the phone and look for the **Connection request** prompt. Tap **OK**.
- If another VPN app is set as **Always-on VPN**, Android may not let gnirehtet
  start. Turn that off in **Settings → Network → VPN**.
- If you denied the prompt earlier, disconnect and connect again to get a new
  one.

## "The relay port is already in use"

Another program is listening on port 31416. This is usually a `gnirehtet run`
or `gnirehtet relay` started in a terminal, or a second copy of this app. Stop
it, or pick another **Relay port** in Preferences.

The app never stops or takes over a relay it did not start itself.

## Connected, but the phone has no internet

- Check that the computer itself is online.
- **DNS:** the phone uses `8.8.8.8` by default. If your network blocks public DNS
  (common on corporate networks), enter your network's DNS server under
  **Preferences → DNS servers**.
- **Firewall:** the relay makes normal outgoing connections on the phone's
  behalf. On Windows, allow `gnirehtet-relay` if the firewall asks. Security
  software that filters outgoing traffic may need an exception.
- **Routes:** if you changed **Preferences → Routes**, only those ranges go
  through the computer.
- IPv6-only sites and `ping` do not work; gnirehtet relays IPv4 TCP and UDP only.

## "Connection interrupted"

The USB connection was lost while connected (cable unplugged, phone rebooted,
adb restarted). Plug the phone back in: the app restores the tunnel
automatically. If it does not, click **Restore tunnel**. To stop completely,
click **Disconnect**.

## "The relay stopped"

The relay process exited unexpectedly. The Log view shows its last output.
Click **Reconnect** to start it again.

## The phone still shows the VPN key icon after the app closed

The app stops the VPN on the phone when you disconnect or quit. It can only do
that while the phone is connected. If the cable was unplugged first, open the
gnirehtet notification on the phone and use its stop action, or reconnect the phone
and click **Connect** and then **Disconnect**.

## "The relay program is missing" / "The gnirehtet client APK is missing"

The installation is incomplete. Reinstall the app. If you built it yourself, run
`pnpm run setup` in `apps/desktop` before building (see
[DEVELOPMENT.md](DEVELOPMENT.md)).

## Reporting a bug

Open an issue at <https://github.com/sawongam/gnirehtet-gui/issues> and include:

- your operating system and version, and the phone model and Android version;
- the app version (**Preferences → About**) and the output of `adb version`;
- what you did, what you expected, and what happened instead;
- the log (**Log → Copy**).
