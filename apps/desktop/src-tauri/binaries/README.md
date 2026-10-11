# Bundled relay

Tauri bundles `gnirehtet-relay-<target-triple>[.exe]` from this directory as a
sidecar and installs it next to the app as `gnirehtet-relay`. It is the
gnirehtet v2.5.1 relay.

The file is not committed. Create it with:

```bash
cd apps/desktop && npm run setup                                   # this machine
node ../../scripts/fetch-upstream.mjs --target x86_64-pc-windows-msvc  # another target
```
