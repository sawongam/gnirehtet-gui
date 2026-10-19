/** Light or dark, from the visitor's choice or else their system. */
class Theme {
  // Already applied to <html> by the inline script in app.html.
  dark = $state(typeof document !== "undefined" && document.documentElement.classList.contains("dark"));

  /** Follows system changes until the visitor picks a theme themselves. */
  watchSystem() {
    matchMedia("(prefers-color-scheme: dark)").addEventListener("change", (e) => {
      if (!this.#saved()) this.#apply(e.matches);
    });
  }

  toggle() {
    const dark = !this.dark;
    try {
      localStorage.setItem("site-theme", dark ? "dark" : "light");
    } catch {
      // Storage blocked: the choice lasts until the page is closed.
    }
    this.#apply(dark);
  }

  #saved() {
    try {
      return localStorage.getItem("site-theme");
    } catch {
      return null;
    }
  }

  #apply(dark: boolean) {
    const root = document.documentElement;
    root.classList.add("theme-anim");
    root.classList.toggle("dark", dark);
    this.dark = dark;
    setTimeout(() => root.classList.remove("theme-anim"), 400);
  }
}

export const theme = new Theme();
