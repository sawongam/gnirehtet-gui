<script lang="ts">
  import { onMount } from "svelte";
  import { base } from "$app/paths";
  import { theme } from "$lib/theme.svelte";
  import { inview } from "$lib/reveal";
  import { VERSION } from "$lib/site";

  type Stage = "unplugged" | "unauthorized" | "ready" | "starting" | "waiting" | "connected";
  type Demo = {
    stage: Stage;
    onstage: ((stage: Stage) => void) | null;
    onvisitor: (() => void) | null;
    plug(): void;
    allow(): void;
    accept(): void;
    unplug(): void;
    reset(): void;
    button(label: string): HTMLButtonElement | undefined;
    cursor: {
      show(): void;
      hide(): void;
      glide(x: number, y: number, ms?: number): Promise<void>;
      click(target: Element | null | undefined): Promise<void>;
      home(): Promise<void>;
    };
  };

  // The app window's own size; the frame scales it down to fit.
  const W = 760;
  const H = 600;
  const TITLEBAR = 40;

  let frame: HTMLIFrameElement;
  let demo = $state<Demo | null>(null);
  let stage = $state<Stage>("unplugged");
  let width = $state(W);
  let frameSrc = $state<string>();
  let loaded = $state(false);
  let visible = $state(false);
  /** The visitor took over: no more autoplay. */
  let manual = $state(false);

  const scale = $derived(Math.min(1, width / W));

  const steps = [
    { title: "Plug in the phone", hint: "With USB debugging on", action: "Plug in" },
    { title: "Allow USB debugging", hint: "A one-time prompt on the phone", action: "Tap Allow" },
    { title: "Click Connect", hint: "In the app", action: "Click" },
    { title: "Accept the VPN request", hint: "And the phone is online", action: "Tap OK" },
  ];
  const current = $derived(
    ({ unplugged: 0, unauthorized: 1, ready: 2, starting: 2, waiting: 3, connected: 4 } as const)[stage],
  );

  function act(index: number) {
    if (!demo || index !== current) return;
    takeOver();
    if (index === 0) demo.plug();
    else if (index === 1) demo.allow();
    else if (index === 2) demo.button("Connect")?.click();
    else if (index === 3) demo.accept();
  }

  function startOver() {
    takeOver();
    demo?.reset();
  }

  function takeOver() {
    manual = true;
    demo?.cursor.hide();
  }

  // ---- Autoplay ------------------------------------------------------------

  const reducedMotion = typeof matchMedia !== "undefined" && matchMedia("(prefers-reduced-motion: reduce)").matches;

  class Stopped extends Error {}

  /** Waits `ms` of on-screen time: the script holds while the demo is scrolled away. */
  async function wait(ms: number) {
    let left = ms;
    while (left > 0) {
      if (manual) throw new Stopped();
      const step = Math.min(left, 100);
      await new Promise((r) => setTimeout(r, step));
      if (visible && !document.hidden) left -= step;
    }
    if (manual) throw new Stopped();
  }

  async function until(target: Stage) {
    while (stage !== target) await wait(100);
  }

  /** The prompt's Allow or OK button on the phone illustration. */
  function phone() {
    return frame.contentDocument?.querySelector(".stage .phone .go") ?? null;
  }

  async function autoplay(d: Demo) {
    try {
      while (true) {
        d.reset();
        await wait(1400);
        d.plug();
        d.cursor.show();
        await wait(1500);
        await d.cursor.click(phone());
        await wait(1500);
        await d.cursor.click(d.button("Connect"));
        await until("waiting");
        await wait(1400);
        await d.cursor.click(phone());
        await wait(300);
        await d.cursor.home();
        await wait(5200);
        await d.cursor.click(d.button("Disconnect"));
        await until("ready");
        await wait(900);
        await d.cursor.home();
        d.unplug();
        await wait(1600);
      }
    } catch (e) {
      if (!(e instanceof Stopped)) throw e;
    }
  }

  onMount(() => {
    const onLoad = () => {
      const d = (frame.contentWindow as (Window & { __demo?: Demo }) | null)?.__demo;
      if (!d) return;
      d.onstage = (s) => (stage = s);
      d.onvisitor = takeOver;
      stage = d.stage;
      demo = d;
      // Give the app a moment to lay out before revealing it.
      setTimeout(() => (loaded = true), 250);
      if (!reducedMotion) void autoplay(d);
      else manual = true;
    };
    frame.addEventListener("load", onLoad);
    // Set here, not in markup, so the app starts in the page's theme and a
    // later theme switch does not reload it.
    frameSrc = `${base}/demo/?version=${VERSION}${theme.dark ? "&theme=dark" : ""}`;
    return () => frame.removeEventListener("load", onLoad);
  });

  // Follow the page's theme without reloading the app.
  $effect(() => {
    const dark = theme.dark;
    const doc = frame?.contentDocument;
    if (!loaded || !doc) return;
    doc.documentElement.classList.toggle("dark", dark);
    try {
      const key = "gnirehtet-gui.settings";
      const settings = JSON.parse(localStorage.getItem(key) ?? "{}");
      localStorage.setItem(key, JSON.stringify({ ...settings, theme: dark ? "dark" : "light" }));
    } catch {
      // Only affects the next visit.
    }
  });

</script>

<div class="demo" use:inview={(v) => (visible = v)}>
  <div class="backdrop">
    <div class="fit" bind:clientWidth={width} style:height="{(H + TITLEBAR) * scale}px">
      <div class="window" class:loaded style:scale style:width="{W}px" style:height="{H + TITLEBAR}px">
        <div class="titlebar">
          <span class="title">Gnirehtet</span>
          <span class="close" aria-hidden="true">
            <svg viewBox="0 0 16 16"><path d="M4.5 4.5l7 7M11.5 4.5l-7 7" /></svg>
          </span>
        </div>
        <iframe
          bind:this={frame}
          src={frameSrc}
          title="Live demo of the Gnirehtet app"
          width={W}
          height={H}
        ></iframe>
      </div>
    </div>
  </div>

  <div class="rail">
    <ol class="steps" aria-label="Try it">
      {#each steps as step, i (i)}
        <li class:done={i < current} class:now={i === current}>
          <span class="num" aria-hidden="true">
            {#if i < current}
              <svg viewBox="0 0 16 16"><path d="M3.5 8.5l3 3 6-7" /></svg>
            {:else}
              {i + 1}
            {/if}
          </span>
          <span class="text">
            <span class="step-title">{step.title}</span>
            <span class="hint">{step.hint}</span>
          </span>
          {#if i === current && stage !== "starting"}
            <button type="button" class="btn small suggested try" onclick={() => act(i)}>{step.action}</button>
          {/if}
        </li>
      {/each}
    </ol>
    <p class="status" aria-live="polite">
      {#if stage === "connected"}
        <span class="dot"></span> The phone is online through the computer.
        <button type="button" class="link" onclick={startOver}>Start over</button>
      {:else if manual}
        Your turn. Click the steps, or the app and phone themselves.
      {:else}
        This is the real app, running on a scripted phone. Click anything to take over.
      {/if}
    </p>
  </div>
</div>

<style>
  .demo {
    display: grid;
    gap: 28px;
  }

  .backdrop {
    position: relative;
    padding: clamp(20px, 6vw, 72px) clamp(12px, 5vw, 64px);
    border-radius: clamp(20px, 3vw, 32px);
    background: var(--wallpaper);
    overflow: hidden;
    isolation: isolate;
  }
  /* A faint grain keeps the large gradient from banding. */
  .backdrop::after {
    content: "";
    position: absolute;
    inset: 0;
    z-index: -1;
    opacity: 0.35;
    background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='160' height='160'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='.9' numOctaves='2' stitchTiles='stitch'/><feColorMatrix values='0 0 0 0 0.5  0 0 0 0 0.5  0 0 0 0 0.5  0 0 0 .18 0'/></filter><rect width='100%' height='100%' filter='url(%23n)'/></svg>");
    pointer-events: none;
  }

  .fit {
    position: relative;
    max-width: 760px;
    margin: 0 auto;
  }

  .window {
    position: absolute;
    top: 0;
    left: 50%;
    translate: -50% 0;
    transform-origin: top center;
    display: flex;
    flex-direction: column;
    border-radius: 14px;
    overflow: hidden;
    background: var(--titlebar);
    box-shadow: var(--window-shadow);
  }

  .titlebar {
    position: relative;
    display: grid;
    place-items: center;
    flex: none;
    height: 40px;
    color: var(--titlebar-fg);
    font-family: var(--font-ui);
    font-size: 14.5px;
    font-weight: 700;
    user-select: none;
  }
  .close {
    position: absolute;
    right: 10px;
    top: 50%;
    translate: 0 -50%;
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: var(--fill);
  }
  .close svg {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.6;
    stroke-linecap: round;
  }

  iframe {
    display: block;
    flex: 1;
    border: 0;
    background: var(--page);
    opacity: 0;
    transition: opacity 600ms var(--ease);
  }
  .loaded iframe {
    opacity: 1;
  }

  /* ---------- Steps ---------- */

  .rail {
    display: grid;
    gap: 16px;
  }

  .steps {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 10px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .steps li {
    position: relative;
    display: grid;
    grid-template-columns: auto 1fr;
    align-content: start;
    gap: 4px 12px;
    min-height: 116px;
    padding: 16px;
    border-radius: var(--radius-row);
    color: var(--fg-faint);
    transition:
      background 400ms var(--ease),
      color 400ms var(--ease),
      box-shadow 400ms var(--ease);
  }
  .steps li.done {
    color: var(--fg-dim);
  }
  .steps li.now {
    color: var(--fg);
    background: var(--raised);
    box-shadow: var(--card-shadow);
  }

  .num {
    display: grid;
    place-items: center;
    width: 26px;
    height: 26px;
    border-radius: 50%;
    font-size: 13px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    background: var(--fill);
    transition: background 400ms var(--ease), color 400ms var(--ease);
  }
  .done .num {
    color: var(--success);
    background: color-mix(in oklab, var(--success-dot) 18%, transparent);
  }
  .now .num {
    color: #fff;
    background: var(--accent-bg);
  }
  .num svg {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .text {
    display: grid;
    gap: 2px;
    padding-top: 2px;
  }
  .step-title {
    font-weight: 650;
    line-height: 1.3;
  }
  .hint {
    font-size: 14px;
    color: var(--fg-faint);
  }

  .try {
    grid-column: 2;
    justify-self: start;
    margin-top: 8px;
    animation: pop 400ms var(--ease-out);
  }
  @keyframes pop {
    from {
      opacity: 0;
      scale: 0.9;
    }
  }

  .status {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: center;
    gap: 6px 10px;
    min-height: 26px;
    font-size: 14.5px;
    color: var(--fg-dim);
    text-align: center;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--success-dot);
    box-shadow: 0 0 0 4px color-mix(in oklab, var(--success-dot) 22%, transparent);
  }
  .link {
    padding: 0;
    border: 0;
    font: inherit;
    font-weight: 650;
    color: var(--accent);
    background: none;
    cursor: pointer;
  }
  .link:hover {
    text-decoration: underline;
    text-underline-offset: 3px;
  }

  @media (max-width: 860px) {
    .steps {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
  @media (max-width: 520px) {
    .steps {
      grid-template-columns: 1fr;
      gap: 4px;
    }
    .steps li {
      min-height: 0;
      padding: 12px 14px;
    }
    /* Only the step at hand on small screens; the rest is one line each. */
    .steps li:not(.now) .hint {
      display: none;
    }
  }
</style>
