# Desktop app

The Tauri 2 application: a SvelteKit frontend (`src/`) and a Rust backend
(`src-tauri/`).

```bash
pnpm install
pnpm run setup    # fetch the bundled relay and APK
pnpm tauri dev    # run with hot reload
pnpm tauri build  # build installers
pnpm check        # type-check the frontend
pnpm test         # frontend unit tests
```

See [docs/DEVELOPMENT.md](../../docs/DEVELOPMENT.md) for prerequisites and
[docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md) for how it works.
