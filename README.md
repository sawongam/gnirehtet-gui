# gnirehtet-gui

Open-source **desktop GUI** successor / front-end for [Genymobile/gnirehtet](https://github.com/Genymobile/gnirehtet) — reverse tethering for Android over `adb`, without root.

This repository is the **product home**. Upstream gnirehtet remains a **pinned dependency** (vendored or submodule), not this repo’s conceptual `origin` for networking logic.

## Status

Early architecture / planning. **Working tree is local-first for now** (GitHub push deferred). Default integration branch: `dev` (authored as `sawongam`). Baseline branch: `master`.

Implementation spikes next once docs stabilize.

## Docs

| Path | Owner |
|------|--------|
| [`docs/architecture/`](docs/architecture/) | Lead architecture, MVP, roadmap, migration |
| [`docs/architecture/rust/`](docs/architecture/rust/) | Rust crate structure, concurrency, refactor plan |

## Upstream pin

Planning docs are pinned to Genymobile/gnirehtet **`1eb2e58`** / **v2.5.1** unless an ADR updates the pin.

## License

Apache License 2.0. Reused gnirehtet components remain under their Apache-2.0 terms with Genymobile copyright retained in those files / `NOTICE`.
