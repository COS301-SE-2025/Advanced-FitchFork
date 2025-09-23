// src/components/common/PercentageTag.tsx
import React from 'react';
import { Tag } from 'antd';
import { useTheme } from '@/context/ThemeContext';
import { scaleColor, type ScaleScheme } from '@/utils/color';

export interface PercentageTagProps {
  value: number;
  min?: number;
  max?: number;
  /** Color scheme name from utils/color (e.g. 'red-green', 'green-red', 'gray-red', 'blue-red') */
  scheme?: ScaleScheme;
  /** Optional custom HEX gradient stops; overrides scheme if provided */
  colors?: string[];
  showValue?: boolean;
  decimals?: number;
  suffix?: string;
  children?: React.ReactNode;
  className?: string;
  style?: React.CSSProperties;
  'data-testid'?: string;
  asText?: boolean;
}

/* helpers (HEX-based) */
const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));
const norm01 = (v: number, lo: number, hi: number) =>
  hi === lo ? 0 : clamp((v - lo) / (hi - lo), 0, 1);

function hexToRgb(hex: string) {
  let h = hex.replace('#', '').trim();
  if (h.length === 3)
    h = h
      .split('')
      .map((c) => c + c)
      .join('');
  const n = parseInt(h, 16);
  return { r: (n >> 16) & 255, g: (n >> 8) & 255, b: n & 255 };
}
function rgbToHex(r: number, g: number, b: number) {
  const toHex = (n: number) => Math.round(n).toString(16).padStart(2, '0');
  return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
}
const mixHex = (aHex: string, bHex: string, t: number) => {
  const a = hexToRgb(aHex);
  const b = hexToRgb(bHex);
  return rgbToHex(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t);
};
const darken = (hex: string, amount: number) => {
  const { r, g, b } = hexToRgb(hex);
  return rgbToHex(r * (1 - amount), g * (1 - amount), b * (1 - amount));
};
const lighten = (hex: string, amount: number) => {
  const { r, g, b } = hexToRgb(hex);
  return rgbToHex(r + (255 - r) * amount, g + (255 - g) * amount, b + (255 - b) * amount);
};
const pickFromStops = (stops: string[], t01: number) => {
  if (stops.length === 0) return '#999999';
  if (stops.length === 1) return stops[0];
  const pos = t01 * (stops.length - 1);
  const i = Math.floor(pos);
  const f = pos - i;
  return mixHex(stops[i], stops[Math.min(i + 1, stops.length - 1)], f);
};

const PercentageTag: React.FC<PercentageTagProps> = ({
  value,
  min = 0,
  max = 100,
  scheme = 'red-green',
  colors,
  showValue = true,
  decimals = 0,
  suffix = '%',
  children,
  className,
  style,
  'data-testid': dataTestId,
  asText = false,
}) => {
  const { isDarkMode } = useTheme();

  const t01 = norm01(value, min, max);
  const pct = Math.round(t01 * 100);

  // main hue (HEX)
  const main = colors && colors.length >= 2 ? pickFromStops(colors, t01) : scaleColor(pct, scheme);

  // theme-aware background & readable text (HEX)
  const bg = isDarkMode ? mixHex(main, '#000000', 0.82) : mixHex(main, '#ffffff', 0.86);
  const text = isDarkMode ? lighten(main, 0.35) : darken(main, 0.25);

  const content =
    children ?? (showValue ? `${value.toFixed(Math.max(0, decimals))}${suffix}` : null);
  const title = `${value.toFixed(Math.max(0, decimals))}${suffix}`;

  if (asText) {
    return (
      <span
        data-testid={dataTestId ?? 'percentage-text'}
        className={['font-medium', className].filter(Boolean).join(' ')}
        style={{ color: text, ...style }}
        title={title}
      >
        {content}
      </span>
    );
  }

  // AntD Tag expects HEX in `color` for custom colored tag.
  // It sets white text by default, so wrap content in a span to override text color reliably.
  return (
    <Tag
      data-testid={dataTestId ?? 'percentage-tag'}
      color={bg} // <- custom HEX background (and border)
      className={['font-medium', className].filter(Boolean).join(' ')}
      style={{ borderColor: main, ...style }} // subtle border with main hue
      title={title}
    >
      <span style={{ color: text }}>{content}</span>
    </Tag>
  );
};

export default PercentageTag;
