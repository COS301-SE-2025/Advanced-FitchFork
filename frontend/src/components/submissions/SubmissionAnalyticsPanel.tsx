import React, { useMemo, useRef, useState, useLayoutEffect } from 'react';
import { Typography, Select, Empty, Statistic, theme, Tooltip } from 'antd';
import { InfoCircleOutlined } from '@ant-design/icons';
import { Column, type ColumnConfig } from '@ant-design/plots';
import type { Submission } from '@/types/modules/assignments/submissions';
import type { Assignment } from '@/types/modules/assignments';
import { useTheme } from '@/context/ThemeContext';

const { Title, Text } = Typography;

/* ---------------- Mock data ---------------- */
const MOCK_ASSIGNMENTS: Record<number, Assignment> = {
  701: { id: 701, module_id: 101, name: 'A1: Basics' } as any,
  702: { id: 702, module_id: 344, name: 'Prac 2: Sockets' } as any,
  703: { id: 703, module_id: 344, name: 'Project: Compiler' } as any,
};

// mock enrolled counts per assignment (for submission rate)
const MOCK_ENROLLED: Record<number, number> = {
  701: 160,
  702: 130,
  703: 200,
};

type SubWithRef = Submission & {
  assignment?: { id: number };
  assignment_id?: number;
  mark?: { earned: number; total: number };
};

const randomNormal = (mean = 68, std = 18) => {
  let u = 0,
    v = 0;
  while (u === 0) u = Math.random();
  while (v === 0) v = Math.random();
  const z = Math.sqrt(-2 * Math.log(u)) * Math.cos(2 * Math.PI * v);
  return Math.max(0, Math.min(100, Math.round(mean + std * z)));
};

const generateSubs = (assignmentId: number, count: number): SubWithRef[] => {
  const subs: SubWithRef[] = [];
  for (let i = 0; i < count; i++) {
    const pct = randomNormal(assignmentId === 703 ? 62 : 70, assignmentId === 703 ? 22 : 16);
    subs.push({
      id: Number(`${assignmentId}${i}`),
      assignment_id: assignmentId,
      mark: { earned: pct, total: 100 },
    } as any);
  }
  return subs;
};

const MOCK_SUBMISSIONS: SubWithRef[] = [
  ...generateSubs(701, 120),
  ...generateSubs(702, 92),
  ...generateSubs(703, 147),
];

/* ---------------- Utils ---------------- */
const toPct = (e?: number, t?: number) =>
  typeof e === 'number' && typeof t === 'number' && t > 0 ? Math.round((e / t) * 100) : null;

const LABELS = [
  '0–9',
  '10–19',
  '20–29',
  '30–39',
  '40–49',
  '50–59',
  '60–69',
  '70–79',
  '80–89',
  '90–100',
];
const bucketIndex = (s: number) => (s <= 9 ? 0 : s >= 90 ? 9 : Math.floor(s / 10));
const getAssignmentId = (s: SubWithRef) => s.assignment_id ?? s.assignment?.id;

const mean = (a: number[]) => (a.length ? a.reduce((x, y) => x + y, 0) / a.length : 0);
const median = (a: number[]) => {
  if (!a.length) return 0;
  const arr = [...a].sort((x, y) => x - y);
  const m = Math.floor(arr.length / 2);
  return arr.length % 2 ? arr[m] : (arr[m - 1] + arr[m]) / 2;
};
const quantile = (a: number[], q: number) => {
  if (!a.length) return 0;
  const arr = [...a].sort((x, y) => x - y);
  const pos = (arr.length - 1) * q;
  const base = Math.floor(pos);
  const rest = pos - base;
  return arr[base] + (arr[base + 1] !== undefined ? rest * (arr[base + 1] - arr[base]) : 0);
};
const stddev = (a: number[]) => {
  if (a.length < 2) return 0;
  const m = mean(a);
  return Math.sqrt(a.reduce((acc, x) => acc + (x - m) ** 2, 0) / (a.length - 1));
};
const passRate = (a: number[]) =>
  a.length ? (a.filter((x) => x >= 50).length / a.length) * 100 : 0;

/* ---------------- Component ---------------- */
const SubmissionAnalyticsPanel: React.FC = () => {
  const { isDarkMode } = useTheme();
  const { token } = theme.useToken();

  const assignmentOptions = Object.values(MOCK_ASSIGNMENTS).map((a) => ({
    label: a.name,
    value: a.id,
  }));
  const [assignmentId, setAssignmentId] = useState<number>(assignmentOptions[0]?.value);

  const { counts, total, percents } = useMemo(() => {
    const rows = MOCK_SUBMISSIONS.filter((s) => getAssignmentId(s) === assignmentId);
    const buckets = new Array(10).fill(0);
    const pcts: number[] = [];
    for (const s of rows) {
      const p = toPct(s.mark?.earned, s.mark?.total);
      if (p === null) continue;
      pcts.push(p);
      buckets[bucketIndex(p)]++;
    }
    return { counts: buckets as number[], total: rows.length, percents: pcts };
  }, [assignmentId]);

  // submission rate (mock enrolled per assignment)
  const enrolled = MOCK_ENROLLED[assignmentId] ?? Math.max(total, 1);
  const submitted = total;
  const notSubmitted = Math.max(0, enrolled - submitted);
  const submissionRate = Math.min(100, Math.max(0, (submitted / Math.max(enrolled, 1)) * 100));

  // chart sizing box
  const chartAbsRef = useRef<HTMLDivElement>(null);
  const [dims, setDims] = useState({ w: 0, h: 0 });
  useLayoutEffect(() => {
    const el = chartAbsRef.current;
    if (!el) return;
    const ro = new ResizeObserver(() => {
      setDims({ w: Math.floor(el.clientWidth), h: Math.floor(el.clientHeight) });
    });
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  // shape chart data
  const data = useMemo(
    () => LABELS.map((bucket, i) => ({ bucket, value: Number(counts[i] ?? 0) })),
    [counts],
  );

  const columnConfig: ColumnConfig = {
    theme: isDarkMode ? 'dark' : 'light',
    data,
    xField: 'bucket',
    yField: 'value',
    autoFit: true,
    height: Math.max(200, dims.h),
    padding: [16, 8, 40, 56],
    columnWidthRatio: 0.8,
    meta: { bucket: { alias: 'Score (%)' }, value: { alias: 'Submissions', nice: true, min: 0 } },
    xAxis: { title: { text: 'Score (%)' }, label: { autoRotate: false, autoHide: true } },
    yAxis: { title: { text: 'Submissions' } },
    label: {
      position: 'top',
      style: { fontSize: 12 },
      formatter: (d: { value?: number }) =>
        typeof d.value === 'number' && d.value > 0 ? String(d.value) : '',
    },
    style: { radiusTopLeft: 6, radiusTopRight: 6 },
    legend: false,
    animation: false,
    interaction: {
      tooltip: {
        render: (_e: unknown, { title, items }: { title?: string; items: any[] }) => (
          <div style={{ minWidth: 120 }}>
            {title ? (
              <Text strong style={{ display: 'block', marginBottom: 4, color: token.colorText }}>
                {title}
              </Text>
            ) : null}
            {items.map((item, idx) => {
              const raw = item?.value ?? item?.data?.value ?? item?.datum?.value ?? 0;
              const value = typeof raw === 'number' ? raw : Number(raw) || 0;
              return (
                <div
                  key={idx}
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    gap: 8,
                    margin: '2px 0',
                  }}
                >
                  <Text
                    style={{
                      display: 'inline-flex',
                      alignItems: 'center',
                      gap: 6,
                      color: token.colorTextSecondary,
                    }}
                  >
                    <i
                      style={{
                        width: 6,
                        height: 6,
                        borderRadius: '50%',
                        background: item?.color || token.colorPrimary,
                        display: 'inline-block',
                      }}
                    />
                    Submissions
                  </Text>
                  <Text strong style={{ color: token.colorText }}>
                    {value}
                  </Text>
                </div>
              );
            })}
          </div>
        ),
      },
    },
  };

  // derived stats
  const avg = mean(percents);
  const med = median(percents);
  const p75 = quantile(percents, 0.75);
  const sd = stddev(percents);
  const pr = passRate(percents);
  const min = percents.length ? Math.min(...percents) : 0;
  const max = percents.length ? Math.max(...percents) : 0;
  const modeBucketIdx = counts.indexOf(Math.max(...counts));
  const modeBucket = modeBucketIdx >= 0 ? LABELS[modeBucketIdx] : '—';

  return (
    <div className="h-full min-h-0 flex flex-col w-full bg-white dark:bg-gray-900 rounded-md border border-gray-200 dark:border-gray-800">
      {/* Header */}
      <div className="px-3 py-2 border-b border-gray-200 dark:border-gray-800">
        <div className="flex items-center justify-between gap-2">
          <Title level={5} className="!mb-0">
            Submission Analytics
          </Title>
          <div className="flex items-center gap-2 sm:gap-3">
            <Text type="secondary" className="hidden sm:inline text-xs">
              Total: {total}
            </Text>
            <Select
              size="small"
              value={assignmentId}
              onChange={setAssignmentId}
              options={assignmentOptions}
              className="min-w-[160px] sm:min-w-[200px] 2xl:min-w-[240px]"
            />
          </div>
        </div>
      </div>

      {/* Body */}
      <div className="flex-1 min-h-0 overflow-hidden">
        {total === 0 ? (
          <div className="h-full flex items-center justify-center p-3">
            <Empty description="No submissions for this assignment yet." />
          </div>
        ) : (
          <div className="h-full grid grid-cols-1 2xl:grid-cols-[2fr_1fr] gap-3 2xl:gap-x-0 2xl:gap-y-0">
            {/* Chart (2xl+) */}
            <div className="relative min-h-0 hidden 2xl:block">
              <div ref={chartAbsRef} className="absolute inset-0 p-3">
                <Column {...columnConfig} />
              </div>
            </div>

            {/* Stats column */}
            <div className="min-h-0 h-full 2xl:border-l 2xl:border-gray-200 dark:2xl:border-gray-800">
              <div className="h-full p-3">
                <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 xl:grid-cols-4 2xl:grid-cols-2 gap-3">
                  {/* NEW: Submission rate with tooltip */}
                  <Statistic
                    title={
                      <span className="inline-flex items-center gap-1.5">
                        <span className="opacity-70">Submission rate</span>
                        <Tooltip
                          title={
                            <div>
                              <span className="font-semibold">{submitted}</span>{' '}
                              <span className="opacity-75">submitted</span>
                              <br />
                              <span className="font-semibold">{notSubmitted}</span>{' '}
                              <span className="opacity-75">not submitted</span>
                              <br />
                              <span className="opacity-60">of {enrolled} enrolled</span>
                            </div>
                          }
                        >
                          <InfoCircleOutlined />
                        </Tooltip>
                      </span>
                    }
                    value={submissionRate.toFixed(1)}
                    suffix="%"
                  />

                  <Statistic
                    title={<Text type="secondary">Average</Text>}
                    value={avg.toFixed(1)}
                    suffix="%"
                  />
                  <Statistic
                    title={<Text type="secondary">Median</Text>}
                    value={med.toFixed(1)}
                    suffix="%"
                  />
                  <Statistic
                    title={<Text type="secondary">75th pct</Text>}
                    value={p75.toFixed(1)}
                    suffix="%"
                  />
                  <Statistic title={<Text type="secondary">Std dev</Text>} value={sd.toFixed(1)} />
                  <Statistic
                    title={<Text type="secondary">Pass rate</Text>}
                    value={pr.toFixed(1)}
                    suffix="%"
                  />
                  <Statistic title={<Text type="secondary">Mode bucket</Text>} value={modeBucket} />
                  <Statistic title={<Text type="secondary">Highest</Text>} value={max} suffix="%" />
                  <Statistic title={<Text type="secondary">Lowest</Text>} value={min} suffix="%" />
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default SubmissionAnalyticsPanel;
