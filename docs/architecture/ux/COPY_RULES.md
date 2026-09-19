# Copy Rules — gnirehtet-gui

**Audience:** Desktop Engineer, UX  
**Status:** Shell vocabulary for ASAP polish  
**SoT states:** `EVENT_STATUS_MAP.md`, `MVP_UX.md`  
**SoT errors:** `ERROR_UX.md`, `apps/desktop/src/lib/errorUx.ts`

Desktop must not invent Sharing claims or new error codes in chrome.

---

## 1. Session chip (exact)

| Chip | When (summary) |
|------|----------------|
| Idle | No owned session |
| Starting | Run pipeline in progress |
| Waiting for VPN | Client started; Device VPN not Active |
| Sharing | Relay + Tunnel + Device VPN handshake healthy |
| Interrupted | Was Sharing; tunnel lost |
| Stopping | Stop pipeline |
| Error | Blocking error; title from ERROR_UX |

**Never** show Sharing / Connected on intent-sent or relay-only.

Chip “Waiting for VPN” may shorten from MVP_UX “Waiting for VPN permission” — keep meaning identical to EVENT_STATUS_MAP.

---

## 2. Primary verbs — decision

| Role | Label | Notes |
|------|-------|-------|
| Primary start | **Run** | Matches current IPC / `+page.svelte` (`runSession`). Keep for shell. |
| Primary stop | **Stop** | Matches `stopClient` teardown. |
| Healthy state | **Sharing** | Chip only — not a button label. |
| Primary subtitle | Share this PC’s network | Under Run when Idle/Ready |

**Do not** dual-label buttons “Start sharing” in the shell while IPC says Run — that forks QA and screenshots. MVP_UX’s “Start sharing” remains the *product verb* in docs; **shell buttons stay Run / Stop**.

| Action | Shell label |
|--------|-------------|
| Repair | **Repair tunnel** |
| Install | **Install helper** (UI; short “Install” OK in tight layouts) |
| Refresh | **Refresh devices** |
| Relay | **Start Relay** / **Stop Relay** (Advanced only) |
| VPN recheck | **I’ve allowed it** (refresh / recheck) |
| Quit confirm | **Stop sharing and quit?** (dialog — “sharing” OK in sentence sense) |

---

## 3. Direction chrome

Always:

> **Internet: This PC → Phone**

Do not invert. Do not say “tether” without “reverse” if used at all (prefer direction badge alone).

---

## 4. Forbidden chrome words

| Ban | Why |
|-----|-----|
| Connected | Ambiguous; implies Sharing without layers |
| Online | Same |
| VPN Connected | Implies commercial VPN / PC VPN |
| Kill switch / BLOCKING INTERNET / Protected / Secured | Wrong metaphor (`ERROR_UX`, `MVP_UX`) |
| HANDSHAKE_LIVENESS_MVP | Developer doc id — never in UI |
| Pending until handshake (see …) | Current `+page.svelte` jargon — strip |

---

## 5. Developer → user rewrites

| Internal / current UI | User-facing |
|-----------------------|-------------|
| Pending until handshake (see HANDSHAKE_LIVENESS_MVP) — not Sharing | Waiting for phone to finish connecting |
| Run ≈ install-if-needed → tunnel → start + session-owned relay… | Share this PC’s network *(subtitle)*; details stay in logs/docs |
| Phase 3 — quit-clean, ERROR_UX recovery… | Remove from header subtitle |
| owned (relay) | Hide from main chrome; Advanced/Diagnostics only if needed |
| Tauri + SvelteKit + Typescript App (`app.html`) | gnirehtet-gui |

Layer tooltips: one plain sentence each (`COMPONENT_INVENTORY.md`) — no architecture filenames.

---

## 6. ERROR_UX in banners

- Always show stable code as **`[CODE]`** before title, e.g. `[TUNNEL_LOST] Sharing interrupted`.  
- Use `errorUxFor(code)` title / explanation / recoveryHint.  
- Do not invent codes. Prefer recovery Buttons named in ERROR_UX (Repair tunnel, Reinstall helper, …).  
- Log dump is not a title; optional collapsed “Show logs”.

---

## 7. Layer labels (short)

| Layer | Healthy | Waiting / off | Bad |
|-------|---------|---------------|-----|
| Relay | Listening | Off | Error |
| Tunnel | OK | — | Lost / Failed |
| Device VPN | On / Active | Waiting / Off | Error |

Port display: `:31416` muted beside Relay when known.

---

## 8. Empty / auth snippets (keep calm)

Reuse MVP_UX tone:

- No devices → cable, USB debugging, unlock, Accept Allow USB debugging.  
- Unauthorized → Unlock and tap Allow; Always allow from this computer.  
- ADB missing → Choose platform-tools adb; app does not auto-download.

---

## 9. Checklist for Desktop PR

- [ ] Buttons: Run / Stop / Repair tunnel / Install helper / Refresh devices / Advanced Relay  
- [ ] Chip vocabulary matches EVENT_STATUS_MAP  
- [ ] No Connected / Online / VPN Connected  
- [ ] No HANDSHAKE_* or architecture ids in chrome  
- [ ] Banners show `[CODE]`  
- [ ] Sharing only when three layers + handshake  

---

*Copy only. Layout → `SHELL_SCREENS.md`. Errors → `ERROR_UX.md`.*
