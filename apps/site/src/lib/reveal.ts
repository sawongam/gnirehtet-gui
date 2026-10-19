import type { Action } from "svelte/action";

const observed = new Map<Element, () => void>();
let observer: IntersectionObserver | null = null;

/**
 * Marks an element as entered once it scrolls into view, which sets
 * `data-in` for CSS to animate from. `delay` staggers siblings (ms).
 */
export const reveal: Action<HTMLElement, number | undefined> = (node, delay = 0) => {
  node.style.setProperty("--reveal-delay", `${delay}ms`);
  node.dataset.reveal = "";
  observer ??= new IntersectionObserver(
    (entries) => {
      for (const entry of entries) {
        if (!entry.isIntersecting) continue;
        observed.get(entry.target)?.();
      }
    },
    { rootMargin: "0px 0px -12% 0px" },
  );
  const done = () => {
    node.dataset.in = "";
    observer?.unobserve(node);
    observed.delete(node);
  };
  observed.set(node, done);
  observer.observe(node);
  return {
    destroy() {
      observer?.unobserve(node);
      observed.delete(node);
    },
  };
};

/** Calls `onchange` as the element enters and leaves the viewport. */
export const inview: Action<HTMLElement, (visible: boolean) => void> = (node, onchange) => {
  const io = new IntersectionObserver(([entry]) => onchange(entry.isIntersecting), { threshold: 0.15 });
  io.observe(node);
  return { destroy: () => io.disconnect() };
};
