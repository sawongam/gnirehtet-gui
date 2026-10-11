import {
  DEFAULT_PORT,
  normalizeList,
  validateDnsServers,
  validatePort,
  validateRoutes,
} from "./validation";
import type { ConnectOptions } from "./types";

export type Theme = "system" | "light" | "dark";

type Stored = {
  port: number;
  dnsServers: string;
  routes: string;
  theme: Theme;
};

const STORAGE_KEY = "gnirehtet-gui.settings";

const DEFAULTS: Stored = {
  port: DEFAULT_PORT,
  dnsServers: "",
  routes: "",
  theme: "system",
};

function load(): Stored {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...DEFAULTS };
    const parsed = JSON.parse(raw) as Partial<Stored>;
    const stored = { ...DEFAULTS, ...parsed };
    // Never let a corrupted entry break connecting.
    if (validatePort(String(stored.port))) stored.port = DEFAULTS.port;
    if (typeof stored.dnsServers !== "string" || validateDnsServers(stored.dnsServers)) {
      stored.dnsServers = "";
    }
    if (typeof stored.routes !== "string" || validateRoutes(stored.routes)) stored.routes = "";
    if (!["system", "light", "dark"].includes(stored.theme)) stored.theme = "system";
    return stored;
  } catch {
    return { ...DEFAULTS };
  }
}

/** Persisted user preferences. Only valid values are ever stored. */
class Settings {
  #values = $state<Stored>(load());

  get port() {
    return this.#values.port;
  }
  get dnsServers() {
    return this.#values.dnsServers;
  }
  get routes() {
    return this.#values.routes;
  }
  get theme() {
    return this.#values.theme;
  }

  update(values: Partial<Stored>) {
    this.#values = { ...this.#values, ...values };
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.#values));
    } catch {
      // Storage unavailable: settings still apply for this run.
    }
    applyTheme(this.#values.theme);
  }

  reset() {
    this.update({ ...DEFAULTS });
  }

  connectOptions(): ConnectOptions {
    return {
      port: this.#values.port,
      dnsServers: normalizeList(this.#values.dnsServers),
      routes: normalizeList(this.#values.routes),
    };
  }
}

export const settings = new Settings();

const darkQuery = typeof window !== "undefined" ? window.matchMedia("(prefers-color-scheme: dark)") : null;

export function applyTheme(theme: Theme) {
  const dark = theme === "dark" || (theme === "system" && !!darkQuery?.matches);
  document.documentElement.classList.toggle("dark", dark);
}

darkQuery?.addEventListener("change", () => applyTheme(settings.theme));
