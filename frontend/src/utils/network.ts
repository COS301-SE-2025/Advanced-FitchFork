// src/utils/network.ts
// Minimal, browser-safe public IP fetch with fallbacks + CIDR formatting.

const FETCH_TIMEOUT_MS = 4500;

function withTimeout<T>(p: Promise<T>, ms = FETCH_TIMEOUT_MS): Promise<T> {
  return new Promise((resolve, reject) => {
    const t = setTimeout(() => reject(new Error('ip-timeout')), ms);
    p.then((v) => { clearTimeout(t); resolve(v); }, (e) => { clearTimeout(t); reject(e); });
  });
}

async function fetchJson(url: string): Promise<any> {
  const res = await withTimeout(fetch(url, { cache: 'no-store' }));
  if (!res.ok) throw new Error(`ip-fetch-failed:${res.status}`);
  return res.json();
}

async function fetchText(url: string): Promise<string> {
  const res = await withTimeout(fetch(url, { cache: 'no-store' }));
  if (!res.ok) throw new Error(`ip-fetch-failed:${res.status}`);
  return (await res.text()).trim();
}

export function isIPv4(ip: string): boolean {
  // quick & strict-ish
  return /^(25[0-5]|2[0-4]\d|1?\d?\d)(\.(25[0-5]|2[0-4]\d|1?\d?\d)){3}$/.test(ip);
}

export function isIPv6(ip: string): boolean {
  // very permissive IPv6 (compressed supported)
  return /^(([0-9A-Fa-f]{1,4}:){7}[0-9A-Fa-f]{1,4}|([0-9A-Fa-f]{1,4}:){1,7}:|([0-9A-Fa-f]{1,4}:){1,6}:[0-9A-Fa-f]{1,4}|([0-9A-Fa-f]{1,4}:){1,5}(:[0-9A-Fa-f]{1,4}){1,2}|([0-9A-Fa-f]{1,4}:){1,4}(:[0-9A-Fa-f]{1,4}){1,3}|([0-9A-Fa-f]{1,4}:){1,3}(:[0-9A-Fa-f]{1,4}){1,4}|([0-9A-Fa-f]{1,4}:){1,2}(:[0-9A-Fa-f]{1,4}){1,5}|[0-9A-Fa-f]{1,4}:((:[0-9A-Fa-f]{1,4}){1,6})|:((:[0-9A-Fa-f]{1,4}){1,7}|:)|fe80:(:[0-9A-Fa-f]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|2[0-4]\d|1?\d?\d)(\.(25[0-5]|2[0-4]\d|1?\d?\d)){3})|([0-9A-Fa-f]{1,4}:){1,4}:((25[0-5]|2[0-4]\d|1?\d?\d)(\.(25[0-5]|2[0-4]\d|1?\d?\d)){3}))$/.test(ip);
}

export function asSingleHostCIDR(ip: string): string {
  if (isIPv4(ip)) return `${ip}/32`;
  if (isIPv6(ip)) return `${ip}/128`;
  throw new Error('invalid-ip');
}

/**
 * Get public IP (client-side) via multiple providers, then return single-host CIDR.
 * Providers:
 *  - ipify (JSON)
 *  - icanhazip (text)
 *  - ifconfig.co (JSON)
 */
export async function getCurrentIpAsCidr(): Promise<string> {
  const errors: unknown[] = [];

  // 1) ipify
  try {
    const j = await fetchJson('https://api.ipify.org?format=json');
    const ip = String(j?.ip ?? '');
    return asSingleHostCIDR(ip);
  } catch (e) { errors.push(e); }

  // 2) icanhazip
  try {
    const t = await fetchText('https://icanhazip.com/');
    return asSingleHostCIDR(t);
  } catch (e) { errors.push(e); }

  // 3) ifconfig.co
  try {
    const j = await fetchJson('https://ifconfig.co/json');
    const ip = String(j?.ip ?? '');
    return asSingleHostCIDR(ip);
  } catch (e) { errors.push(e); }

  // Give a concise error; you can log 'errors' if needed
  throw new Error('could-not-detect-public-ip');
}
