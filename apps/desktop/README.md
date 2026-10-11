# Desktop app

The Tauri 2 application: a SvelteKit frontend (`src/`) and a Rust backend
(`src-tauri/`).

```bash
npm install
npm run setup        # fetch the bundled relay and APK
npm run tauri dev    # run with hot reload
npm run tauri build  # build installers
npm run check        # type-check the frontend
npm test             # frontend unit tests
```

See [docs/DEVELOPMENT.md](../../docs/DEVELOPMENT.md) for prerequisites and
[docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md) for how it works.
