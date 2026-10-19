import { REPO, type Download } from "$lib/site";

const API = "https://api.github.com/repos/sawongam/gnirehtet-gui/releases/latest";
const CACHE = "site-release";
const CACHE_MS = 60 * 60 * 1000;

type Asset = { name: string; url: string };
type Latest = { version: string; url: string; assets: Asset[] };

/**
 * The latest release, asked of GitHub when the page opens, so a new release
 * shows up here without redeploying the site. Until the answer arrives (or if
 * GitHub can't be reached) it is the version the site was built with.
 */
class Release {
  version = $state(__APP_VERSION__);
  url = $state(`${REPO}/releases/tag/v${__APP_VERSION__}`);
  #assets = $state<Asset[] | null>(null);

  fileName(d: Download) {
    return this.#asset(d)?.name ?? `gnirehtet-gui-${this.version}-${d.suffix}`;
  }

  fileUrl(d: Download) {
    const asset = this.#asset(d);
    if (asset) return asset.url;
    // A release without this file: its page lists what it does have.
    if (this.#assets) return this.url;
    return `${REPO}/releases/download/v${this.version}/${this.fileName(d)}`;
  }

  #asset(d: Download) {
    return this.#assets?.find((a) => d.pattern.test(a.name));
  }

  async load() {
    const latest = this.#cached() ?? (await this.#fetch());
    if (!latest) return;
    this.version = latest.version;
    this.url = latest.url;
    this.#assets = latest.assets;
  }

  async #fetch(): Promise<Latest | null> {
    try {
      const res = await fetch(API, { headers: { Accept: "application/vnd.github+json" } });
      if (!res.ok) return null;
      const json = await res.json();
      const latest: Latest = {
        version: String(json.tag_name).replace(/^v/, ""),
        url: json.html_url,
        assets: (json.assets ?? []).map((a: { name: string; browser_download_url: string }) => ({
          name: a.name,
          url: a.browser_download_url,
        })),
      };
      try {
        sessionStorage.setItem(CACHE, JSON.stringify({ at: Date.now(), latest }));
      } catch {
        // Storage blocked: ask again on the next visit.
      }
      return latest;
    } catch {
      // Offline or rate limited: keep the build's version.
      return null;
    }
  }

  // GitHub allows 60 unauthenticated requests an hour, so don't ask on every page view.
  #cached(): Latest | null {
    try {
      const saved = JSON.parse(sessionStorage.getItem(CACHE) ?? "null");
      return saved && Date.now() - saved.at < CACHE_MS ? saved.latest : null;
    } catch {
      return null;
    }
  }
}

export const release = new Release();
