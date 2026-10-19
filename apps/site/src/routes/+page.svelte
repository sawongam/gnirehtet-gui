<script lang="ts">
  import { base } from "$app/paths";
  import Nav from "$lib/components/Nav.svelte";
  import DownloadButton from "$lib/components/DownloadButton.svelte";
  import Demo from "$lib/components/Demo.svelte";
  import HowItWorks from "$lib/components/HowItWorks.svelte";
  import Downloads from "$lib/components/Downloads.svelte";
  import { reveal } from "$lib/reveal";
  import { DOCS, REPO, UPSTREAM, VERSION } from "$lib/site";

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

  const details = [
    {
      title: "Explains what the phone needs",
      text: "Unauthorized, offline, or blocked by Linux udev permissions: the app says what's wrong and what to do.",
    },
    {
      title: "Shows each state as it happens",
      text: "Starting, waiting for the VPN prompt, connected, interrupted.",
    },
    {
      title: "Recovers from a pulled cable",
      text: "Plug it back in and the USB tunnel is restored without a click.",
    },
    {
      title: "Leaves nothing running",
      text: "Disconnecting or quitting stops the VPN on the phone and the relay on the computer.",
    },
    {
      title: "Preferences when you need them",
      text: "Custom DNS servers, routes and relay port.",
    },
    {
      title: "A log for troubleshooting",
      text: "Every relay and adb step, with errors flagged.",
    },
    {
      title: "Light and dark",
      text: "Follows your system, or pick one.",
    },
  ];

  const TITLE = "Gnirehtet: your computer's internet on your Android phone, over USB";
  const DESCRIPTION =
    "A desktop app for reverse tethering. Plug in an Android phone, click Connect, and it uses your computer's internet. No root, for Windows, macOS and Linux.";
</script>

<svelte:head>
  <title>{TITLE}</title>
  <meta name="description" content={DESCRIPTION} />
  <meta property="og:title" content="Gnirehtet" />
  <meta property="og:description" content={DESCRIPTION} />
  <meta property="og:type" content="website" />
  <meta property="og:image" content="https://sawongam.github.io{base}/og.png" />
  <meta name="twitter:card" content="summary_large_image" />
</svelte:head>

<Nav />

<main>
  <section class="hero">
    <div class="wrap intro">
      <a class="release" href="{REPO}/releases/tag/v{VERSION}">
        <span class="pill">New</span> Version {VERSION} is out
      </a>
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
      <div class="lists">
        <ul class="boxed-list" use:reveal={60}>
          {#each details.slice(0, 4) as d (d.title)}
            <li><span class="row-title">{d.title}</span><span class="row-sub">{d.text}</span></li>
          {/each}
        </ul>
        <ul class="boxed-list" use:reveal={140}>
          {#each details.slice(4) as d (d.title)}
            <li><span class="row-title">{d.title}</span><span class="row-sub">{d.text}</span></li>
          {/each}
        </ul>
      </div>
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
  @keyframes rise {
    from {
      opacity: 0;
      translate: 0 16px;
    }
  }

  .release {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    padding: 4px 14px 4px 4px;
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
  .pill {
    padding: 2px 10px;
    border-radius: 999px;
    font-size: 12.5px;
    font-weight: 700;
    color: #fff;
    background: var(--accent-bg);
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
  .lists {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 20px;
    align-items: start;
    margin-top: clamp(36px, 5vw, 56px);
  }
  .lists li {
    display: grid;
    gap: 2px;
    padding: 16px 20px;
  }
  .row-title {
    font-weight: 650;
  }
  .row-sub {
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

  @media (max-width: 760px) {
    .uses li,
    .lists {
      grid-template-columns: 1fr;
    }
  }
</style>
