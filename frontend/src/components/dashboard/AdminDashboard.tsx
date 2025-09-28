// AdminDashboard.tsx
import React, { useCallback, useEffect, useMemo, useState, useRef, useLayoutEffect } from 'react';
import {
  Card,
  Space,
  DatePicker,
  Select,
  InputNumber,
  Button,
  Statistic,
  Modal,
  Progress,
  Divider,
  Tooltip,
  Tag,
  message,
} from 'antd';
import { DownloadOutlined, InfoCircleOutlined, TeamOutlined } from '@ant-design/icons';
import dayjs, { Dayjs } from 'dayjs';
import { Column, type ColumnConfig } from '@ant-design/plots';
import { useTheme } from '@/context/ThemeContext';
import { useUI } from '@/context/UIContext';
import {
  getSystemMetrics,
  exportSystemMetrics,
  type MetricsPoint,
} from '@/services/system/metrics/get';
import {
  getSubmissionsOverTime,
  exportSubmissionsOverTime,
  type SubmissionsPoint,
} from '@/services/system/submissions/get';
import { scaleColor } from '@/utils/color';
import { useWsEvents, Topics, type PayloadOf } from '@/ws';
import {
  getMaxConcurrent,
  setMaxConcurrent as setMaxConcurrentApi,
} from '@/services/system/code_manager';
import { CapacityModal } from '../system';

const PercentBar: React.FC<{
  percent: number | null | undefined;
  height?: number;
  ariaLabel?: string;
}> = ({ percent, height = 5, ariaLabel }) => {
  const v = Math.max(0, Math.min(100, percent ?? 0));
  const color = scaleColor(v, 'green-red');
  return (
    <div
      className="relative w-full rounded-full bg-gray-200 dark:bg-gray-700 overflow-hidden"
      style={{ height }}
      role="progressbar"
      aria-valuemin={0}
      aria-valuemax={100}
      aria-valuenow={Math.round(v)}
      aria-label={ariaLabel}
    >
      <div
        className="absolute inset-y-0 left-0 rounded-full will-change-[width,background-color]"
        style={{
          width: `${v}%`,
          background: color,
          transition: 'width 240ms ease, background-color 160ms linear',
        }}
      />
    </div>
  );
};

const MeterRow: React.FC<{
  label: React.ReactNode;
  valueText?: string;
  percent: number | null | undefined;
}> = ({ label, valueText, percent }) => (
  <div className="flex flex-col gap-1">
    <div className="flex items-center justify-between text-[11px] text-gray-500 dark:text-gray-400">
      <span className="truncate">{label}</span>
      {valueText ? <span className="tabular-nums">{valueText}</span> : null}
    </div>
    <PercentBar percent={percent} />
  </div>
);

const MiniBars: React.FC<{
  values: number[];
  /** total pixel height of the strip (bars scale within this) */
  containerHeight?: number;
  ariaLabel?: string;
}> = ({ values, containerHeight = 96, ariaLabel }) => {
  const safe = values.map((v) => Math.max(0, Math.min(100, v ?? 0)));
  const barBase = Math.max(2, Math.floor((containerHeight - 20) * 0.04)); // min bar height baseline

  return (
    <div
      role="group"
      aria-label={ariaLabel}
      className="rounded-lg bg-gray-100 dark:bg-gray-800/60 border border-gray-200 dark:border-gray-700 p-2"
      style={{ height: containerHeight }}
    >
      <div
        className="h-full grid gap-[3px] items-end"
        style={{
          // One column per core → bars take the FULL width evenly
          gridTemplateColumns: `repeat(${Math.max(1, safe.length)}, 1fr)`,
        }}
      >
        {safe.map((v, i) => {
          const h = Math.max(barBase, Math.round((v / 100) * (containerHeight - 24)));
          return (
            <div
              key={i}
              className="w-full rounded-[2px]"
              style={{
                height: h,
                background: scaleColor(v, 'green-red'),
                transition: 'height 160ms ease',
              }}
              title={`Core ${i + 1}: ${v.toFixed(1)}%`}
            />
          );
        })}
      </div>
    </div>
  );
};

type UiBucket = 'day' | 'week' | 'month' | 'year';

const naturalRange = (b: UiBucket): [Dayjs, Dayjs] => {
  const now = dayjs();
  switch (b) {
    case 'day':
      return [now.startOf('day'), now.endOf('day')];
    case 'week': {
      const weekday = (now.day() + 6) % 7;
      const s = now.subtract(weekday, 'day').startOf('day');
      return [s, s.add(6, 'day').endOf('day')];
    }
    case 'month':
      return [now.startOf('month'), now.endOf('month')];
    case 'year':
      return [now.startOf('year'), now.endOf('year')];
  }
};

const metricsLabel = (iso: string, b: UiBucket) => {
  const d = dayjs(iso);
  if (b === 'day') return d.format('HH:00');
  if (b === 'week' || b === 'month') return d.format('MMM D');
  return d.format('MMM');
};

const subsLabel = (period: string, b: UiBucket) => {
  if (b === 'day') {
    const d = dayjs(period, 'YYYY-MM-DD HH:mm:ss', true);
    return d.isValid() ? d.format('HH:00') : period;
  }
  if (b === 'week' || b === 'month') {
    const d = dayjs(period, 'YYYY-MM-DD', true);
    return d.isValid() ? d.format('MMM D') : period;
  }
  const d = dayjs(period, 'YYYY-MM', true);
  return d.isValid() ? d.format('MMM') : period;
};

const formatBytes = (n?: number | null) => {
  if (!n || n <= 0) return '0 B';
  const k = 1024;
  const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'] as const;
  const i = Math.floor(Math.log(n) / Math.log(k));
  const v = n / Math.pow(k, i);
  return `${v.toFixed(v >= 100 ? 0 : v >= 10 ? 1 : 2)} ${units[i]}`;
};

const radiusFor = (b: UiBucket) => ({ day: 4, week: 6, month: 8, year: 10 })[b];

function useViewportClamp(pad = 16) {
  const ref = useRef<HTMLDivElement | null>(null);
  const [height, setHeight] = useState<number>(480);
  const measure = useCallback(() => {
    const el = ref.current;
    if (!el) return;
    const top = el.getBoundingClientRect().top;
    const vh = window.innerHeight || document.documentElement.clientHeight;
    const h = Math.max(320, Math.floor(vh - top - pad));
    setHeight(h);
  }, [pad]);
  useLayoutEffect(() => {
    const onResize = () => requestAnimationFrame(measure);
    requestAnimationFrame(measure);
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  }, [measure]);
  return { ref, height };
}

const StatCell: React.FC<{
  title: React.ReactNode;
  value: number | string | null | undefined;
  suffix?: React.ReactNode;
  precision?: number;
  extra?: React.ReactNode;
}> = ({ title, value, suffix, precision, extra }) => {
  const isEmpty =
    value === null ||
    value === undefined ||
    value === '—' ||
    (typeof value === 'number' && Number.isNaN(value));
  const p = typeof value === 'number' ? precision : undefined;
  return (
    <div className="min-w-0">
      {isEmpty ? (
        <>
          <div className="text-xs font-medium opacity-70">{title}</div>
          <div className="text-lg font-semibold mt-1">—</div>
        </>
      ) : (
        <>
          <Statistic
            title={<span className="text-xs opacity-70">{title}</span>}
            value={value as any}
            precision={p}
            suffix={suffix}
            valueStyle={{ fontSize: 20 }}
          />
          {extra ? <div className="text-xs opacity-60 -mt-1">{extra}</div> : null}
        </>
      )}
    </div>
  );
};

const AdminDashboard: React.FC = () => {
  const { isDarkMode } = useTheme();
  const { isXl, isMobile } = useUI();

  // live system health (admin-only stream)
  const [live, setLive] = useState<PayloadOf<'system.health_admin'> | null>(null);

  // optional override set via the modal (kept same name you used before)
  const [maxConcurrent, setMaxConcurrent] = useState<number | null>(null);

  // saving flag while we hit the REST endpoint to update maxConcurrent
  const [saving, setSaving] = useState<boolean>(false);
  const [capOpen, setCapOpen] = useState(false);

  // Subscribe to the admin topic and keep `live` updated
  useWsEvents([Topics.systemAdmin()], {
    'system.health_admin': (p) => setLive(p),
  });

  const refreshMaxConcurrent = useCallback(async () => {
    try {
      const res = await getMaxConcurrent();
      if (res.success && typeof res.data === 'number') {
        setMaxConcurrent(res.data);
      } else {
        throw new Error(res.message || 'Failed to load max concurrency');
      }
    } catch (e: any) {
      message.error(e?.message ?? 'Failed to load max concurrency');
    }
  }, []);

  const updateMaxConcurrent = useCallback(async (next: number) => {
    setSaving(true);
    try {
      const res = await setMaxConcurrentApi(next);
      if (!res.success) throw new Error(res.message || 'Update failed');

      // server echoes the number in data
      const newVal = typeof res.data === 'number' ? res.data : next;
      setMaxConcurrent(newVal);
      message.success('Updated code manager capacity');
      return { success: true };
    } catch (e: any) {
      message.error(e?.message ?? 'Failed to update capacity');
      return { success: false };
    } finally {
      setSaving(false);
    }
  }, []);

  // NOTE: if you ALSO want the general stream (non-admin) available to the same page,
  // you can add Topics.system() and a handler for 'system.health' too.
  // Example:
  // useWsEvents([Topics.system()], { 'system.health': (p) => {/* optional */} });

  const [mBucket, setMBucket] = useState<UiBucket>('day');
  const [mRange, setMRange] = useState<[Dayjs, Dayjs]>(() => naturalRange('day'));
  const [metrics, setMetrics] = useState<MetricsPoint[]>([]);

  const [sBucket, setSBucket] = useState<UiBucket>('day');
  const [sRange, setSRange] = useState<[Dayjs, Dayjs]>(() => naturalRange('day'));
  const [subs, setSubs] = useState<SubmissionsPoint[]>([]);

  const unwrap = <T,>(res: any): T[] =>
    !res
      ? []
      : 'success' in res
        ? res.success
          ? (res.data?.points ?? [])
          : []
        : (res.points ?? []);

  const reload = useCallback(async () => {
    const [ms, me] = [mRange[0].toDate().toISOString(), mRange[1].toDate().toISOString()];
    const [ss, se] = [sRange[0].toDate().toISOString(), sRange[1].toDate().toISOString()];
    try {
      const [mr, sr] = await Promise.all([
        getSystemMetrics({ start: ms, end: me, bucket: mBucket }),
        getSubmissionsOverTime({ start: ss, end: se, bucket: sBucket }),
      ]);
      setMetrics(unwrap<MetricsPoint>(mr));
      setSubs(unwrap<SubmissionsPoint>(sr));
    } catch {
      setMetrics([]);
      setSubs([]);
    }
  }, [mRange, mBucket, sRange, sBucket]);

  useEffect(() => void reload(), [reload]);

  const metricsSeries = useMemo(() => {
    const rows: { x: string; series: string; value: number }[] = [];
    for (const p of metrics) {
      const x = metricsLabel(p.ts, mBucket);
      const cpu = Number((p as any).cpu_avg);
      const ram = Number((p as any).mem_pct);

      if (Number.isFinite(cpu)) rows.push({ x, series: 'CPU', value: cpu });
      if (Number.isFinite(ram)) rows.push({ x, series: 'RAM', value: ram });
    }
    return rows;
  }, [metrics, mBucket]);

  const subsSeries = useMemo(() => {
    const rows: { x: string; count: number }[] = [];
    for (const p of subs) {
      const x = subsLabel(p.period, sBucket);
      const count = Number((p as any).count);
      if (Number.isFinite(count)) rows.push({ x, count });
    }
    return rows;
  }, [subs, sBucket]);

  const metricsCats = useMemo(() => {
    const seen = new Set<string>();
    const arr: string[] = [];
    for (const p of metrics) {
      const x = metricsLabel(p.ts, mBucket);
      if (!seen.has(x)) {
        seen.add(x);
        arr.push(x);
      }
    }
    return arr;
  }, [metrics, mBucket]);

  const subsCats = useMemo(() => {
    const seen = new Set<string>();
    const arr: string[] = [];
    for (const p of subs) {
      const x = subsLabel(p.period, sBucket);
      if (!seen.has(x)) {
        seen.add(x);
        arr.push(x);
      }
    }
    return arr;
  }, [subs, sBucket]);

  const makeEverySecondFormatter = (bucket: UiBucket, cats: string[]) => {
    if (bucket === 'day' || bucket === 'month') {
      const keep = new Set<string>();
      cats.forEach((label, idx) => {
        if (idx % 2 === 0) keep.add(label);
      });
      return ((text: string) => (keep.has(text) ? text : '')) as any;
    }
    return ((text: string) => text) as any;
  };

  const metricsRadius = radiusFor(mBucket);
  const subsRadius = radiusFor(sBucket);

  const metricsCfg: ColumnConfig = {
    theme: isDarkMode ? 'dark' : 'light',
    data: metricsSeries,
    xField: 'x',
    yField: 'value',
    seriesField: 'series',
    isGroup: true,
    autoFit: true,
    minColumnWidth: 2,
    maxColumnWidth: 18,
    xAxis: {
      label: {
        autoHide: false,
        autoRotate: false,
        formatter: makeEverySecondFormatter(mBucket, metricsCats),
      } as any,
    },
    yAxis: {
      min: 0,
      max: 100,
      nice: false,
      title: { text: 'CPU / RAM (%)' },
      label: {
        formatter: (v: any) => {
          const n = Number(v);
          if (!Number.isFinite(n)) return '';
          return n >= 10 ? `${n.toFixed(0)}%` : `${n.toFixed(1)}%`;
        },
      },
    },
    style: { radiusTopLeft: metricsRadius, radiusTopRight: metricsRadius },
    padding: [12, 8, 32, 56],
    legend: { position: 'top' },
    animation: false,
  };

  const subsCfg: ColumnConfig = {
    theme: isDarkMode ? 'dark' : 'light',
    data: subsSeries,
    xField: 'x',
    yField: 'count',
    autoFit: true,
    minColumnWidth: 2,
    maxColumnWidth: 18,
    xAxis: {
      label: {
        autoHide: false,
        autoRotate: false,
        formatter: makeEverySecondFormatter(sBucket, subsCats),
      } as any,
    },
    yAxis: { min: 0, nice: true, title: { text: 'Submissions' } },
    style: { radiusTopLeft: subsRadius, radiusTopRight: subsRadius },
    padding: [12, 8, 32, 56],
    legend: false,
    animation: false,
  };

  // clamp page + chart boxes (UNCHANGED)
  const { ref: pageRef, height: pageH } = useViewportClamp(16);
  const chartsColumnRef = useRef<HTMLDivElement | null>(null);
  const [chartHeight, setChartHeight] = useState<number>(240);
  const measureCharts = useCallback(() => {
    const container = chartsColumnRef.current;
    if (!container) return;
    const gap = 16;
    const bounds = container.getBoundingClientRect();
    const total = bounds.height;
    const available = Math.max(0, total - gap);
    const next = Math.max(180, Math.floor((available - 164) / 2));
    setChartHeight((prev) => (Math.abs(prev - next) > 4 ? next : prev));
  }, []);
  const metricsChartHeight = chartHeight;
  const subsChartHeight = chartHeight;

  // live values (no toNumber)
  const { cpuPerCore, cpuCoreCount, cpuAvgNum } = useMemo(() => {
    const perCore: number[] = Array.isArray(live?.cpu?.per_core)
      ? (live!.cpu!.per_core as number[]).map((v) => Math.min(100, Math.max(0, v ?? 0)))
      : [];
    const countFromPayload =
      typeof live?.cpu?.cores === 'number' ? (live!.cpu!.cores as number) : null;
    const coreCount = countFromPayload ?? (perCore.length ? perCore.length : null);
    const avgFromCores = perCore.length
      ? perCore.reduce((sum, value) => sum + value, 0) / perCore.length
      : null;
    const avg =
      typeof live?.cpu?.avg_usage === 'number' ? (live!.cpu!.avg_usage as number) : avgFromCores;
    return { cpuPerCore: perCore, cpuCoreCount: coreCount, cpuAvgNum: avg ?? null };
  }, [live?.cpu]);

  useLayoutEffect(() => {
    measureCharts();
  }, [measureCharts, pageH, cpuPerCore.length]);

  useEffect(() => {
    const handler = () => measureCharts();
    window.addEventListener('resize', handler);
    return () => window.removeEventListener('resize', handler);
  }, [measureCharts]);

  const memUsed = typeof live?.memory?.used === 'number' ? live!.memory!.used : null;
  const memTotal = typeof live?.memory?.total === 'number' ? live!.memory!.total : null;
  const memUsedStr = formatBytes(memUsed ?? undefined);
  const memTotalStr = formatBytes(memTotal ?? undefined);
  const memPctNum =
    memUsed != null && memTotal != null && memTotal > 0
      ? Math.round((memUsed / memTotal) * 100)
      : null;

  const swapTotal = typeof live?.memory?.swap_total === 'number' ? live!.memory!.swap_total : null;
  const swapUsed = typeof live?.memory?.swap_used === 'number' ? live!.memory!.swap_used : null;
  const swapPctNum =
    swapTotal != null && swapTotal > 0 ? Math.round(((swapUsed ?? 0) / swapTotal) * 100) : null;

  type DiskRow = {
    name?: string;
    total?: number;
    available?: number;
    file_system?: string;
    mount_point?: string;
  };
  const rawDisks: DiskRow[] = Array.isArray(live?.disks) ? (live!.disks as any) : [];
  const disks = useMemo((): DiskRow[] => {
    const map = new Map<string, DiskRow>();
    for (const d of rawDisks) {
      const key = `${d.file_system ?? ''}|${d.total ?? 0}|${d.name ?? ''}`;
      const prev = map.get(key);
      if (!prev) {
        map.set(key, d);
        continue;
      }
      const curMp = `${d.mount_point ?? ''}`;
      const prevMp = `${prev.mount_point ?? ''}`;
      const preferCur =
        (curMp === '/' && prevMp !== '/') ||
        (prevMp !== '/' && curMp.length > 0 && curMp.length < prevMp.length);
      if (preferCur) map.set(key, d);
    }
    return Array.from(map.values());
  }, [rawDisks]);

  const diskCount = disks.length;

  const effectiveMax =
    typeof maxConcurrent === 'number'
      ? maxConcurrent
      : typeof live?.code_manager?.max_concurrent === 'number'
        ? live.code_manager.max_concurrent
        : null;

  const [maxModalOpen, setMaxModalOpen] = useState(false);
  const [maxDraft, setMaxDraft] = useState<number | undefined>(undefined);
  useEffect(() => {
    if (!maxModalOpen) return;
    setMaxDraft(typeof effectiveMax === 'number' ? effectiveMax : undefined);
  }, [effectiveMax, maxModalOpen]);

  const maxDraftValid = typeof maxDraft === 'number' && maxDraft > 0;
  const handleSaveMax = useCallback(async () => {
    if (!maxDraftValid || typeof maxDraft !== 'number' || maxDraft <= 0) return;
    const res = await updateMaxConcurrent(maxDraft);
    if (res.success) {
      await refreshMaxConcurrent();
      setMaxModalOpen(false);
    }
  }, [maxDraft, maxDraftValid, refreshMaxConcurrent, updateMaxConcurrent]);

  const handleExportMetrics = useCallback(() => {
    void exportSystemMetrics({
      start: mRange?.[0]?.toDate().toISOString(),
      end: mRange?.[1]?.toDate().toISOString(),
      bucket: mBucket,
    });
  }, [mBucket, mRange]);
  const handleExportSubmissions = useCallback(() => {
    void exportSubmissionsOverTime({
      start: sRange?.[0]?.toDate().toISOString(),
      end: sRange?.[1]?.toDate().toISOString(),
      bucket: sBucket,
    });
  }, [sBucket, sRange]);

  const cpuCoresDisplay = cpuCoreCount != null ? `${cpuCoreCount} cores` : 'Core count unavailable';
  const [showPerCoreDetails, setShowPerCoreDetails] = useState(false);
  // right above the return (inside component)
  const uptimeSeconds = Number(live?.uptime_seconds) || 0;
  // "3 hours", "a minute", etc (no "ago" suffix)
  const uptimeLabel =
    uptimeSeconds > 0 ? dayjs().subtract(uptimeSeconds, 'second').fromNow(true) : null;
  // optional exact tooltip text like "up since 2025-09-18 12:34:56"
  const uptimeExact =
    uptimeSeconds > 0
      ? dayjs().subtract(uptimeSeconds, 'second').format('YYYY-MM-DD HH:mm:ss')
      : null;
  const updatedStr = typeof live?.ts === 'string' ? dayjs(live.ts).format('HH:mm:ss') : null;

  return (
    <div ref={pageRef} style={{ height: pageH }} className="h-full">
      <div
        className="grid h-full gap-4"
        style={isXl ? { gridTemplateColumns: 'minmax(0,0.85fr) minmax(0,1.15fr)' } : undefined}
      >
        <div className="h-full flex flex-col">
          <Card
            className="flex-1 rounded-2xl !border-gray-200 dark:!border-gray-800 !shadow-none"
            style={{ boxShadow: 'none' }}
            styles={{
              body: {
                padding: 16,
                height: '100%',
                display: 'flex',
                flexDirection: 'column',
                gap: 16,
                minHeight: 0,
              },
              header: { padding: '12px 16px' },
            }}
            title={<span className="font-semibold">System health</span>}
            extra={
              <Space size={8} wrap>
                {uptimeLabel ? (
                  <Tooltip
                    title={
                      <div className="text-xs">
                        {uptimeExact ? <div>Up since {uptimeExact}</div> : null}
                        {updatedStr ? <div>Updated {updatedStr}</div> : null}
                      </div>
                    }
                  >
                    <Tag color={isDarkMode ? 'geekblue' : 'blue'} style={{ marginRight: 0 }}>
                      Up {uptimeLabel}
                    </Tag>
                  </Tooltip>
                ) : null}
              </Space>
            }
          >
            {/* === Top: CPU & RAM side-by-side (50/50) with AntD Divider === */}
            <div className="flex flex-col sm:flex-row gap-4">
              {/* CPU column */}
              <div className="sm:flex-1 flex flex-col gap-3">
                <div className="flex items-end justify-between">
                  <div>
                    <div className="text-xs font-medium opacity-70">CPU</div>
                    <div className="text-3xl font-semibold">
                      {cpuAvgNum != null ? `${cpuAvgNum.toFixed(1)}%` : '—'}
                    </div>
                  </div>
                  <div className="text-xs text-gray-500 dark:text-gray-400">
                    {cpuCoreCount != null ? `${cpuCoreCount} cores` : cpuCoresDisplay}
                  </div>
                </div>

                {cpuPerCore.length > 0 ? (
                  <MiniBars
                    values={cpuPerCore}
                    ariaLabel="CPU per-core utilisation"
                    containerHeight={96}
                  />
                ) : (
                  <div className="text-xs text-gray-500 dark:text-gray-400">
                    Waiting for per-core telemetry…
                  </div>
                )}

                {cpuPerCore.length > 0 && (
                  <>
                    <div className="flex items-center justify-end">
                      <button
                        className="text-[11px] text-blue-600 dark:text-blue-400 hover:underline"
                        onClick={() => setShowPerCoreDetails((s) => !s)}
                      >
                        {showPerCoreDetails ? 'Hide details' : 'Show details'}
                      </button>
                    </div>
                    {showPerCoreDetails && (
                      <div
                        className="grid gap-2 pr-1"
                        style={{
                          gridTemplateColumns: 'repeat(auto-fill, minmax(140px, 1fr))',
                          maxHeight: 220,
                          overflowY: 'auto',
                        }}
                      >
                        {cpuPerCore.map((pct, idx) => (
                          <MeterRow
                            key={idx}
                            label={`Core ${idx + 1}`}
                            valueText={`${pct.toFixed(1)}%`}
                            percent={pct}
                          />
                        ))}
                      </div>
                    )}
                  </>
                )}

                {isMobile && (
                  <div className="grid grid-cols-3 gap-3">
                    <StatCell
                      title="Load 1m"
                      value={typeof live?.load?.one === 'number' ? live.load.one : null}
                      precision={2}
                    />
                    <StatCell
                      title="Load 5m"
                      value={typeof live?.load?.five === 'number' ? live.load.five : null}
                      precision={2}
                    />
                    <StatCell
                      title="Load 15m"
                      value={typeof live?.load?.fifteen === 'number' ? live.load.fifteen : null}
                      precision={2}
                    />
                  </div>
                )}
              </div>

              {/* Vertical divider only on sm+ */}
              <Divider
                type="vertical"
                className="hidden sm:block self-stretch m-0"
                style={{ height: 'auto' }}
              />

              {/* RAM column */}
              <div className="sm:flex-1 flex flex-col gap-3">
                <div className="text-xs font-medium opacity-70">RAM</div>
                <div className="flex items-center gap-4">
                  <Progress
                    type="dashboard"
                    percent={memPctNum ?? 0}
                    size={128}
                    format={(p) => (p != null ? `${p}%` : '—')}
                  />

                  {/* stack vertically */}
                  <div className="flex-1 flex flex-col gap-3">
                    <StatCell
                      title={
                        <span className="inline-flex items-center gap-1">
                          Used
                          <Tooltip
                            placement="top"
                            title={
                              <div className="text-xs">
                                <div>
                                  <strong>Memory:</strong> {memUsedStr} / {memTotalStr}
                                </div>
                              </div>
                            }
                          >
                            <InfoCircleOutlined className="text-gray-400 hover:text-gray-500 dark:text-gray-400 dark:hover:text-gray-300 cursor-help" />
                          </Tooltip>
                        </span>
                      }
                      value={memPctNum}
                      suffix="%"
                    />

                    <StatCell
                      title={
                        <span className="inline-flex items-center gap-1">
                          Swap used
                          <Tooltip
                            placement="top"
                            title={
                              typeof swapTotal === 'number' ? (
                                <div className="text-xs">
                                  <div>
                                    <strong>Swap:</strong> {formatBytes(swapUsed ?? 0)} /{' '}
                                    {formatBytes(swapTotal)}
                                  </div>
                                </div>
                              ) : (
                                <div className="text-xs">Swap not available</div>
                              )
                            }
                          >
                            <InfoCircleOutlined className="text-gray-400 hover:text-gray-500 dark:text-gray-400 dark:hover:text-gray-300 cursor-help" />
                          </Tooltip>
                        </span>
                      }
                      value={typeof swapPctNum === 'number' ? swapPctNum : null}
                      suffix="%"
                    />
                  </div>
                </div>
              </div>
            </div>

            <Divider className="my-2" />

            {/* === Disks (individual only, no aggregate) === */}
            <div className="flex flex-col gap-2">
              <div className="flex items-center justify-between">
                <div className="text-xs font-medium opacity-70">Disks</div>
                <div className="text-xs opacity-70">
                  {diskCount} {diskCount === 1 ? 'disk' : 'disks'}
                </div>
              </div>

              <div className="grid gap-2 sm:grid-cols-2 xl:grid-cols-3">
                {disks.slice(0, 6).map((d, i) => {
                  const total = d?.total ?? 0;
                  const avail = d?.available ?? 0;
                  const used = Math.max(0, total - avail);
                  const pct = total > 0 ? Math.round((used / total) * 100) : 0;
                  const mp = (d?.mount_point ?? '') as string;
                  const mpHint = mp && mp !== '/' ? ` — ${mp}` : '';
                  return (
                    <div key={i} className="flex flex-col gap-1">
                      <div className="flex items-center justify-between text-[11px] text-gray-500 dark:text-gray-400">
                        <span className="truncate">
                          {d?.name ?? 'disk'}
                          {d?.file_system ? (
                            <span className="opacity-60"> ({d.file_system})</span>
                          ) : null}
                          {mpHint ? <span className="opacity-50">{mpHint}</span> : null}
                        </span>
                        <span className="tabular-nums">
                          {formatBytes(used)} / {formatBytes(total)}
                        </span>
                      </div>
                      <PercentBar percent={pct} ariaLabel={`Disk ${d?.name ?? i} used percent`} />
                    </div>
                  );
                })}
              </div>
            </div>

            {/* === Code manager (mobile shown above button) === */}
            {isMobile && (
              <div className="flex flex-col gap-2 mt-2">
                <div className="text-xs font-medium opacity-70">Code manager</div>
                <div className="grid grid-cols-3 gap-3">
                  <StatCell
                    title="Running"
                    value={
                      typeof live?.code_manager?.running === 'number'
                        ? live.code_manager.running
                        : null
                    }
                  />
                  <StatCell
                    title="Waiting"
                    value={
                      typeof live?.code_manager?.waiting === 'number'
                        ? live.code_manager.waiting
                        : null
                    }
                  />
                  <StatCell
                    title="Max"
                    value={
                      typeof effectiveMax === 'number'
                        ? effectiveMax
                        : typeof live?.code_manager?.max_concurrent === 'number'
                          ? live.code_manager.max_concurrent
                          : null
                    }
                  />
                </div>
              </div>
            )}

            {isMobile && (
              <div className="pt-2">
                <Button
                  block
                  type="primary"
                  icon={<TeamOutlined />}
                  onClick={() => setCapOpen(true)}
                >
                  Update code manager capacity
                </Button>

                <CapacityModal
                  open={capOpen}
                  onClose={() => setCapOpen(false)}
                  onSaved={async () => {
                    // pull fresh value for the dashboard tiles
                    await refreshMaxConcurrent();
                  }}
                />
              </div>
            )}
          </Card>
        </div>

        {isXl && (
          <div ref={chartsColumnRef} className="min-h-0 flex flex-col gap-4 h-full">
            <Card
              className="flex-1 rounded-2xl !border-gray-200 dark:!border-gray-800"
              styles={{
                body: { padding: 12, height: '100%', display: 'flex', flexDirection: 'column' },
              }}
              title={<span className="font-semibold">Metrics</span>}
              extra={
                <Space wrap>
                  <DatePicker.RangePicker
                    value={mRange}
                    onChange={(v) => {
                      if (!v || v.length !== 2) return;
                      setMRange([v[0] as Dayjs, v[1] as Dayjs]);
                    }}
                  />
                  <Select
                    value={mBucket}
                    options={[
                      { value: 'day', label: 'Day' },
                      { value: 'week', label: 'Week' },
                      { value: 'month', label: 'Month' },
                      { value: 'year', label: 'Year' },
                    ]}
                    onChange={(v: UiBucket) => {
                      setMBucket(v);
                      setMRange(naturalRange(v));
                    }}
                    style={{ minWidth: 120 }}
                  />
                  <Button
                    icon={<DownloadOutlined />}
                    onClick={handleExportMetrics}
                    disabled={metricsSeries.length === 0}
                  >
                    Export CSV
                  </Button>
                </Space>
              }
            >
              <div className="flex-1 min-h-0 overflow-hidden">
                {metricsSeries.length === 0 ? (
                  <div className="h-full flex items-center justify-center text-sm text-gray-500">
                    No data in selected range.
                  </div>
                ) : (
                  <Column
                    {...metricsCfg}
                    height={metricsChartHeight}
                    onReady={(plot) => {
                      plot.on('plot:mouseleave', () => plot.chart?.hideTooltip());
                      plot.on('element:mouseleave', () => plot.chart?.hideTooltip());
                    }}
                  />
                )}
              </div>
            </Card>

            <Card
              className="flex-1 rounded-2xl !border-gray-200 dark:!border-gray-800"
              styles={{
                body: { padding: 12, height: '100%', display: 'flex', flexDirection: 'column' },
              }}
              title={<span className="font-semibold">Submissions</span>}
              extra={
                <Space wrap>
                  <DatePicker.RangePicker
                    value={sRange}
                    onChange={(v) => {
                      if (!v || v.length !== 2) return;
                      setSRange([v[0] as Dayjs, v[1] as Dayjs]);
                    }}
                  />
                  <Select
                    value={sBucket}
                    options={[
                      { value: 'day', label: 'Day' },
                      { value: 'week', label: 'Week' },
                      { value: 'month', label: 'Month' },
                      { value: 'year', label: 'Year' },
                    ]}
                    onChange={(v: UiBucket) => {
                      setSBucket(v);
                      setSRange(naturalRange(v));
                    }}
                    style={{ minWidth: 120 }}
                  />
                  <Button
                    icon={<DownloadOutlined />}
                    onClick={handleExportSubmissions}
                    disabled={subsSeries.length === 0}
                  >
                    Export CSV
                  </Button>
                </Space>
              }
            >
              <div className="flex-1 min-h-0 overflow-hidden">
                {subsSeries.length === 0 ? (
                  <div className="h-full flex items-center justify-center text-sm text-gray-500">
                    No data in selected range.
                  </div>
                ) : (
                  <Column
                    {...subsCfg}
                    height={subsChartHeight}
                    onReady={(plot) => {
                      plot.on('plot:mouseleave', () => plot.chart?.hideTooltip());
                      plot.on('element:mouseleave', () => plot.chart?.hideTooltip());
                    }}
                  />
                )}
              </div>
            </Card>
          </div>
        )}
      </div>

      <Modal
        title="Update code manager capacity"
        open={maxModalOpen}
        onCancel={() => setMaxModalOpen(false)}
        onOk={handleSaveMax}
        okText="Save"
        okButtonProps={{ disabled: !maxDraftValid }}
        confirmLoading={saving}
        destroyOnHidden
      >
        <Space direction="vertical" size={12} className="w-full">
          <div className="text-sm text-gray-600 dark:text-gray-300">
            Choose the maximum number of code runs that can execute concurrently.
          </div>
          <InputNumber
            min={1}
            style={{ width: '100%' }}
            value={maxDraft}
            onChange={(v) => setMaxDraft(typeof v === 'number' ? v : undefined)}
          />
          <div className="text-xs text-gray-500 dark:text-gray-400">
            Current effective limit: {typeof effectiveMax === 'number' ? effectiveMax : '—'}
          </div>
        </Space>
      </Modal>
    </div>
  );
};

export default AdminDashboard;
