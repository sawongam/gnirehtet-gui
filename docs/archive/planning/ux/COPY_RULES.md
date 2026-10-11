# Copy Rules — gnirehtet-gui

**Audience:** Desktop Engineer, UX  
**Status:** Aligned to visual SoT v3 (2026-09-20)  
**SoT states:** `EVENT_STATUS_MAP.md`, `MVP_UX.md`  
**SoT errors:** `ERROR_UX.md`, `apps/desktop/src/lib/errorUx.ts`  
**Visual:** [`VISUAL_APPROVED_V3.md`](./VISUAL_APPROVED_V3.md)

Desktop must not invent Sharing / Connected claims or new error codes in chrome.

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

---

## 2. Connected synonym (v3)

UI **may** say **Connected** / “Internet tunnel active” as a user-facing synonym for the Sharing chip **only when** EVENT_STATUS_MAP three-layer Sharing is healthy.

| OK when three-layer healthy | Still forbidden early |
|-----------------------------|------------------------|
| Connected | Connected on intent-sent |
| Internet tunnel active | Online as synonym for Sharing without layers |
| Sharing chip | VPN Connected (commercial VPN metaphor) |

Chip vocabulary in diagnostics / compact chrome may still prefer **Sharing**; dashboard hero may use **Connected** under the rule above.

---

## 3. Primary verbs — decision

| Role | Label | Notes |
|------|-------|-------|
| Primary start | **Run** | IPC `runSession`. Screenshot “Connect device” may label the same pipeline on empty state. |
| Primary stop | **Stop** | IPC teardown. Screenshot **Stop tethering** / Disconnect = same stop session. |
| Healthy state | **Sharing** / **Connected** | Chip / hero only — not a button. Connected only if three-layer healthy. |
| Primary subtitle | Share this PC’s network | Under Run when Idle/Ready |

| Action | Shell label | Screenshot alias |
|--------|-------------|------------------|
| Repair | **Repair tunnel** | Restart tunnel |
| Install | **Install helper** | Install client |
| Refresh | **Refresh devices** | — |
| Relay | **Start Relay** / **Stop Relay** | Service / Advanced |
| VPN recheck | **I’ve allowed it** | — |
| Quit confirm | **Stop sharing and quit?** | — |

Do not fork QA with dual IPC names — aliases are labels only.

---

## 4. Direction chrome

Always valid:

> **Internet: This PC → Phone**

Do not invert. Prefer direction badge; “reverse tethering” OK in brand/subtitle.

---

## 5. Color semantics in copy/chrome

| Color | Meaning |
|-------|---------|
| **Green** | Health only (Connected/Sharing, ADB ok, Online, service healthy) |
| **Blue** | Actions (Run, Connect, nav, switches ON) |
| **Red** | Destructive / error only (Stop tethering, Disconnect, fatal) |

Switches ON = blue, **not** green.

---

## 6. Forbidden chrome words (still)

| Ban | Why |
|-----|-----|
| Connected *(before three-layer healthy)* | Lies about tunnel |
| Online *(as session chip for Sharing)* | Ambiguous — device Online ≠ Sharing |
| VPN Connected | Commercial / PC VPN metaphor |
| Kill switch / BLOCKING INTERNET / Protected / Secured | Wrong metaphor |
| HANDSHAKE_LIVENESS_MVP | Dev id — never in UI |

---

## 7. Developer → user rewrites

| Internal / current UI | User-facing |
|-----------------------|-------------|
| Pending until handshake (see HANDSHAKE_LIVENESS_MVP) — not Sharing | Waiting for phone to finish connecting |
| Run ≈ install-if-needed → tunnel → start… | Share this PC’s network *(subtitle)* |
| owned (relay) | Hide from main chrome |
| Tauri + SvelteKit + Typescript App | gnirehtet-gui |

---

## 8. ERROR_UX in banners

- Always show **`[CODE]`** before title.  
- Use `errorUxFor(code)` — do not invent codes.  
- Recovery Buttons named in ERROR_UX (Repair tunnel, Reinstall helper, …).

---

## 9. Layer labels (short)

| Layer | Healthy | Waiting / off | Bad |
|-------|---------|---------------|-----|
| Relay | Listening | Off | Error |
| Tunnel | OK | — | Lost / Failed |
| Device VPN | On / Active | Waiting / Off | Error |

---

## 10. Checklist for Desktop PR

- [ ] Buttons: Run/Stop (or screenshot aliases) / Repair tunnel / Install helper / Refresh / Relay  
- [ ] Connected / Sharing only when three-layer healthy  
- [ ] No early Connected; no HANDSHAKE_* in chrome  
- [ ] Banners show `[CODE]`  
- [ ] Green = health; blue = actions; red = destructive only  
- [ ] Direction: Internet: This PC → Phone  

---

*Copy only. Layout → `SHELL_SCREENS.md`. Tokens → `DESIGN_TOKENS.md`. Errors → `ERROR_UX.md`.*
