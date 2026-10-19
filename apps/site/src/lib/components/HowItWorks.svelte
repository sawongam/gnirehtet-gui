<script lang="ts">
  import { onMount } from "svelte";
  import { inview, reveal } from "$lib/reveal";
  import ReverseWord from "./ReverseWord.svelte";

  type Part = "relay" | "client" | "tunnel" | "vpn" | "traffic";

  const steps: { part: Part; title: string; text: string }[] = [
    {
      part: "relay",
      title: "Starts the relay",
      text: "The gnirehtet relay runs on your computer and passes the phone's traffic on to the network.",
    },
    {
      part: "client",
      title: "Installs the client",
      text: "The first time you connect, the app puts the gnirehtet Android client on the phone.",
    },
    {
      part: "tunnel",
      title: "Opens a tunnel through the cable",
      text: "An adb reverse tunnel links the phone to the relay over USB.",
    },
    {
      part: "vpn",
      title: "Starts the VPN on the phone",
      text: "You accept Android's VPN request, and from then on every app on the phone goes through the tunnel.",
    },
    {
      part: "traffic",
      title: "Keeps it running, then cleans up",
      text: "A pulled cable is picked up again when you plug it back in. Disconnect or quit, and both ends shut down.",
    },
  ];

  const CYCLE_MS = 3600;

  let active = $state(0);
  let pinned = $state(false);
  let visible = $state(false);
  let svg: SVGSVGElement;

  const part = $derived(steps[active].part);

  onMount(() => {
    const timer = setInterval(() => {
      if (visible && !pinned && !document.hidden) active = (active + 1) % steps.length;
    }, CYCLE_MS);
    return () => clearInterval(timer);
  });

  // SMIL animations only run while the diagram is on screen.
  $effect(() => {
    if (visible) svg?.unpauseAnimations();
    else svg?.pauseAnimations();
  });

  function pick(i: number) {
    active = i;
    pinned = true;
  }
</script>

<section id="how" class="how">
  <div class="wrap">
    <header class="head">
      <p class="eyebrow" use:reveal>How it works</p>
      <h2 class="section-title" use:reveal={60}>
        <span class="word"><ReverseWord /></span>
        <span class="sub">is tethering, the other way round.</span>
      </h2>
      <p class="lede" use:reveal={120}>
        Tethering usually shares a phone's mobile data with a computer. Gnirehtet goes the other way: the phone
        uses the computer's connection, through the USB cable. Here is what the app does when you click Connect.
      </p>
    </header>

    <div class="body" use:inview={(v) => (visible = v)}>
      <figure class="diagram on-{part}" use:reveal={80}>
        <svg bind:this={svg} viewBox="0 0 640 360" role="img" aria-label="Phone, USB cable, computer, internet">
          <defs>
            <!-- Along the cable, under the computer, then out to the network. -->
            <path id="route-out" d="M75 262 C 75 345, 220 335, 270 296 L 345 268 L 470 268 C 520 268, 540 180, 556 178" />
            <path id="route-in" d="M556 178 C 540 180, 520 268, 470 268 L 345 268 L 270 296 C 220 335, 75 345, 75 262" />
            <path id="route-apk" d="M345 268 L 270 296 C 220 335, 75 345, 75 262" />
          </defs>

          <!-- Network: the computer's own connection -->
          <g class="part net">
            <path class="link" d="M470 268 C 520 268, 540 180, 560 180" />
            <circle class="globe" cx="580" cy="168" r="34" />
            <ellipse class="globe-line" cx="580" cy="168" rx="14" ry="34" />
            <path class="globe-line" d="M546 168 H614 M552 150 H608 M552 186 H608" />
            <text x="580" y="226" text-anchor="middle">Internet</text>
          </g>

          <!-- Cable -->
          <g class="part tunnel">
            <path class="cable" d="M75 262 C 75 345, 220 335, 270 296" />
            <path class="tunnel-line" d="M75 262 C 75 345, 220 335, 270 296" />
            <text x="160" y="352" text-anchor="middle">USB</text>
          </g>

          <!-- Computer -->
          <g class="part relay">
            <rect class="device" x="230" y="90" width="230" height="150" rx="12" />
            <rect class="screen" x="240" y="100" width="210" height="130" rx="5" />
            <path class="device" d="M200 252 H490 L478 268 H212 Z" />
            <g class="relay-chip">
              <rect x="285" y="148" width="120" height="34" rx="17" />
              <text x="345" y="170" text-anchor="middle">relay :31416</text>
            </g>
          </g>

          <!-- Phone -->
          <g class="part client vpn">
            <rect class="device" x="40" y="70" width="70" height="192" rx="14" />
            <rect class="screen" x="45" y="76" width="60" height="180" rx="10" />
            <g class="apk">
              <rect x="57" y="138" width="36" height="36" rx="9" />
              <path d="M66 156 h18 M75 147 v18" />
            </g>
            <g class="vpn-chip">
              <rect x="52" y="84" width="30" height="14" rx="4" />
              <text x="67" y="94.5" text-anchor="middle">VPN</text>
            </g>
            <g class="online">
              <circle cx="75" cy="160" r="15" />
              <path d="M68 160.5 l4.6 4.4 l9-10" />
            </g>
          </g>

          <!-- Moving parts -->
          <g class="packets apk-packets">
            <circle r="5">
              <animateMotion dur="1.6s" repeatCount="indefinite"><mpath href="#route-apk" /></animateMotion>
            </circle>
          </g>
          <g class="packets traffic-packets">
            {#each [0, 0.6, 1.2, 1.8] as delay (delay)}
              <circle class="out" r="4.5">
                <animateMotion dur="2.4s" begin="{delay}s" repeatCount="indefinite"><mpath href="#route-out" /></animateMotion>
              </circle>
              <circle class="in" r="4.5">
                <animateMotion dur="2.4s" begin="{delay + 0.3}s" repeatCount="indefinite"><mpath href="#route-in" /></animateMotion>
              </circle>
            {/each}
          </g>
        </svg>
      </figure>

      <ol class="steps">
        {#each steps as step, i (step.part)}
          <li class:active={i === active} use:reveal={i * 60}>
            <button
              type="button"
              onclick={() => pick(i)}
              onmouseenter={() => pick(i)}
              aria-current={i === active ? "step" : undefined}
            >
              <span class="bar" aria-hidden="true">
                <span class="fill" class:running={i === active && !pinned && visible} style:--cycle="{CYCLE_MS}ms"></span>
              </span>
              <span class="num" aria-hidden="true">{i + 1}</span>
              <span class="title">{step.title}</span>
              <span class="text"><span>{step.text}</span></span>
            </button>
          </li>
        {/each}
      </ol>
    </div>
  </div>
</section>

<style>
  .how {
    padding: clamp(96px, 14vw, 180px) 0 clamp(64px, 10vw, 120px);
  }

  .head {
    display: grid;
    gap: 22px;
    max-width: 780px;
  }

  h2 {
    display: grid;
    gap: 6px;
  }
  .word {
    font-size: clamp(56px, 11vw, 132px);
    line-height: 1;
    letter-spacing: -0.045em;
    font-weight: 700;
    color: var(--accent-bg);
  }
  :global(:root.dark) .word {
    color: var(--accent);
  }
  .sub {
    font-size: clamp(28px, 4vw, 46px);
  }

  .body {
    display: grid;
    grid-template-columns: minmax(0, 1.15fr) minmax(0, 1fr);
    gap: clamp(32px, 6vw, 80px);
    align-items: center;
    margin-top: clamp(48px, 7vw, 88px);
  }

  /* ---------- Diagram ---------- */

  figure {
    margin: 0;
    padding: clamp(12px, 3vw, 32px);
    border-radius: 28px;
    background: var(--diagram);
  }
  svg {
    display: block;
    width: 100%;
    height: auto;
    overflow: visible;
  }

  text {
    font-family: var(--font-ui);
    font-size: 13px;
    font-weight: 650;
    fill: var(--fg-dim);
  }

  .part {
    transition: opacity 500ms var(--ease);
  }

  .device {
    fill: var(--device);
  }
  .screen {
    fill: var(--device-screen);
  }
  .cable,
  .link {
    fill: none;
    stroke: var(--cable);
    stroke-width: 7;
    stroke-linecap: round;
  }
  .link {
    stroke-width: 3;
    stroke-dasharray: 2 7;
  }
  .tunnel-line {
    fill: none;
    stroke: var(--accent-bg);
    stroke-width: 3;
    stroke-linecap: round;
    stroke-dasharray: 8 10;
    opacity: 0;
    transition: opacity 400ms var(--ease);
    animation: dash 900ms linear infinite;
  }
  @keyframes dash {
    to {
      stroke-dashoffset: -18;
    }
  }
  .globe,
  .globe-line {
    fill: none;
    stroke: var(--fg-faint);
    stroke-width: 2;
  }
  .globe {
    fill: var(--raised);
  }

  .relay-chip rect {
    fill: var(--accent-bg);
  }
  .relay-chip text {
    fill: #fff;
    font-family: var(--font-mono);
    font-size: 13px;
  }
  .relay-chip,
  .apk,
  .vpn-chip,
  .online {
    opacity: 0;
    transform-box: fill-box;
    transform-origin: center;
    scale: 0.8;
    transition:
      opacity 400ms var(--ease),
      scale 500ms cubic-bezier(0.34, 1.4, 0.64, 1);
  }
  .apk rect {
    fill: var(--success-dot);
  }
  .apk path,
  .online path {
    fill: none;
    stroke: #fff;
    stroke-width: 3;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .vpn-chip rect {
    fill: var(--fg);
  }
  .vpn-chip text {
    fill: var(--page);
    font-size: 9px;
    font-weight: 800;
  }
  .online circle {
    fill: var(--success-dot);
  }

  .packets circle {
    fill: var(--accent-bg);
  }
  .packets circle.in {
    fill: var(--success-dot);
  }
  .packets {
    opacity: 0;
    transition: opacity 400ms var(--ease);
  }

  /* What each step lights up. Everything else steps back. */
  .on-relay .part:not(.relay),
  .on-client .part:not(.client),
  .on-tunnel .part:not(.tunnel):not(.relay):not(.client),
  .on-vpn .part:not(.vpn) {
    opacity: 0.35;
  }
  .on-relay .relay-chip,
  .on-tunnel .relay-chip,
  .on-vpn .relay-chip,
  .on-traffic .relay-chip,
  .on-client .apk,
  .on-vpn .vpn-chip,
  .on-traffic .vpn-chip,
  .on-traffic .online {
    opacity: 1;
    scale: 1;
  }
  .on-tunnel .tunnel-line,
  .on-vpn .tunnel-line {
    opacity: 1;
  }
  .on-client .apk-packets,
  .on-traffic .traffic-packets {
    opacity: 1;
  }

  /* ---------- Steps ---------- */

  .steps {
    display: grid;
    gap: 2px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .steps button {
    position: relative;
    display: grid;
    grid-template-columns: 28px 1fr;
    gap: 2px 14px;
    width: 100%;
    padding: 14px 16px 14px 20px;
    border: 0;
    border-radius: var(--radius-row);
    font: inherit;
    text-align: left;
    color: var(--fg-faint);
    background: transparent;
    cursor: pointer;
    transition:
      background 300ms var(--ease),
      color 300ms var(--ease);
  }
  .steps button:hover {
    color: var(--fg-dim);
  }
  .active button {
    color: var(--fg);
    background: var(--raised);
    box-shadow: var(--card-shadow);
  }

  .bar {
    position: absolute;
    left: 8px;
    top: 16px;
    bottom: 16px;
    width: 3px;
    border-radius: 2px;
    overflow: hidden;
  }
  .fill {
    display: block;
    height: 100%;
    background: var(--accent-bg);
    transform-origin: top;
    scale: 1 0;
  }
  .active .fill {
    scale: 1 1;
  }
  .fill.running {
    animation: grow var(--cycle) linear;
  }
  @keyframes grow {
    from {
      scale: 1 0;
    }
  }

  .num {
    font-family: var(--font-display);
    font-size: 15px;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    line-height: 1.5;
  }
  .title {
    font-weight: 650;
    font-size: 16.5px;
  }
  .text {
    grid-column: 2;
    display: grid;
    grid-template-rows: 0fr;
    font-size: 15px;
    color: var(--fg-dim);
    overflow: hidden;
    opacity: 0;
    transition:
      grid-template-rows 450ms var(--ease-out),
      opacity 300ms var(--ease);
  }
  .text > span {
    min-height: 0;
  }
  .active .text {
    grid-template-rows: 1fr;
    opacity: 1;
    padding-top: 4px;
  }

  @media (max-width: 860px) {
    .body {
      grid-template-columns: 1fr;
    }
  }
</style>
