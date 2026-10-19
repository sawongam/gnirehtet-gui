// A scripted stand-in for the Tauri backend, so the unmodified desktop UI runs
// in the landing page's demo window. scripts/build-demo.mjs loads it before
// the app starts.
//
// The page drives it through `window.__demo` (same origin): plug the phone in,
// allow USB debugging, accept the VPN request, move a fake cursor and click.
// Visitors can also click the app and the phone illustration themselves.
(() => {
  const REPO_URL = "https://github.com/sawongam/gnirehtet-gui";
  const PORT = 31416;
  const PHONE = { serial: "37211FDJH00A4K", model: "Pixel 8", product: "shiba" };
  const STOPPED = { state: "stopped", port: null, pid: null, message: null };
  const RUNNING = { state: "running", port: PORT, pid: 48213, message: null };

  try {
    // Start from the defaults on each visit, in the page's light theme.
    localStorage.setItem(
      "gnirehtet-gui.settings",
      JSON.stringify({ port: PORT, dnsServers: "", routes: "", theme: "light" }),
    );
  } catch {
    // Storage blocked: the app falls back to its defaults.
  }

  // The app polls adb every 2 s. Here the "phone" answers instantly, so poll
  // often enough that plugging it in feels immediate.
  const setIntervalNative = window.setInterval.bind(window);
  window.setInterval = (fn, ms, ...rest) => setIntervalNative(fn, ms === 2000 ? 250 : ms, ...rest);

  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const callbacks = new Map();
  const listeners = new Map();
  let nextId = 1;
  let installed = false;

  const demo = (window.__demo = {
    /** unplugged | unauthorized | ready | starting | waiting | connected */
    stage: "unplugged",
    /** Set by the page: the version the About panel shows. */
    version: "",
    devices: [],
    relay: STOPPED,
    /** Set by the page: called with the stage after every change. */
    onstage: null,
    /** Set by the page: called once the visitor uses the demo themselves. */
    onvisitor: null,

    plug() {
      if (demo.stage !== "unplugged") return;
      demo.devices = [{ ...PHONE, state: "unauthorized", model: null, product: null }];
      setStage("unauthorized");
    },
    allow() {
      if (demo.stage !== "unauthorized") return;
      demo.devices = [{ ...PHONE, state: "device" }];
      setStage("ready");
    },
    accept() {
      if (demo.stage !== "waiting") return;
      log("info", "relay", "Client #0 connected");
      emit("relay-client", { connected: true, clientId: 0, connectedClients: 1 });
      setStage("connected");
    },
    unplug() {
      demo.devices = [];
      setStage("unplugged");
    },
    /** Puts everything back to the start, as if the app was just opened. */
    reset() {
      if (demo.relay.state === "running") {
        demo.relay = STOPPED;
        emit("relay-status", STOPPED);
      }
      demo.unplug();
    },
    button(label) {
      return [...document.querySelectorAll("button")].find(
        (b) => b.textContent.trim() === label && !b.disabled && b.offsetParent,
      );
    },
    cursor: null,
  });

  function setStage(stage) {
    demo.stage = stage;
    document.documentElement.dataset.demoStage = stage;
    demo.onstage?.(stage);
  }

  function emit(event, payload) {
    for (const id of listeners.get(event) ?? []) callbacks.get(id)?.({ event, id: 0, payload });
  }

  function log(level, source, message) {
    emit("log", { timestampMs: Date.now(), level, source, message });
  }

  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: "main" }, currentWebview: { windowLabel: "main", label: "main" } },
    transformCallback(cb, once) {
      const id = nextId++;
      callbacks.set(id, (d) => {
        if (once) callbacks.delete(id);
        return cb && cb(d);
      });
      return id;
    },
    unregisterCallback: (id) => callbacks.delete(id),
    convertFileSrc: (p) => p,
    async invoke(cmd, args) {
      switch (cmd) {
        case "plugin:event|listen":
          if (!listeners.has(args.event)) listeners.set(args.event, []);
          listeners.get(args.event).push(args.handler);
          if (args.event === "log") {
            setTimeout(() => log("info", "app", "adb: /usr/bin/adb · relay: gnirehtet 2.5.1 · client: gnirehtet.apk"));
          }
          return args.handler;
        case "plugin:event|unlisten":
          return null;
        case "check_adb":
          return { path: "/usr/bin/adb", version: "35.0.2" };
        case "list_devices":
          return demo.devices;
        case "get_session":
          return { relay: demo.relay, serial: null, connectedClients: 0 };
        case "connect":
          setStage("starting");
          log("info", "app", `Starting the relay on port ${PORT}`);
          await sleep(500);
          demo.relay = RUNNING;
          emit("relay-status", RUNNING);
          log("info", "relay", `Relay running (pid ${RUNNING.pid}, port ${PORT})`);
          if (!installed) {
            log("info", "app", `Installing the client on ${PHONE.serial}`);
            await sleep(700);
            log("info", "app", `Client installed on ${PHONE.serial}`);
            installed = true;
          }
          await sleep(300);
          log("info", "app", `Waiting for the VPN to be accepted on ${PHONE.serial}`);
          setStage("waiting");
          return RUNNING;
        case "disconnect":
          log("info", "app", `Stopping the client on ${PHONE.serial}`);
          await sleep(600);
          log("info", "app", "Relay stopped");
          demo.relay = STOPPED;
          setStage(demo.devices.length ? "ready" : "unplugged");
          return STOPPED;
        case "install_client":
          await sleep(900);
          installed = true;
          return null;
        case "repair_tunnel":
          await sleep(300);
          return null;
        case "get_environment":
          return {
            appVersion: demo.version,
            adbPath: "/usr/bin/adb",
            relayPath: "gnirehtet",
            relayFound: true,
            apkPath: "gnirehtet.apk",
            apkFound: true,
          };
        case "open_repo":
          window.open(REPO_URL, "_blank", "noopener");
          return null;
        default:
          // Window plugin calls (close requests and such) have nothing to do here.
          return null;
      }
    },
  };

  // ---- Visitor interaction ---------------------------------------------------

  let visitor = false;
  function takeOver() {
    if (visitor) return;
    visitor = true;
    demo.cursor?.hide();
    demo.onvisitor?.();
  }
  // Real pointer use (not the scripted cursor, which dispatches no pointer events).
  addEventListener("pointerdown", (e) => e.isTrusted && takeOver(), true);
  addEventListener("keydown", (e) => e.isTrusted && takeOver(), true);

  addEventListener("click", (e) => {
    // The phone illustration is a picture in the real app; here its prompts
    // can be tapped, standing in for the real phone.
    if (!e.target.closest?.(".stage .phone")) return;
    if (demo.stage === "unauthorized") demo.allow();
    else if (demo.stage === "waiting") demo.accept();
  });

  // ---- Scripted cursor ---------------------------------------------------------

  const ARROW = `<svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 22 22"><path d="M3 2 L3 18 L7.2 14.2 L10 20.5 L12.8 19.3 L10.1 13.2 L15.8 13.2 Z" fill="#111" stroke="#fff" stroke-width="1.4" stroke-linejoin="round"/></svg>`;
  const HAND = `<svg xmlns="http://www.w3.org/2000/svg" width="22" height="22" viewBox="0 0 22 22"><path d="M8 3.2c0-.9.7-1.6 1.6-1.6s1.6.7 1.6 1.6V9l.5-.1c.8-.1 1.5.5 1.6 1.3l.1.3.4-.1c.8-.1 1.5.4 1.6 1.2v.3l.4-.1c.8 0 1.5.6 1.5 1.4v3.6c0 2.8-2.2 4.8-5 4.8h-1.2c-1.7 0-3.2-.8-4.2-2.2L3.2 14c-.5-.7-.3-1.6.4-2 .6-.4 1.4-.3 1.9.2L8 14.4Z" fill="#fff" stroke="#111" stroke-width="1.3" stroke-linejoin="round"/></svg>`;

  addEventListener("DOMContentLoaded", () => {
    const style = document.createElement("style");
    style.textContent = `
      :root[data-demo-stage="unauthorized"] .stage .phone,
      :root[data-demo-stage="waiting"] .stage .phone { cursor: pointer; }
      #demo-cursor { position: fixed; left: 0; top: 0; z-index: 99999; pointer-events: none;
        filter: drop-shadow(0 1px 1.5px rgb(0 0 0 / 0.35)); opacity: 0; transition: opacity 300ms; }
      #demo-cursor.on { opacity: 1; }
      #demo-cursor svg { display: block; transition: transform 90ms ease-out; }
      #demo-cursor.down svg { transform: scale(0.86); }
    `;
    document.head.appendChild(style);

    const el = document.createElement("div");
    el.id = "demo-cursor";
    el.innerHTML = ARROW;
    document.documentElement.appendChild(el);

    let pos = { x: innerWidth * 0.66, y: innerHeight * 0.9 };
    let hand = false;
    const place = () => {
      const over = document.elementFromPoint(pos.x, pos.y);
      const pointer = !!over?.closest("button:not(:disabled), a") ||
        (!!over?.closest(".stage .phone") && (demo.stage === "unauthorized" || demo.stage === "waiting"));
      if (pointer !== hand) {
        hand = pointer;
        el.innerHTML = hand ? HAND : ARROW;
      }
      el.style.translate = `${pos.x - (hand ? 8 : 3)}px ${pos.y - 2}px`;
    };
    place();

    const ease = (k) => (k < 0.5 ? 4 * k * k * k : 1 - Math.pow(-2 * k + 2, 3) / 2);
    demo.cursor = {
      show() {
        if (!visitor) el.classList.add("on");
      },
      hide() {
        el.classList.remove("on");
      },
      /** Glides to a point with a slight arc, which reads as a hand, not a robot. */
      glide(x, y, ms = 800) {
        const from = { ...pos };
        const t0 = performance.now();
        return new Promise((resolve) => {
          const step = (now) => {
            const k = Math.min(1, (now - t0) / ms);
            const e = ease(k);
            pos = { x: from.x + (x - from.x) * e, y: from.y + (y - from.y) * e - Math.sin(k * Math.PI) * 16 };
            place();
            if (k < 1) requestAnimationFrame(step);
            else resolve();
          };
          requestAnimationFrame(step);
        });
      },
      /** Glides to an element and clicks it. */
      async click(target) {
        if (!target) return;
        const r = target.getBoundingClientRect();
        await demo.cursor.glide(r.x + r.width / 2 + 4, r.y + r.height / 2 + 3);
        await sleep(220);
        // Hidden meanwhile: the visitor took over, so the script must not click.
        if (!el.classList.contains("on")) return;
        el.classList.add("down");
        await sleep(110);
        el.classList.remove("down");
        target.click();
      },
      home() {
        return demo.cursor.glide(innerWidth * 0.78, innerHeight * 0.88, 900);
      },
    };
  });
})();
