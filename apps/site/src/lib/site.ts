export const VERSION = __APP_VERSION__;
export const REPO = "https://github.com/sawongam/gnirehtet-gui";
export const RELEASES = `${REPO}/releases/latest`;
export const DOCS = `${REPO}/blob/master/docs`;
export const UPSTREAM = "https://github.com/Genymobile/gnirehtet";

export type Os = "windows" | "mac" | "linux";

export type Download = {
  os: Os;
  system: string;
  detail: string;
  /** File name suffix after `gnirehtet-gui-<version>-`, as release.yml names them. */
  suffix: string;
};

/** Same order and names as the Downloads table in the release notes. */
export const DOWNLOADS: Download[] = [
  { os: "windows", system: "Windows", detail: "Windows 10 and 11", suffix: "windows-x64-setup.exe" },
  { os: "mac", system: "macOS", detail: "Apple Silicon, M1 and later", suffix: "macos-apple-silicon.dmg" },
  { os: "mac", system: "macOS", detail: "Intel", suffix: "macos-intel.dmg" },
  { os: "linux", system: "Debian, Ubuntu, Mint", detail: ".deb package", suffix: "linux-x64.deb" },
  { os: "linux", system: "Fedora, openSUSE", detail: ".rpm package", suffix: "linux-x64.rpm" },
  { os: "linux", system: "Other Linux", detail: "AppImage", suffix: "linux-x64.AppImage" },
];

export function fileName(d: Download) {
  return `gnirehtet-gui-${VERSION}-${d.suffix}`;
}

export function fileUrl(d: Download) {
  return `${REPO}/releases/download/v${VERSION}/${fileName(d)}`;
}

export const OS_NAMES: Record<Os, string> = { windows: "Windows", mac: "macOS", linux: "Linux" };

/** Best guess at the visitor's desktop system; null on phones and the unknown. */
export function detectOs(): Os | null {
  const ua = navigator.userAgent;
  if (/Android|iPhone|iPad|iPod/i.test(ua)) return null;
  if (/Windows/i.test(ua)) return "windows";
  if (/Mac OS X|Macintosh/i.test(ua)) return "mac";
  if (/Linux|X11|CrOS/i.test(ua)) return "linux";
  return null;
}
