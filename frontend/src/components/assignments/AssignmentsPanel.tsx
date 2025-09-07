import { useEffect, useMemo, useState } from 'react';
import { List, Typography, Segmented, Empty, Tooltip, Progress, message } from 'antd';
import { BookOutlined, ClockCircleOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';

import type { ModuleRole } from '@/types/modules';
import type { AssignmentStatus, AssignmentType } from '@/types/modules/assignments';
import AssignmentTypeTag from './AssignmentTypeTag';
import AssignmentStatusTag from './AssignmentStatusTag';
import { getMyAssignments } from '@/services/me/assignments/get';
import { useNavigate } from 'react-router-dom';

dayjs.extend(relativeTime);

const { Text, Title } = Typography;
const now = () => dayjs();

type Props = { role?: ModuleRole };

type DisplayAssignment = {
  id: number;
  name: string;
  status: AssignmentStatus | string;
  available_from: string;
  due_date: string;
  created_at: string;
  updated_at: string;
  module: { id: number; code: string };
  assignment_type?: AssignmentType;
};

function progressPercent(a: DisplayAssignment): number | null {
  if (a.status !== 'open') return null;
  const start = dayjs(a.available_from);
  const end = dayjs(a.due_date);
  const n = now();
  if (start.isAfter(n) || !end.isAfter(n)) return null;
  const total = end.diff(start);
  const spent = n.diff(start);
  const remainingPct = Math.min(100, Math.max(0, 100 - (spent / total) * 100));
  return Number.isFinite(remainingPct) ? Math.round(remainingPct) : null;
}

function progressColor(pct: number): string {
  if (pct > 50) return '#1677ff';
  if (pct > 20) return '#fa8c16';
  return '#ff4d4f';
}

const AssignmentsPanel: React.FC<Props> = ({ role }) => {
  const [view, setView] = useState<'due' | 'upcoming'>('due');
  const [items, setItems] = useState<DisplayAssignment[]>([]);
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();

  useEffect(() => {
    let cancelled = false;
    (async () => {
      setLoading(true);
      try {
        const res = await getMyAssignments({ role, page: 1, per_page: 50 });
        if (!res?.data) throw new Error('Empty response');
        if (!res.success) throw new Error(res.message || 'Request failed');

        const mapped: DisplayAssignment[] = res.data.assignments.map((a) => ({
          id: a.id,
          name: a.name,
          status: a.status,
          available_from: a.available_from,
          due_date: a.due_date,
          created_at: a.created_at,
          updated_at: a.updated_at,
          module: a.module,
          assignment_type: 'assignment', // fallback so tag renders
        }));

        if (!cancelled) setItems(mapped);
      } catch (err: any) {
        if (!cancelled) {
          message.error(err?.message ?? 'Failed to load assignments');
          setItems([]);
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [role]);

  const filtered = useMemo(() => {
    const n = now();
    if (view === 'due') {
      return items
        .filter((a) => a.status !== 'archived')
        .filter((a) => !dayjs(a.available_from).isAfter(n))
        .filter((a) => dayjs(a.due_date).isAfter(n))
        .sort((a, b) => dayjs(a.due_date).valueOf() - dayjs(b.due_date).valueOf());
    }
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
        loading={loading}
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
          const due = dayjs(a.due_date);
          const opens = dayjs(a.available_from);
          const pct = progressPercent(a);

          const timing =
            view === 'upcoming' ? (
              <div className="flex items-center gap-2">
                <Text type="secondary">{a.module.code}</Text>
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
                  {a.module.code}
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
              onClick={() => navigate(`/modules/${a.module.id}/assignments/${a.id}`)}
            >
              <div className="flex flex-col gap-1.5 w-full">
                {/* Row 1: name + tags */}
                <div className="flex items-center justify-between gap-2 min-w-0">
                  <Text strong className="truncate">
                    {a.name}
                  </Text>
                  <div className="flex items-center gap-2 shrink-0">
                    <div className="hidden sm:block">
                      <AssignmentTypeTag type={a.assignment_type ?? 'assignment'} />
                    </div>
                    <AssignmentStatusTag status={a.status as AssignmentStatus} />
                  </div>
                </div>
                {timing}
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
