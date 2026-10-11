# Bundled Android client

`gnirehtet.apk` is the unmodified gnirehtet v2.5.1 Android client. It is bundled
with the app and installed on the phone on first connection.

The file is not committed. Create it with:

```bash
cd apps/desktop && npm run setup
```

This extracts it from the pinned upstream release and verifies its SHA-256
checksum (see `scripts/fetch-upstream.mjs`).
