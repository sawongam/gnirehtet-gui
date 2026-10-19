<script lang="ts">
  import { base } from "$app/paths";
  import Nav from "$lib/components/Nav.svelte";
  import DownloadButton from "$lib/components/DownloadButton.svelte";
  import Demo from "$lib/components/Demo.svelte";
  import HowItWorks from "$lib/components/HowItWorks.svelte";
  import Downloads from "$lib/components/Downloads.svelte";
  import { reveal } from "$lib/reveal";
  import { DOCS, REPO, UPSTREAM } from "$lib/site";
  import { release } from "$lib/release.svelte";

  const uses = [
    {
      title: "Wired desk, no Wi-Fi",
      text: "Your computer is on Ethernet and there's no wireless network for the phone.",
    },
    {
      title: "No signal, no SIM",
      text: "The phone has no data plan, or no coverage where you're sitting.",
    },
    {
      title: "Testing on a real device",
      text: "The phone joins your computer's network, VPN included, while you test your app on it.",
    },
    {
      title: "Saving mobile data",
      text: "It's plugged in to charge anyway. Let it use the computer's connection meanwhile.",
    },
  ];

  // Each detail shows a small piece of the app, in its own words.
  type Sample =
    | { status: [string, string] }
    | { cycle: [string, string][] }
    | { row: [string, string] }
    | { log: [string, "info" | "error", string][] };

  const details: { title: string; text: string; sample: Sample }[] = [
    {
      title: "Explains what the phone needs",
      text: "Unauthorized, offline, or blocked by Linux udev permissions: the app says what's wrong and what to do.",
      sample: { status: ["Allow USB debugging", "Unlock the phone and accept the prompt."] },
    },
    {
      title: "Shows each state as it happens",
      text: "Starting, waiting for the VPN prompt, connected, interrupted.",
      sample: {
        cycle: [
          ["Starting", "Starting the relay and the client."],
          ["Accept on your phone", "Tap OK on the connection request."],
          ["Connected", "Pixel 8 is using this computer’s internet."],
        ],
      },
    },
    {
      title: "Recovers from a pulled cable",
      text: "Plug it back in and the USB tunnel is restored without a click.",
      sample: { status: ["Cable unplugged", "Plug it back in and it resumes by itself."] },
    },
    {
      title: "Leaves nothing running",
      text: "Disconnecting or quitting stops the VPN on the phone and the relay on the computer.",
      sample: { status: ["Disconnecting", "Turning off the VPN on Pixel 8."] },
    },
    {
      title: "Preferences when you need them",
      text: "Custom DNS servers, routes and relay port. Light, dark, or follow your system.",
      sample: { row: ["DNS servers", "1.1.1.1, 9.9.9.9"] },
    },
    {
      title: "A log for troubleshooting",
      text: "Every relay and adb step, with errors flagged.",
      sample: {
        log: [
          ["12:04:31", "info", "adb reverse tcp:31416"],
          ["12:04:33", "error", "device unauthorized"],
        ],
      },
    },
  ];

  const TITLE = "Gnirehtet GUI: your computer's internet on your Android phone, over USB";
  const DESCRIPTION =
    "A desktop app for reverse tethering. Plug in an Android phone, click Connect, and it uses your computer's internet. No root, for Windows, macOS and Linux.";
</script>

<svelte:head>
  <title>{TITLE}</title>
  <meta name="description" content={DESCRIPTION} />
  <meta property="og:title" content="Gnirehtet GUI" />
  <meta property="og:description" content={DESCRIPTION} />
  <meta property="og:type" content="website" />
  <meta property="og:image" content="https://gnirehtet.sangamadhikari.com/og.png" />
  <meta name="twitter:card" content="summary_large_image" />
</svelte:head>

<Nav />

<main>
  <section class="hero">
    <div class="wrap intro">
      <a class="release" href={release.url}>Version {release.version} is out</a>
      <p class="name">Gnirehtet GUI</p>
      <h1>Your computer's internet, on your Android phone.</h1>
      <p class="lede">
        Plug the phone in with a USB cable and click Connect. It works without root, and there's nothing to set up
        on the phone.
      </p>
      <DownloadButton />
      <p class="platforms">For Windows, macOS and Linux</p>
    </div>
    <div class="wrap stage">
      <Demo />
    </div>
  </section>

  <HowItWorks />

  <section class="uses">
    <div class="wrap">
      <p class="eyebrow" use:reveal>When it's useful</p>
      <ul>
        {#each uses as use, i (use.title)}
          <li use:reveal={i * 70}>
            <h3>{use.title}</h3>
            <p>{use.text}</p>
          </li>
        {/each}
      </ul>
    </div>
  </section>

  <section class="details">
    <div class="wrap">
      <header>
        <p class="eyebrow" use:reveal>The details</p>
        <h2 class="section-title" use:reveal={60}>The terminal parts, handled.</h2>
        <p class="lede" use:reveal={120}>
          gnirehtet is a command-line tool. This app runs it for you, watches the phone, and tidies up afterwards.
        </p>
      </header>
      <ul class="tiles">
        {#each details as d, i (d.title)}
          <li use:reveal={(i % 3) * 70}>
            <div class="sample" aria-hidden="true">
              {#if "status" in d.sample}
                <span class="status-title">{d.sample.status[0]}</span>
                <span class="status-detail">{d.sample.status[1]}</span>
              {:else if "cycle" in d.sample}
                <span class="cycle">
                  {#each d.sample.cycle as [title, detail], j (title)}
                    <span style:--i={j} style:--n={d.sample.cycle.length}>
                      <span class="status-title">{title}</span>
                      <span class="status-detail">{detail}</span>
                    </span>
                  {/each}
                </span>
              {:else if "row" in d.sample}
                <span class="row"><span>{d.sample.row[0]}</span><span>{d.sample.row[1]}</span></span>
              {:else}
                {#each d.sample.log as [time, level, text] (time)}
                  <code class={level}><span>{time}</span>{text}</code>
                {/each}
              {/if}
            </div>
            <h3>{d.title}</h3>
            <p>{d.text}</p>
          </li>
        {/each}
      </ul>
    </div>
  </section>

  <Downloads />

  <section class="credit">
    <div class="wrap" use:reveal>
      <p>
        Built on <a href={UPSTREAM}>gnirehtet</a> by Genymobile. The app ships the unmodified 2.5.1 relay and Android
        client, and adds the desktop interface and process management around them.
      </p>
    </div>
  </section>
</main>

<footer>
  <div class="wrap foot">
    <span class="brand">
      <img src="{base}/icon.svg" alt="" width="22" height="22" />
      Gnirehtet GUI
    </span>
    <nav aria-label="Project">
      <a href={REPO}>Source code</a>
      <a href="{DOCS}/TROUBLESHOOTING.md">Troubleshooting</a>
      <a href="{REPO}/blob/master/CHANGELOG.md">Changelog</a>
      <a href="{REPO}/blob/master/LICENSE">Apache License 2.0</a>
    </nav>
  </div>
</footer>

<style>
  /* ---------- Hero ---------- */

  .hero {
    padding-top: clamp(32px, 6vw, 72px);
  }

  .intro {
    display: grid;
    justify-items: center;
    gap: 22px;
    text-align: center;
  }
  .intro > :global(*) {
    animation: rise 1000ms var(--ease-out) both;
  }
  .intro > :global(:nth-child(2)) {
    animation-delay: 80ms;
  }
  .intro > :global(:nth-child(3)) {
    animation-delay: 160ms;
  }
  .intro > :global(:nth-child(4)) {
    animation-delay: 240ms;
  }
  .intro > :global(:nth-child(5)) {
    animation-delay: 300ms;
  }
  .intro > :global(:nth-child(6)) {
    animation-delay: 360ms;
  }
  @keyframes rise {
    from {
      opacity: 0;
      translate: 0 16px;
    }
  }

  .release {
    display: inline-flex;
    align-items: center;
    padding: 6px 16px;
    border-radius: 999px;
    font-size: 14px;
    font-weight: 600;
    color: var(--fg-dim);
    background: var(--raised);
    box-shadow: var(--card-shadow);
    text-decoration: none !important;
    transition: color 150ms var(--ease);
  }
  .release:hover {
    color: var(--fg);
  }
  .name {
    margin-bottom: -10px;
    font-family: var(--font-display);
    font-size: clamp(20px, 2.4vw, 26px);
    font-weight: 650;
    letter-spacing: -0.02em;
    color: var(--accent);
  }
  h1 {
    max-width: 14ch;
    font-size: clamp(42px, 7.2vw, 84px);
    font-weight: 700;
    letter-spacing: -0.045em;
    line-height: 0.98;
  }

  .hero .lede {
    max-width: 34em;
  }

  .platforms {
    margin-top: -8px;
    font-size: 14px;
    color: var(--fg-faint);
  }

  .stage {
    max-width: 1240px;
    margin-top: clamp(36px, 5vw, 56px);
    animation: rise-big 1400ms var(--ease-out) 350ms both;
  }
  @keyframes rise-big {
    from {
      opacity: 0;
      translate: 0 48px;
      scale: 0.97;
    }
  }

  /* ---------- Uses ---------- */

  .uses ul {
    margin: 24px 0 0;
    padding: 0;
    list-style: none;
    border-top: 1px solid var(--line);
  }
  .uses li {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 8px 48px;
    align-items: baseline;
    padding: clamp(22px, 3vw, 34px) 0;
    border-bottom: 1px solid var(--line);
  }
  .uses h3 {
    font-size: clamp(26px, 3.4vw, 40px);
    font-weight: 600;
    letter-spacing: -0.03em;
  }
  .uses p {
    font-size: 17px;
    color: var(--fg-dim);
  }

  /* ---------- Details ---------- */

  .details {
    padding-top: clamp(96px, 14vw, 180px);
  }
  .details header {
    display: grid;
    gap: 20px;
    max-width: 700px;
  }
  .tiles {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 16px;
    margin: clamp(36px, 5vw, 56px) 0 0;
    padding: 0;
    list-style: none;
  }
  .tiles li {
    display: grid;
    grid-template-rows: auto auto 1fr;
    gap: 6px;
    padding: 20px 22px 24px;
    border-radius: 16px;
    background: var(--raised);
    box-shadow: var(--card-shadow);
  }
  .sample {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 2px;
    height: 76px;
    margin-bottom: 12px;
    padding: 12px 16px;
    border-radius: var(--radius-control);
    background: var(--page);
    box-shadow: inset 0 0 0 1px var(--line);
    overflow: hidden;
  }
  .status-title {
    font-size: 15px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }
  .status-detail {
    overflow: hidden;
    font-size: 13px;
    color: var(--fg-dim);
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  /* The titles take turns, like the app's status as a connection comes up. */
  .cycle {
    display: grid;
  }
  .cycle > span {
    display: grid;
    gap: 2px;
    min-width: 0;
    grid-area: 1 / 1;
    opacity: 0;
    animation: turn calc(var(--n) * 1.8s) var(--ease) calc(var(--i) * 1.8s) infinite;
  }
  @keyframes turn {
    0% {
      opacity: 0;
      translate: 0 6px;
    }
    6%,
    28% {
      opacity: 1;
      translate: 0 0;
    }
    34%,
    100% {
      opacity: 0;
      translate: 0 -6px;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .cycle > span {
      animation: none;
    }
    .cycle > span:last-child {
      opacity: 1;
    }
  }
  .row {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 14px;
    border-radius: var(--radius-control);
    font-size: 14px;
    background: var(--raised);
    box-shadow: var(--card-shadow);
  }
  .row > span:last-child {
    color: var(--fg-dim);
  }
  .sample code {
    font-family: var(--font-mono);
    font-size: 12.5px;
    color: var(--fg-dim);
    white-space: nowrap;
  }
  .sample code span {
    margin-right: 10px;
    color: var(--fg-faint);
  }
  .sample code.error {
    color: var(--error);
  }

  .tiles h3 {
    font-family: var(--font-ui);
    font-size: 16.5px;
    font-weight: 650;
    letter-spacing: 0;
  }
  .tiles p {
    font-size: 15px;
    color: var(--fg-dim);
  }

  /* ---------- Credit and footer ---------- */

  .credit {
    padding: 0 0 clamp(48px, 8vw, 96px);
  }
  .credit p {
    max-width: 640px;
    margin: 0 auto;
    font-size: 15px;
    color: var(--fg-faint);
    text-align: center;
  }

  footer {
    border-top: 1px solid var(--line);
  }
  .foot {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 16px 32px;
    padding-top: 28px;
    padding-bottom: 40px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    font-family: var(--font-display);
    font-weight: 650;
  }
  .foot nav {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 24px;
    font-size: 14.5px;
  }
  .foot nav a {
    color: var(--fg-dim);
  }
  .foot nav a:hover {
    color: var(--fg);
  }

  @media (max-width: 960px) {
    .tiles {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
  @media (max-width: 760px) {
    .uses li,
    .tiles {
      grid-template-columns: 1fr;
    }
  }
</style>
