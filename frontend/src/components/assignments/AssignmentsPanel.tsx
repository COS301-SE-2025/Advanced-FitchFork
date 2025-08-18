import { useMemo, useState } from 'react';
import { List, Typography, Segmented, Empty, Tooltip, Progress } from 'antd';
import { BookOutlined, ClockCircleOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import type { Assignment, AssignmentStatus, AssignmentType } from '@/types/modules/assignments';
import type { Module } from '@/types/modules';
import AssignmentTypeTag from './AssignmentTypeTag';
import AssignmentStatusTag from './AssignmentStatusTag';

dayjs.extend(relativeTime);

const { Text, Title } = Typography;
const now = () => dayjs();

// ---- Mock data ----
const MOCK_MODULES: Record<number, Module> = {
  101: {
    id: 101,
    code: 'COS101',
    year: 2025,
    description: 'Intro to CS',
    credits: 16,
    created_at: '',
    updated_at: '',
  },
  344: {
    id: 344,
    code: 'COS344',
    year: 2025,
    description: 'Systems',
    credits: 24,
    created_at: '',
    updated_at: '',
  },
  222: {
    id: 222,
    code: 'COS222',
    year: 2025,
    description: 'Algorithms',
    credits: 24,
    created_at: '',
    updated_at: '',
  },
};

const mk = (
  id: number,
  module_id: number,
  name: string,
  type: AssignmentType,
  openInHours: number,
  dueInHours: number,
  status: AssignmentStatus,
): Assignment =>
  ({
    id,
    module_id,
    name,
    description: '',
    assignment_type: type,
    available_from: now().add(openInHours, 'hour').toISOString(),
    due_date: now().add(dueInHours, 'hour').toISOString(),
    status,
    created_at: '',
    updated_at: '',
  }) as any;

const MOCK_ASSIGNMENTS: Assignment[] = [
  mk(501, 101, 'A1: Basics', 'assignment', -48, 72, 'open'),
  mk(502, 344, 'Prac 2: Sockets', 'practical', 6, 54, 'ready'),
  mk(503, 101, 'A2: Data Structures', 'assignment', 24, 192, 'setup'),
  mk(504, 344, 'Prac 3: Threads', 'practical', -12, 12, 'open'),
  mk(505, 101, 'A0: Warm-up', 'assignment', -240, -12, 'closed'),
  mk(506, 222, 'A3: Graphs', 'assignment', 36, 240, 'setup'),
  mk(507, 222, 'Prac 1: Tooling', 'practical', -3, 48, 'open'),
  mk(508, 101, 'A4: Sorting', 'assignment', 12, 120, 'ready'),
  mk(509, 344, 'Prac 4: Concurrency', 'practical', -23, 1, 'open'),
];

// ---- Helpers ----
function progressPercent(a: Assignment): number | null {
  if (a.status !== 'open') return null;
  const start = dayjs(a.available_from);
  const end = dayjs(a.due_date);
  const n = now();
  if (start.isAfter(n) || !end.isAfter(n)) return null;

  const total = end.diff(start);
  const spent = n.diff(start);

  // normal pct: spent / total → now invert it
  const remainingPct = Math.min(100, Math.max(0, 100 - (spent / total) * 100));

  return Number.isFinite(remainingPct) ? Math.round(remainingPct) : null;
}

function progressColor(pct: number): string {
  // pct now represents "time remaining", so redder as it gets lower
  if (pct > 50) return '#1677ff'; // blue for lots of time left
  if (pct > 20) return '#fa8c16'; // orange when mid
  return '#ff4d4f'; // red when nearly out of time
}

// ---- Component ----
const AssignmentsPanel: React.FC = () => {
  const [view, setView] = useState<'due' | 'upcoming'>('due');
  const [items] = useState<Assignment[]>(MOCK_ASSIGNMENTS);

  const filtered = useMemo(() => {
    const n = now();

    if (view === 'due') {
      // started and not yet due
      return items
        .filter((a) => a.status !== 'archived')
        .filter((a) => !dayjs(a.available_from).isAfter(n))
        .filter((a) => dayjs(a.due_date).isAfter(n))
        .sort((a, b) => dayjs(a.due_date).valueOf() - dayjs(b.due_date).valueOf());
    }

    // upcoming (opens in future)
    return items
      .filter((a) => a.status !== 'archived')
      .filter((a) => dayjs(a.available_from).isAfter(n))
      .sort((a, b) => dayjs(a.available_from).valueOf() - dayjs(b.available_from).valueOf());
  }, [items, view]);

  return (
    <div className="h-full min-h-0 flex flex-col w-full bg-white dark:bg-gray-900 rounded-md border border-gray-200 dark:border-gray-800">
      {/* Header */}
      <div className="px-3 py-2 border-b border-gray-200 dark:border-gray-800">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <BookOutlined className="text-gray-500" />
            <Title level={5} className="!mb-0">
              Assignments
            </Title>
          </div>
          <Segmented
            value={view}
            onChange={(v) => setView(v as 'due' | 'upcoming')}
            options={[
              { label: 'Due', value: 'due' },
              { label: 'Upcoming', value: 'upcoming' },
            ]}
            size="small"
            className="role-seg"
          />
        </div>
      </div>

      {/* List */}
      <List
        className="flex-1 overflow-y-auto"
        locale={{
          emptyText: (
            <Empty
              image={Empty.PRESENTED_IMAGE_SIMPLE}
              description={
                view === 'due' ? 'No assignments currently open.' : 'No assignments opening soon.'
              }
            />
          ),
        }}
        dataSource={filtered}
        renderItem={(a) => {
          const moduleCode = MOCK_MODULES[a.module_id]?.code ?? `M-${a.module_id}`;
          const due = dayjs(a.due_date);
          const opens = dayjs(a.available_from);
          const pct = progressPercent(a);

          const timing =
            view === 'upcoming' ? (
              <div className="flex items-center gap-2">
                <Text type="secondary">{moduleCode}</Text>
                <Text type="secondary" className="!text-[12px]">
                  •
                </Text>
                <Text type="secondary" className="inline-flex items-center !text-[12px]">
                  opens {opens.fromNow()}
                  <Tooltip title={opens.format('YYYY-MM-DD HH:mm')}>
                    <ClockCircleOutlined className="ml-1" />
                  </Tooltip>
                </Text>
              </div>
            ) : (
              <div className="flex items-center gap-2">
                <Text type="secondary" className="!text-[12px]">
                  {moduleCode}
                </Text>
                <Text type="secondary" className="!text-[12px]">
                  •
                </Text>
                <Text type="secondary" className="inline-flex items-center !text-[12px]">
                  due {due.fromNow()}
                  <Tooltip title={due.format('YYYY-MM-DD HH:mm')}>
                    <ClockCircleOutlined className="ml-1" />
                  </Tooltip>
                </Text>
              </div>
            );

          return (
            <List.Item
              className="!px-3 cursor-pointer"
              onClick={() => console.log('Open assignment', { id: a.id, name: a.name })}
            >
              <div className="flex flex-col gap-1.5 w-full">
                {/* Row 1: name + tags */}
                <div className="flex items-center justify-between gap-2 min-w-0">
                  <Text strong className="truncate">
                    {a.name}
                  </Text>
                  <div className="flex items-center gap-2 shrink-0">
                    <div className="hidden sm:block">
                      <AssignmentTypeTag type={a.assignment_type} />
                    </div>
                    <AssignmentStatusTag status={a.status} />
                  </div>
                </div>

                {/* Row 2: timing */}
                {timing}

                {/* Row 3: progress (only when open) */}
                {typeof pct === 'number' && (
                  <div className="pt-1">
                    <Progress
                      percent={pct}
                      size="small"
                      showInfo={false}
                      strokeLinecap="round"
                      strokeColor={progressColor(pct)}
                    />
                  </div>
                )}
              </div>
            </List.Item>
          );
        }}
      />
    </div>
  );
};

export default AssignmentsPanel;
