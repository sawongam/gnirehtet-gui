/**
 * Validation of connection options. The backend enforces the same rules;
 * checking here gives immediate feedback in Settings.
 */

export const DEFAULT_PORT = 31416;

const IPV4 = /^(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)(\.(25[0-5]|2[0-4]\d|1\d\d|[1-9]?\d)){3}$/;

function items(value: string): string[] {
  return value
    .split(",")
    .map((s) => s.trim())
    .filter((s) => s.length > 0);
}

/** Returns an error message, or null when valid. */
export function validatePort(value: string): string | null {
  if (!/^\d+$/.test(value.trim())) return "Enter a number between 1024 and 65535.";
  const port = Number(value);
  if (port < 1024 || port > 65535) return "Use a port between 1024 and 65535.";
  return null;
}

/** Empty means "use the client default" and is valid. */
export function validateDnsServers(value: string): string | null {
  for (const server of items(value)) {
    if (!IPV4.test(server)) return `“${server}” is not an IPv4 address.`;
  }
  return null;
}

/** Empty means "route all traffic" and is valid. */
export function validateRoutes(value: string): string | null {
  for (const route of items(value)) {
    const [address, prefix, extra] = route.split("/");
    if (extra !== undefined || prefix === undefined || !IPV4.test(address)) {
      return `“${route}” is not a route like 10.0.0.0/8.`;
    }
    if (!/^\d{1,2}$/.test(prefix) || Number(prefix) > 32) {
      return `“${route}” has an invalid prefix length (0–32).`;
    }
  }
  return null;
}

/** Canonical comma-separated form, or null for "use the default". */
export function normalizeList(value: string): string | null {
  const list = items(value);
  return list.length > 0 ? list.join(",") : null;
}
