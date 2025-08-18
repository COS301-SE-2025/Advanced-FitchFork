// components/common/PercentageTag.tsx
import React from 'react';
import { Tag } from 'antd';

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

function clamp(v: number, lo: number, hi: number) {
  return Math.max(lo, Math.min(hi, v));
}
function normalize(v: number, lo: number, hi: number) {
  if (hi === lo) return 0;
  return clamp((v - lo) / (hi - lo), 0, 1);
}
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
  const to = (n: number) => n.toString(16).padStart(2, '0');
  return `#${to(r)}${to(g)}${to(b)}`;
}
function lerp(a: number, b: number, t: number) {
  return a + (b - a) * t;
}
function interpColor(aHex: string, bHex: string, t: number) {
  const a = hexToRgb(aHex);
  const b = hexToRgb(bHex);
  return rgbToHex(
    Math.round(lerp(a.r, b.r, t)),
    Math.round(lerp(a.g, b.g, t)),
    Math.round(lerp(a.b, b.b, t)),
  );
}
function pickColor(stops: string[], t01: number) {
  if (stops.length === 0) return '#999999';
  if (stops.length === 1) return stops[0];
  const pos = t01 * (stops.length - 1);
  const i = Math.floor(pos);
  const f = pos - i;
  const a = stops[i];
  const b = stops[Math.min(i + 1, stops.length - 1)];
  return interpColor(a, b, f);
}
function contrastText(hex: string) {
  const { r, g, b } = hexToRgb(hex);
  const [R, G, B] = [r, g, b].map((v) => {
    const x = v / 255;
    return x <= 0.03928 ? x / 12.92 : Math.pow((x + 0.055) / 1.055, 2.4);
  });
  const L = 0.2126 * R + 0.7152 * G + 0.0722 * B;
  return L > 0.5 ? '#000000' : '#ffffff';
}

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
  const stops = colors && colors.length >= 2 ? colors : PRESET_PALETTES[palette];
  const t = normalize(value, min, max);
  const bg = pickColor(stops, t);
  const fg = contrastText(bg);

  return (
    <Tag
      data-testid={dataTestId ?? 'percentage-tag'}
      color={bg}
      className={['font-semibold', className].filter(Boolean).join(' ')} // ← Tailwind bold
      style={{
        color: fg, // dynamic contrast color
        borderColor: bg, // match border to bg
        ...style,
      }}
      title={`${value.toFixed(Math.max(0, decimals))}${suffix}`}
    >
      {children ?? (showValue ? `${value.toFixed(Math.max(0, decimals))}${suffix}` : null)}
    </Tag>
  );
};

export default PercentageTag;
