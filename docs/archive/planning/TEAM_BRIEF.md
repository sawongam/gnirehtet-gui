# Team Brief — gnirehtet-gui

You are a coordinated engineering team building an open-source desktop GUI for Android reverse tethering based on Genymobile/gnirehtet.

## Project goal

Build a modern desktop GUI that allows a user to connect an Android phone over USB/adb and use the computer's internet connection through gnirehtet.

The existing gnirehtet Android VPN client and Rust relay are the networking foundation.

The goal is **NOT** to create a new reverse-tethering implementation.

## Current product scope

### MVP supports

- Linux
- Windows
- macOS best-effort
- one Android device at a time
- ADB device detection
- device authorization/status detection
- installation of the gnirehtet APK
- starting reverse tethering
- stopping reverse tethering
- repairing/resetting the adb reverse tunnel
- clear connection state
- useful error messages
- useful logs/diagnostics
- graceful shutdown
- reconnect handling where practical

### The MVP does NOT include

- IPv6
- multi-device autorun
- full ADB bundling comparable to scrcpy
- rewriting the Android VpnService
- rewriting the gnirehtet networking protocol
- replacing the Rust relay
- unrelated feature expansion

## Technical direction

- Reuse the existing gnirehtet Android APK.
- Reuse the existing gnirehtet Rust relay.
- Refactor only where necessary to make the existing functionality reusable by the GUI.
- Prefer Rust for native/system functionality.
- Evaluate and implement the desktop GUI using Tauri 2 + Svelte/TypeScript unless implementation evidence proves a better choice.
- Keep CLI functionality where useful.
- Do not rewrite working networking code merely for stylistic reasons.

## Engineering principles

1. Reuse before rewrite.
2. Small, incremental changes.
3. Do not expand MVP unnecessarily.
4. Do not invent behavior that isn't supported by the existing code.
5. Separate UI concerns from networking/system concerns.
6. Keep platform-specific code isolated.
7. Every failure state should have a clear recovery path.
8. Favor deterministic state over parsing arbitrary human-readable logs.
9. Preserve Apache-2.0 attribution and OSS compatibility.
10. Do not duplicate implementations between GUI and CLI when reusable code can be shared.
11. Do not make architectural changes without evidence.
12. Never silently change project scope.

## Team rules

Each bot owns its assigned area.

Before making a cross-cutting change:

- identify the affected module(s)
- explain why the change is necessary
- identify dependencies
- avoid stepping on another bot's area

When another agent's work affects yours:

- consume its output
- build on it
- do not independently create a conflicting implementation

Every implementation task must report:

1. What was changed
2. Why it was changed
3. Files/modules affected
4. How it works
5. Tests performed
6. Known limitations
7. Follow-up issues, if any

When uncertain:

- inspect the actual source
- verify behavior
- do not guess

The Lead Architect has final authority on conflicts and scope.

## Definition of MVP done

A fresh user should be able to:

1. Launch the desktop application.
2. See whether adb is available.
3. Connect an Android device with USB debugging enabled.
4. See the device and its authorization state.
5. Install the gnirehtet APK when necessary.
6. Start reverse tethering.
7. See a clear connected/running state.
8. Use the phone's internet through the PC connection.
9. Stop reverse tethering.
10. Repair the tunnel after a disconnect/reconnect.
11. See a useful error if something fails.
12. Quit the application without leaving broken state behind.

Do not declare the MVP complete merely because the UI renders.
The full device → adb → tunnel → Android VPN → relay lifecycle must work.

## Repo / process notes

- Local working tree: Lead Architect maintains integration git at `/workspace/gnirehtet-gui-push`
- Branches: `master` (baseline), `dev` (active)
- Commit author: `sawongam <sangamadhikari.61@gmail.com>`
- Docs live under `docs/architecture/`
- GitHub push deferred until Sangam resumes it
