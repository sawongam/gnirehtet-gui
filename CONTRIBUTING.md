# Contributing

Bug reports, fixes and improvements are welcome.

## Reporting a problem

Open an [issue](https://github.com/sawongam/gnirehtet-gui/issues) and include
your operating system, phone model and Android version, the app version, and
the log from the **Logs** page. [TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)
lists what else helps.

Problems with the networking itself (particular apps or protocols not working
through the tunnel) usually belong to
[upstream gnirehtet](https://github.com/Genymobile/gnirehtet/issues). If they
also happen with the `gnirehtet` command-line tool, report them there.

## Making a change

1. Set up the project as described in [DEVELOPMENT.md](docs/DEVELOPMENT.md).
2. Create a branch from `dev`.
3. Keep the change focused. Add or update tests for behavior changes. The
   session logic in `apps/desktop/src-tauri/src/session.rs` and
   `apps/desktop/src/lib/session-logic.ts` is covered by tests that do not need
   a phone.
4. Run the checks:
   ```bash
   cargo fmt --all --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cd apps/desktop && npm run check && npm test
   ```
5. For changes to the connection flow, go through the manual test checklist in
   DEVELOPMENT.md with a real phone.
6. Open a pull request against `dev`. Say what changed, why, and how you tested
   it.

Commit messages follow [Conventional Commits](docs/COMMIT_CONVENTION.md)
(`fix: …`, `feat(ui): …`, `docs: …`).

## Guidelines

- Do not modify `crates/gnirehtet-relay`. It is upstream code, kept identical to
  the pinned release.
- Everything that talks to adb goes through `gnirehtet-adb`, and every user
  input that reaches `adb shell` must be validated there.
- User-visible errors have a stable code (see `CommandError` in Rust and
  `src/lib/errors.ts`). Add an explanation to `errors.ts` when you add a code.
- Keep the UI copy short and actionable: what happened and what to do.

## License

By contributing, you agree that your contributions are licensed under the
Apache License 2.0, like the rest of the project.
