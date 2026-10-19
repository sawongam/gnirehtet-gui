<script lang="ts" module>
  /**
   * What the illustration shows. The phone's screen mirrors what the user
   * should be looking at on the real phone (the prompt to accept, the VPN
   * indicator), so the picture doubles as the instructions.
   */
  export type Scene =
    | "absent"
    | "unauthorized"
    | "unusable"
    | "ready"
    | "starting"
    | "waiting"
    | "connected"
    | "interrupted"
    | "failed"
    | "stopping";
</script>

<script lang="ts">
  import { fade } from "svelte/transition";

  let { scene, clock }: { scene: Scene; clock: string } = $props();

  const present = $derived(scene !== "absent");
  const plugged = $derived(scene !== "absent" && scene !== "interrupted");
  // One period of a traffic curve, repeated twice so it can scroll seamlessly.
  const WAVE = "C4 22 8 20 14 18 S22 4 28 10 S36 24 44 20 S52 16 58 18 S66 2 74 8 S84 26 92 20 S96 22 100 22";
  const TRAFFIC = `M0 22 ${WAVE} ${WAVE.replace(/(\d+) (\d+)/g, (_, x, y) => `${+x + 100} ${y}`)}`;

  const flowing = $derived(scene === "connected" || scene === "starting");
  const vpn = $derived(scene === "connected" || scene === "waiting" || scene === "interrupted" || scene === "failed");
  const halo = $derived(
    ({ connected: "success", waiting: "warning", unauthorized: "warning", failed: "danger", interrupted: "warning" } as const)[
      scene as string
    ] ?? "",
  );
</script>

<div class="stage {scene}" aria-hidden="true">
  <div class="halo {halo}"></div>

  <div class="phone" class:present>
    <div class="outline"></div>
    <div class="body">
      <div class="screen">
        <div class="statusbar">
          <span class="numeric">{clock}</span>
          <span class="vpn" class:on={vpn}>VPN</span>
        </div>

        {#key scene}
          <div class="content" in:fade={{ duration: 260, delay: 120 }} out:fade={{ duration: 120 }}>
            {#if scene === "unauthorized"}
              <div class="prompt">
                <p class="prompt-title">Allow USB debugging?</p>
                <p class="prompt-text">Always allow from this computer</p>
                <div class="prompt-buttons"><span>Cancel</span><span class="go">Allow</span></div>
              </div>
            {:else if scene === "waiting"}
              <div class="prompt">
                <p class="prompt-title">Connection request</p>
                <p class="prompt-text">Gnirehtet wants to set up a VPN connection.</p>
                <div class="prompt-buttons"><span>Cancel</span><span class="go">OK</span></div>
              </div>
            {:else if scene === "starting"}
              <div class="loader"><span></span></div>
            {:else if scene === "connected"}
              <div class="online">
                <div class="badge">
                  <svg viewBox="0 0 24 24"><path d="M6.5 12.5l3.7 3.6 7.3-8" /></svg>
                </div>
                <p class="online-title">Connected</p>
                <p class="online-sub">Internet via USB</p>
              </div>
              <!-- Like Android's ongoing VPN notification, with live traffic. -->
              <div class="traffic">
                <div class="traffic-head"><span>Gnirehtet</span><span class="live">Live</span></div>
                <svg class="graph" viewBox="0 0 100 32" preserveAspectRatio="none">
                  <defs>
                    <linearGradient id="traffic-fill" x1="0" y1="0" x2="0" y2="1">
                      <stop offset="0" stop-color="currentColor" stop-opacity="0.35" />
                      <stop offset="1" stop-color="currentColor" stop-opacity="0" />
                    </linearGradient>
                  </defs>
                  <g class="graph-scroll">
                    <path class="area" d="{TRAFFIC} L200 32 L0 32 Z" fill="url(#traffic-fill)" />
                    <path class="line" d={TRAFFIC} />
                  </g>
                </svg>
              </div>
            {:else if scene === "failed"}
              <p class="note danger">No internet</p>
            {:else if scene === "unusable"}
              <p class="note">Not available</p>
            {/if}
          </div>
        {/key}
      </div>
    </div>
  </div>

  <div class="cable-slot">
    <div class="cable" class:plugged class:flowing>
      <div class="plug"></div>
      <div class="wire">
        <span class="packet"></span>
        <span class="packet p2"></span>
        <span class="packet p3"></span>
      </div>
    </div>
  </div>
</div>

<style>
  .stage {
    /* Sized from the space <main> leaves after the text and buttons (~380px). */
    --phone-w: clamp(92px, calc((100cqh - 380px) / 2.04), 142px);
    --phone-h: calc(var(--phone-w) * 2.04);
    --spring: cubic-bezier(0.34, 1.4, 0.64, 1);

    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
    padding-top: 12px;
  }

  /* ---------- Halo ---------- */

  /* A soft radial wash, not a blurred disc: no filter, so it composites
     cheaply, and the tint lives in the gradient stops so it fades out
     smoothly instead of ending in a visible rim. */
  @property --halo {
    syntax: "<color>";
    inherits: false;
    initial-value: transparent;
  }
  .halo {
    --halo: transparent;
    --strength: 28%;
    position: absolute;
    top: calc(12px + var(--phone-h) / 2);
    left: 50%;
    width: calc(var(--phone-w) * 3.6);
    aspect-ratio: 1;
    translate: -50% -50%;
    opacity: 0;
    scale: 0.85;
    pointer-events: none;
    /* Bell-shaped falloff: many stops so no ring shows where two meet. */
    background: radial-gradient(
      closest-side,
      color-mix(in oklab, var(--halo) var(--strength), transparent) 0%,
      color-mix(in oklab, var(--halo) calc(var(--strength) * 0.93), transparent) 15%,
      color-mix(in oklab, var(--halo) calc(var(--strength) * 0.76), transparent) 30%,
      color-mix(in oklab, var(--halo) calc(var(--strength) * 0.54), transparent) 45%,
      color-mix(in oklab, var(--halo) calc(var(--strength) * 0.34), transparent) 60%,
      color-mix(in oklab, var(--halo) calc(var(--strength) * 0.17), transparent) 75%,
      color-mix(in oklab, var(--halo) calc(var(--strength) * 0.06), transparent) 88%,
      transparent 100%
    );
    will-change: opacity, scale;
    transition:
      --halo 700ms var(--ease),
      opacity 700ms var(--ease),
      scale 900ms var(--ease);
  }
  .halo.success,
  .halo.warning,
  .halo.danger {
    opacity: 1;
  }
  .halo.success {
    --halo: var(--success-dot);
    scale: 1;
    animation: breathe 5s ease-in-out infinite;
  }
  .halo.warning {
    --halo: var(--warning-dot);
    scale: 0.94;
  }
  .halo.danger {
    --halo: var(--danger-dot);
    --strength: 20%;
    scale: 0.92;
  }
  @keyframes breathe {
    50% {
      scale: 1.06;
    }
  }

  /* ---------- Phone ---------- */

  .phone {
    position: relative;
    z-index: 1;
    width: var(--phone-w);
    height: var(--phone-h);
  }

  .outline,
  .body {
    position: absolute;
    inset: 0;
    border-radius: calc(var(--phone-w) * 0.2);
    transition:
      opacity 420ms var(--ease),
      translate 620ms var(--spring),
      scale 620ms var(--spring);
  }

  .outline {
    border: 2px dashed var(--fg-faint);
    opacity: 1;
  }
  .present .outline {
    opacity: 0;
    scale: 1.04;
  }

  .body {
    padding: 5px;
    background: var(--phone-frame);
    box-shadow:
      inset 0 0 0 1px var(--phone-edge),
      0 18px 40px -12px rgb(0 0 6 / 0.35),
      0 4px 10px -4px rgb(0 0 6 / 0.2);
    opacity: 0;
    translate: 0 14px;
    scale: 0.96;
  }
  .present .body {
    opacity: 1;
    translate: 0 0;
    scale: 1;
  }

  /* Power button. */
  .body::after {
    content: "";
    position: absolute;
    top: 26%;
    right: -3px;
    width: 3px;
    height: 12%;
    border-radius: 0 2px 2px 0;
    background: var(--phone-frame);
  }

  .screen {
    position: relative;
    height: 100%;
    overflow: hidden;
    border-radius: calc(var(--phone-w) * 0.2 - 5px);
    background: var(--screen);
    transition: background-color 500ms var(--ease);
  }
  /* Wallpaper, faded out when the screen is busy with something else. */
  .screen::after {
    content: "";
    position: absolute;
    inset: 0;
    background: var(--wallpaper);
    opacity: 0;
    transition: opacity 600ms var(--ease);
  }
  .ready .screen::after {
    opacity: 1;
  }
  .statusbar,
  .content {
    z-index: 1;
  }
  .statusbar {
    position: relative;
  }
  .connected .screen {
    background: var(--screen-online);
  }
  .interrupted .screen,
  .stopping .screen {
    background: var(--screen-dim);
  }

  /* Camera. */
  .screen::before {
    content: "";
    position: absolute;
    top: 8px;
    left: 50%;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    translate: -50% 0;
    z-index: 2;
    background: var(--phone-frame);
  }

  .statusbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 6px 12px 0;
    height: 22px;
    font-size: 9px;
    font-weight: 700;
    color: var(--screen-fg);
  }

  .vpn {
    padding: 0 4px;
    border-radius: 3px;
    font-size: 7.5px;
    letter-spacing: 0.04em;
    background: var(--screen-fg);
    color: var(--screen);
    opacity: 0;
    scale: 0.6;
    transition:
      opacity 300ms var(--ease),
      scale 400ms var(--spring);
  }
  .vpn.on {
    opacity: 1;
    scale: 1;
  }
  .connected .vpn {
    color: var(--screen-online);
  }

  .content {
    position: absolute;
    inset: 22px 0 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 8px;
    color: var(--screen-fg);
  }

  /* Android-style dialog, sitting low on the screen like the real one. */
  .prompt {
    align-self: stretch;
    margin-top: auto;
    margin-bottom: 14%;
    padding: 9px 9px 7px;
    border-radius: 10px;
    background: var(--screen-card);
    text-align: left;
    box-shadow: 0 6px 16px -6px rgb(0 0 6 / 0.35);
    animation: rise 520ms var(--spring) both;
  }
  .prompt-title {
    font-size: 9px;
    font-weight: 800;
    line-height: 1.25;
  }
  .prompt-text {
    margin-top: 4px;
    font-size: 7.5px;
    line-height: 1.35;
    opacity: 0.7;
  }
  .prompt-buttons {
    display: flex;
    justify-content: flex-end;
    gap: 4px;
    margin-top: 8px;
    font-size: 8px;
    font-weight: 700;
  }
  .prompt-buttons span {
    padding: 2px 6px;
    border-radius: 999px;
    opacity: 0.6;
  }
  .prompt-buttons .go {
    opacity: 1;
    color: #fff;
    background: var(--accent-bg);
    animation: tap 2.2s var(--ease) infinite 600ms;
  }
  @keyframes rise {
    from {
      opacity: 0;
      translate: 0 18px;
    }
  }
  @keyframes tap {
    0%,
    60%,
    100% {
      box-shadow: 0 0 0 0 var(--accent-ring);
    }
    30% {
      box-shadow: 0 0 0 5px transparent;
    }
  }

  .loader {
    width: 46%;
    height: 3px;
    border-radius: 3px;
    overflow: hidden;
    background: color-mix(in srgb, var(--screen-fg) 15%, transparent);
  }
  .loader span {
    display: block;
    width: 40%;
    height: 100%;
    border-radius: 3px;
    background: var(--screen-fg);
    animation: sweep 1.2s var(--ease) infinite;
  }
  @keyframes sweep {
    from {
      translate: -100% 0;
    }
    to {
      translate: 260% 0;
    }
  }

  .online {
    display: flex;
    flex-direction: column;
    align-items: center;
    margin-top: auto;
  }
  .badge {
    position: relative;
    display: grid;
    place-items: center;
    width: calc(var(--phone-w) * 0.28);
    aspect-ratio: 1;
    border-radius: 50%;
    background: var(--success-dot);
    box-shadow: 0 4px 14px color-mix(in srgb, var(--success-dot) 45%, transparent);
    animation: pop 600ms var(--spring) both 150ms;
  }
  /* A single soft pulse every few seconds, rather than constant ripples. */
  .badge::after {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: 50%;
    border: 1.5px solid var(--success-dot);
    opacity: 0;
    animation: pulse 3.2s var(--ease) infinite 1.2s;
  }
  .badge svg {
    width: 55%;
    height: 55%;
    fill: none;
    stroke: #fff;
    stroke-width: 2.6;
    stroke-linecap: round;
    stroke-linejoin: round;
    stroke-dasharray: 20;
    stroke-dashoffset: 20;
    animation: draw 420ms var(--ease) forwards 520ms;
  }
  .online-title {
    margin-top: 9px;
    font-size: 11px;
    font-weight: 800;
    color: var(--screen-fg);
    animation: rise 480ms var(--ease) both 380ms;
  }
  .online-sub {
    margin-top: 1px;
    font-size: 8px;
    font-weight: 600;
    color: var(--success);
    animation: rise 480ms var(--ease) both 460ms;
  }
  @keyframes pop {
    from {
      opacity: 0;
      scale: 0.4;
    }
  }
  @keyframes draw {
    to {
      stroke-dashoffset: 0;
    }
  }
  @keyframes pulse {
    0% {
      opacity: 0.7;
      scale: 1;
    }
    55%,
    100% {
      opacity: 0;
      scale: 1.9;
    }
  }

  .traffic {
    align-self: stretch;
    margin-top: auto;
    margin-bottom: 4px;
    padding: 7px 8px 6px;
    border-radius: 9px;
    background: var(--screen-card);
    box-shadow: 0 1px 3px rgb(0 0 6 / 0.08);
    animation: rise 520ms var(--spring) both 600ms;
  }
  .traffic-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 7.5px;
    font-weight: 800;
    color: var(--screen-fg);
  }
  .live {
    display: flex;
    align-items: center;
    gap: 3px;
    font-weight: 700;
    color: var(--success);
  }
  .live::before {
    content: "";
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--success-dot);
    animation: blink 1.6s ease-in-out infinite;
  }
  @keyframes blink {
    50% {
      opacity: 0.25;
    }
  }
  .graph {
    display: block;
    width: 100%;
    height: 24px;
    margin-top: 5px;
    overflow: hidden;
    color: var(--success-dot);
  }
  .graph-scroll {
    animation: scroll 5s linear infinite;
  }
  .graph .line {
    fill: none;
    stroke: currentColor;
    stroke-width: 1.6;
    stroke-linejoin: round;
    vector-effect: non-scaling-stroke;
  }
  /* On a short window the phone shrinks: keep only the essentials. */
  @container (max-height: 640px) {
    .online-sub,
    .traffic-head span:first-child {
      display: none;
    }
    .online-title {
      margin-top: 6px;
    }
    .traffic {
      padding: 5px 6px 4px;
    }
    .graph {
      height: 16px;
      margin-top: 3px;
    }
  }

  @keyframes scroll {
    to {
      translate: -100px 0;
    }
  }

  .note {
    font-size: 9px;
    font-weight: 700;
    opacity: 0.6;
  }
  .note.danger {
    color: var(--danger);
    opacity: 1;
  }

  /* ---------- Cable ---------- */

  /* Keeps a lowered plug from running into the text below. */
  .cable-slot {
    height: 92px;
    overflow: hidden;
  }

  .cable {
    display: flex;
    flex-direction: column;
    align-items: center;
    height: 92px;
    translate: 0 30px;
    transition: translate 700ms var(--spring);
  }
  .cable.plugged {
    translate: 0 0;
  }
  /* Unplugged, the plug hangs and sways a little: "plug me in". */
  .absent .cable {
    animation: dangle 3.2s ease-in-out infinite 700ms;
  }
  @keyframes dangle {
    50% {
      translate: 0 22px;
    }
  }

  .plug {
    flex-shrink: 0;
    width: 22px;
    height: 26px;
    margin-top: 2px;
    border-radius: 4px 4px 6px 6px;
    background: var(--cable);
    box-shadow: inset 0 3px 0 -1px color-mix(in srgb, var(--cable) 60%, var(--window));
  }

  .wire {
    position: relative;
    flex: 1;
    width: 5px;
    border-radius: 0 0 3px 3px;
    background: linear-gradient(var(--cable), var(--cable) 30%, transparent);
    overflow: hidden;
  }

  /* Data travelling up into the phone. */
  .packet {
    position: absolute;
    left: 0;
    bottom: -12px;
    width: 5px;
    height: 12px;
    border-radius: 3px;
    background: var(--success-dot);
    opacity: 0;
  }
  .flowing .packet {
    animation: climb 1.5s cubic-bezier(0.45, 0, 0.55, 1) infinite;
  }
  .flowing .p2 {
    animation-delay: 0.5s;
  }
  .flowing .p3 {
    animation-delay: 1s;
  }
  .starting .packet {
    background: var(--accent-bg);
  }
  @keyframes climb {
    0% {
      opacity: 0;
      translate: 0 0;
    }
    20% {
      opacity: 1;
    }
    100% {
      opacity: 1;
      translate: 0 -110px;
    }
  }
</style>
