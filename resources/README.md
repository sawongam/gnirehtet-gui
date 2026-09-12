# Bundled Android client (APK)

Place the upstream **gnirehtet.apk** here as:

```text
resources/gnirehtet.apk
```

## Source

Download from the pinned Genymobile/gnirehtet release (see repo README / architecture pin, currently **v2.5.1** / `1eb2e58`). Do **not** rebuild or modify the VpnService for MVP.

Example:

```bash
# Extract gnirehtet.apk from the upstream linux64 release zip (see also scripts/phase0_lab_relay.sh)
curl -fsSL -o /tmp/gnirehtet-rust-linux64-v2.5.1.zip \
  "https://github.com/Genymobile/gnirehtet/releases/download/v2.5.1/gnirehtet-rust-linux64-v2.5.1.zip"
unzip -p /tmp/gnirehtet-rust-linux64-v2.5.1.zip gnirehtet-rust-linux64/gnirehtet.apk \
  > resources/gnirehtet.apk
```

## Resolution order (orchestrator)

When the host-orchestrator resolves the APK path (later settings + install flows):

1. Explicit app setting (UI Settings → APK path)
2. Environment variable `GNIREHTET_APK`
3. Bundled default: `resources/gnirehtet.apk` (copied into the app bundle as a Tauri resource)

The APK file is **not** committed to git (binary; see `.gitignore`). CI and local clones must drop it in or set `GNIREHTET_APK`.

## Tauri bundle

`apps/desktop/src-tauri/tauri.conf.json` includes `../../resources/gnirehtet.apk` under `bundle.resources` when present. Builds tolerate a missing APK in early spikes; install commands will return a clear error until the file is provided.
