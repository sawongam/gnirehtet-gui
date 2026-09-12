# gnirehtet-gui

Open-source **desktop GUI** successor / front-end for [Genymobile/gnirehtet](https://github.com/Genymobile/gnirehtet) — reverse tethering for Android over `adb`, without root.

This repository is the **product home**. Upstream gnirehtet remains a **pinned dependency** (vendored or submodule), not this repo’s conceptual `origin` for networking logic.

## Status

Early architecture / planning. **Working tree is local-first for now** (GitHub push deferred). Default integration branch: `dev` (authored as `sawongam`). Baseline branch: `master`.

Implementation: **Phase 0 desktop scaffold** lives under `apps/desktop` (Tauri 2 + Svelte/TS).
Rust Phase 1 crates land under `crates/` (`gnirehtet-adb`, `gnirehtet-relay`, `gnirehtet-cli`).
The GUI does **not** link `relaylib` — it spawns stock `gnirehtet` via externalBin/PATH.

```bash
cd apps/desktop && npm install && npm run tauri dev
# or: cargo check -p gnirehtet-desktop
```

See `apps/desktop/README.md`, `apps/desktop/docs/SIDECAR.md`, `resources/README.md`.


## Docs

| Path | Owner |
|------|--------|
| [`docs/architecture/`](docs/architecture/) | Lead architecture, MVP, roadmap, migration |
| [`docs/architecture/rust/`](docs/architecture/rust/) | Rust crate structure, concurrency, refactor plan |

## Upstream pin

Planning docs are pinned to Genymobile/gnirehtet **`1eb2e58`** / **v2.5.1** unless an ADR updates the pin.

## License

Apache License 2.0. Reused gnirehtet components remain under their Apache-2.0 terms with Genymobile copyright retained in those files / `NOTICE`.
