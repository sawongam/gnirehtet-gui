<script lang="ts">
  import { onMount } from "svelte";

  /**
   * "tethering", whose letters swap places until it reads "gnirehtet". Each
   * letter moves to its mirrored slot; letters heading right hop over the
   * ones heading left.
   */
  const WORD = "tethering";
  const letters = [...WORD];

  let root: HTMLSpanElement;
  let ghost: HTMLSpanElement;
  let shifts = $state<number[]>(letters.map(() => 0));
  let reversed = $state(false);
  /** Set on the first flip, so the way back doesn't play on load. */
  let moved = $state(false);

  function measure() {
    const forward = [...root.querySelectorAll<HTMLElement>(".letter")].map((el) => el.offsetLeft);
    const backward = [...ghost.children].map((el) => (el as HTMLElement).offsetLeft);
    shifts = forward.map((x, i) => backward[letters.length - 1 - i] - x);
  }

  function flip() {
    measure();
    reversed = !reversed;
    moved = true;
  }

  onMount(() => {
    void document.fonts?.ready.then(measure);
    measure();
    let timer: ReturnType<typeof setTimeout> | undefined;
    const io = new IntersectionObserver(
      ([entry]) => {
        clearTimeout(timer);
        if (entry.isIntersecting && !reversed) timer = setTimeout(flip, 700);
      },
      { threshold: 0.6 },
    );
    io.observe(root);
    const onResize = () => measure();
    addEventListener("resize", onResize);
    return () => {
      io.disconnect();
      clearTimeout(timer);
      removeEventListener("resize", onResize);
    };
  });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<span
  class="word"
  class:reversed
  class:moved
  bind:this={root}
  onpointerenter={flip}
  role="img"
  aria-label="tethering, reversed: gnirehtet"
>
  {#each letters as letter, i (i)}
    <span
      class="letter"
      class:up={shifts[i] > 0}
      style:--dx="{reversed ? shifts[i] : 0}px"
      style:--delay="{Math.abs(letters.length / 2 - i) * 28}ms">{letter}</span
    >
  {/each}
  <span class="ghost" bind:this={ghost} aria-hidden="true">
    {#each [...letters].reverse() as letter, i (i)}<span>{letter}</span>{/each}
  </span>
</span>

<style>
  .word {
    position: relative;
    display: inline-block;
    white-space: nowrap;
    cursor: default;
  }

  .letter {
    position: relative;
    display: inline-block;
    translate: var(--dx) 0;
    transition: translate 1100ms cubic-bezier(0.65, 0, 0.25, 1) var(--delay);
  }
  /* The hop: letters moving right arc up, letters moving left dip down. */
  .reversed .letter {
    animation: hop-down 1100ms cubic-bezier(0.65, 0, 0.25, 1) var(--delay);
  }
  .reversed .letter.up {
    animation-name: hop-up;
  }
  .moved:not(.reversed) .letter {
    animation: hop-up-back 1100ms cubic-bezier(0.65, 0, 0.25, 1) var(--delay);
  }
  .moved:not(.reversed) .letter.up {
    animation-name: hop-down-back;
  }
  @keyframes hop-up {
    50% {
      top: -0.32em;
    }
  }
  @keyframes hop-down {
    50% {
      top: 0.18em;
    }
  }
  @keyframes hop-up-back {
    50% {
      top: -0.32em;
    }
  }
  @keyframes hop-down-back {
    50% {
      top: 0.18em;
    }
  }

  .ghost {
    position: absolute;
    left: 0;
    top: 0;
    visibility: hidden;
    pointer-events: none;
  }
</style>
