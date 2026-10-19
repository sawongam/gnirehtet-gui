# Security

The app runs `adb` commands against your phone and starts a network relay on
your computer, so security bugs matter here. Examples: a way to run commands on
the phone through the app's settings, the relay listening on more than
localhost, or the app killing processes it didn't start.

## Reporting

Please don't open a public issue. Use GitHub's
[private vulnerability reporting](https://github.com/sawongam/gnirehtet-gui/security/advisories/new).

Include what you found, how to reproduce it, and the app version. I'll reply
within a week and let you know when a fix is released.

Problems in the relay or the Android client themselves belong to
[upstream gnirehtet](https://github.com/Genymobile/gnirehtet).

## Supported versions

Only the latest release gets fixes.
