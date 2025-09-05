import React from 'react';
import { Tag } from 'antd';
import { useTheme } from '@/context/ThemeContext';

type PaletteName = 'traffic' | 'greenRed' | 'blueCyan' | 'purplePink' | 'gray';

export interface PercentageTagProps {
  value: number;
  min?: number;
  max?: number;
  colors?: string[];
  palette?: PaletteName;
  showValue?: boolean;
  decimals?: number;
  suffix?: string;
  children?: React.ReactNode;
  className?: string;
  style?: React.CSSProperties;
  'data-testid'?: string;
}

const PRESET_PALETTES: Record<PaletteName, string[]> = {
  traffic: ['#ef4444', '#f59e0b', '#22c55e'],
  greenRed: ['#22c55e', '#eab308', '#ef4444'],
  blueCyan: ['#60a5fa', '#22d3ee'],
  purplePink: ['#8b5cf6', '#ec4899'],
  gray: ['#e5e7eb', '#9ca3af', '#4b5563'],
};

// ---------- tiny color helpers ----------
const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));
const norm = (v: number, lo: number, hi: number) =>
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
const rgbToHex = (r: number, g: number, b: number) =>
  `#${[r, g, b].map((n) => Math.round(n).toString(16).padStart(2, '0')).join('')}`;

const mix = (aHex: string, bHex: string, t: number) => {
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

const pickColor = (stops: string[], t01: number) => {
  if (stops.length === 0) return '#999999';
  if (stops.length === 1) return stops[0];
  const pos = t01 * (stops.length - 1);
  const i = Math.floor(pos);
  const f = pos - i;
  return mix(stops[i], stops[Math.min(i + 1, stops.length - 1)], f);
};

// ---------- component ----------
const PercentageTag: React.FC<PercentageTagProps> = ({
  value,
  min = 0,
  max = 100,
  colors,
  palette = 'traffic',
  showValue = true,
  decimals = 0,
  suffix = '%',
  children,
  className,
  style,
  'data-testid': dataTestId,
}) => {
  const { isDarkMode } = useTheme();

  const stops = colors && colors.length >= 2 ? colors : PRESET_PALETTES[palette];
  const t = norm(value, min, max);
  const main = pickColor(stops, t); // brand color for border

  // Light mode: very light tint background toward white; text slightly darker than main
  // Dark mode: dark tint background toward black; text slightly lighter than main
  const bg = isDarkMode ? mix(main, '#000000', 0.82) : mix(main, '#ffffff', 0.86);
  const text = isDarkMode ? lighten(main, 0.35) : darken(main, 0.25);

  return (
    <Tag
      data-testid={dataTestId ?? 'percentage-tag'}
      bordered
      className={['font-medium', className].filter(Boolean).join(' ')}
      style={{
        color: text,
        backgroundColor: bg,
        borderColor: main,
        // subtle polish to feel closer to AntD tokens
        boxShadow: 'inset 0 0 0 1px var(--tag-border, currentColor, 0)',
        ...style,
      }}
      title={`${value.toFixed(Math.max(0, decimals))}${suffix}`}
    >
      {children ?? (showValue ? `${value.toFixed(Math.max(0, decimals))}${suffix}` : null)}
    </Tag>
  );
};

export default PercentageTag;
