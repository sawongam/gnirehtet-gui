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
                <span class="ring"></span>
                <span class="ring late"></span>
                <span class="online-label">Online</span>
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

  .halo {
    position: absolute;
    top: calc(12px + var(--phone-h) / 2);
    left: 50%;
    width: calc(var(--phone-w) * 2.6);
    aspect-ratio: 1;
    border-radius: 50%;
    translate: -50% -50%;
    opacity: 0;
    scale: 0.8;
    transition:
      opacity 600ms var(--ease),
      scale 900ms var(--ease),
      background-color 600ms var(--ease);
    filter: blur(28px);
  }
  .halo.success {
    background: var(--success-dot);
    opacity: 0.22;
    scale: 1;
    animation: breathe 4s ease-in-out infinite;
  }
  .halo.warning {
    background: var(--warning-dot);
    opacity: 0.18;
    scale: 0.92;
  }
  .halo.danger {
    background: var(--danger-dot);
    opacity: 0.14;
    scale: 0.9;
  }
  @keyframes breathe {
    50% {
      scale: 1.08;
      opacity: 0.3;
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
    position: relative;
    display: grid;
    place-items: center;
    width: 100%;
    aspect-ratio: 1;
  }
  .ring {
    position: absolute;
    width: 30%;
    aspect-ratio: 1;
    border-radius: 50%;
    border: 1.5px solid var(--success-dot);
    opacity: 0;
    animation: ripple 2.6s var(--ease) infinite;
  }
  .ring.late {
    animation-delay: 1.3s;
  }
  .online-label {
    position: relative;
    font-size: 10px;
    font-weight: 800;
    letter-spacing: 0.02em;
    color: var(--success);
  }
  @keyframes ripple {
    0% {
      opacity: 0.9;
      scale: 0.6;
    }
    100% {
      opacity: 0;
      scale: 2.6;
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
