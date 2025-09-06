/** 32-bit FNV-1a hash → stable across sessions without crypto deps */
export function hash32(str: string): number {
  let h = 0x811c9dc5 >>> 0;
  for (let i = 0; i < str.length; i++) {
    h ^= str.charCodeAt(i);
    h = Math.imul(h, 0x01000193);
  }
  // Avoid exact 0 to keep % math nice
  return (h >>> 0) || 1;
}

/** Convert string → HSL parts with tunable saturation/lightness */
export function hslFromString(
  seed: string,
  opts?: { sat?: number; light?: number }
): { h: number; s: number; l: number } {
  const h = hash32(seed) % 360;
  const s = Math.max(0, Math.min(100, opts?.sat ?? 70));
  const l = Math.max(0, Math.min(100, opts?.light ?? 50));
  return { h, s, l };
}

export function hslToCss({ h, s, l }: { h: number; s: number; l: number }): string {
  return `hsl(${h} ${s}% ${l}%)`;
}

/** Avatar colors (bg/text) tuned for light/dark themes */
export function getAvatarColors(username: string, isDarkMode: boolean): {
  background: string;
  text: string;
} {
  const base = hslFromString(username, {
    sat: isDarkMode ? 70 : 70,
    light: isDarkMode ? 30 : 85,
  });
  const bg = hslToCss(base);

  // Text color: same hue/sat, opposite lightness band for contrast
  const text = hslToCss({
    h: base.h,
    s: base.s,
    l: isDarkMode ? 85 : 30,
  });
  return { background: bg, text };
}

/** Node color for graphs: single fill color, theme-aware */
export function getNodeColor(id: string, isDarkMode: boolean): string {
  const parts = hslFromString(id, {
    sat: isDarkMode ? 70 : 65,
    light: isDarkMode ? 55 : 50,
  });
  return hslToCss(parts);
}

function lerp(a: number, b: number, t: number) {
  return a + (b - a) * t;
}

function interpolateColor(start: string, end: string, t: number): string {
  const s = parseInt(start.slice(1), 16);
  const e = parseInt(end.slice(1), 16);

  const sr = (s >> 16) & 0xff, sg = (s >> 8) & 0xff, sb = s & 0xff;
  const er = (e >> 16) & 0xff, eg = (e >> 8) & 0xff, eb = e & 0xff;

  const r = Math.round(lerp(sr, er, t));
  const g = Math.round(lerp(sg, eg, t));
  const b = Math.round(lerp(sb, eb, t));

  return `rgb(${r}, ${g}, ${b})`;
}

export type ScaleScheme = "gray-red" | "red-green" | "green-red" | "blue-red";

/** 0–100 → color with orange midpoint */
export function scaleColor(value: number, scheme: ScaleScheme = "red-green"): string {
  const t = Math.max(0, Math.min(1, value / 100));

  // fixed orange midpoint
  const mid = "#f59e0b";

  let start: string, end: string;
  switch (scheme) {
    case "gray-red":
      start = "#9ca3af"; // gray-400
      end = "#ef4444";   // red-500
      break;
    case "green-red":
      start = "#22c55e"; // green-500
      end = "#ef4444";   // red-500
      break;
    case "blue-red":
      start = "#3b82f6"; // blue-500
      end = "#ef4444";   // red-500
      break;
    case "red-green":
    default:
      start = "#ef4444"; // red-500
      end = "#22c55e";   // green-500
      break;
  }

  if (t < 0.5) {
    // first half: start → orange
    return interpolateColor(start, mid, t / 0.5);
  } else {
    // second half: orange → end
    return interpolateColor(mid, end, (t - 0.5) / 0.5);
  }
}
