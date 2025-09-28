import { useEffect, useMemo, useState } from 'react';
import { List, Typography, Segmented, Empty, Tooltip, Progress, message, Tag } from 'antd';
import { BookOutlined, ClockCircleOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';

import type { ModuleRole } from '@/types/modules';
import type { AssignmentReadiness, AssignmentStatus, AssignmentType } from '@/types/modules/assignments';
import { getMyAssignments } from '@/services/me/assignments/get';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '@/context/AuthContext';
import { PercentageTag } from '../common';

dayjs.extend(relativeTime);

const { Text, Title } = Typography;
const now = () => dayjs();

type Props = {
  role?: ModuleRole;
  viewLabels?: {
    due: string;
    upcoming: string;
  };
  moduleId?: number;
  limit?: number;
  minimal?: boolean;
};

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
  grade: {
    percentage: number;
    earned: number;
    total: number;
  } | null;
  submissionSummary: {
    submitted: number;
    totalStudents: number;
  } | null;
  readiness: AssignmentReadiness | null;
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

const AssignmentsPanel: React.FC<Props> = ({ role, viewLabels, moduleId, limit, minimal = false }) => {
  const [view, setView] = useState<'due' | 'upcoming'>('due');
  const [items, setItems] = useState<DisplayAssignment[]>([]);
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();
  const { isStudent, isStaff, isTutor, isAssistantLecturer, isLecturer } = useAuth();

  const labels = viewLabels ?? { due: 'Due', upcoming: 'Upcoming' };
  const pageSize = limit ?? 50;

  useEffect(() => {
    let cancelled = false;
    (async () => {
      setLoading(true);
      try {
        const res = await getMyAssignments({ role, module_id: moduleId, page: 1, per_page: pageSize });
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
          grade: a.grade
            ? {
                percentage: a.grade.percentage,
                earned: a.grade.earned,
                total: a.grade.total,
              }
            : null,
          submissionSummary: a.submission_summary
            ? {
                submitted: a.submission_summary.submitted,
                totalStudents: a.submission_summary.total_students,
              }
            : null,
          readiness: a.readiness ?? null,
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
  }, [role, moduleId, pageSize]);

  const filtered = useMemo(() => {
    const n = now();
    if (view === 'due') {
      const dueList = items
        .filter((a) => a.status !== 'archived')
        .filter((a) => !dayjs(a.available_from).isAfter(n))
        .filter((a) => dayjs(a.due_date).isAfter(n))
        .sort((a, b) => dayjs(a.due_date).valueOf() - dayjs(b.due_date).valueOf());
      return limit ? dueList.slice(0, limit) : dueList;
    }
    const upcomingList = items
      .filter((a) => a.status !== 'archived')
      .filter((a) => dayjs(a.available_from).isAfter(n))
      .sort((a, b) => dayjs(a.available_from).valueOf() - dayjs(b.available_from).valueOf());
    return limit ? upcomingList.slice(0, limit) : upcomingList;
  }, [items, view, limit]);

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
              { label: labels.due, value: 'due' },
              { label: labels.upcoming, value: 'upcoming' },
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
                view === 'due'
                  ? labels.due === 'Due'
                    ? 'No assignments currently open.'
                    : `No ${labels.due.toLowerCase()} assignments.`
                  : labels.upcoming === 'Upcoming'
                    ? 'No assignments opening soon.'
                    : `No ${labels.upcoming.toLowerCase()} assignments.`
              }
            />
          ),
        }}
        dataSource={filtered}
        renderItem={(a) => {
          const due = dayjs(a.due_date);
          const opens = dayjs(a.available_from);
          const pct = progressPercent(a);
          const gradePercentage = a.grade ? Math.round(a.grade.percentage) : null;
          const showGrade = gradePercentage !== null && isStudent(a.module.id);
          const showSubmissions =
            a.submissionSummary != null &&
            (isLecturer(a.module.id) ||
              isAssistantLecturer(a.module.id) ||
              isTutor(a.module.id) ||
              isStaff(a.module.id));

          const targetMoment = view === 'upcoming' ? opens : due;
          const targetLabel = view === 'upcoming' ? 'opens' : 'due';
          const dateDisplay = (
            <Text type="secondary" className="inline-flex items-center !text-[12px]">
              {targetLabel} {targetMoment.fromNow()}
              <Tooltip title={targetMoment.format('YYYY-MM-DD HH:mm')}>
                <ClockCircleOutlined className="ml-1" />
              </Tooltip>
            </Text>
          );

          const timing = minimal
            ? null
            : (
                <div className="flex items-center gap-2">
                  <Text type="secondary" className="!text-[12px]">
                    {a.module.code}
                  </Text>
                  <Text type="secondary" className="!text-[12px]">
                    •
                  </Text>
                  {dateDisplay}
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
                  {(minimal || showGrade || (showSubmissions && a.submissionSummary)) && (
                    <div className="flex items-center gap-2 shrink-0 flex-wrap justify-end">
                      {minimal && <span>{dateDisplay}</span>}
                      {showGrade && <PercentageTag value={gradePercentage!} />}
                      {showSubmissions && a.submissionSummary && (
                        <Tag color="blue" className="!m-0">
                          {a.submissionSummary.submitted}/{a.submissionSummary.totalStudents}{' '}
                          submitted
                        </Tag>
                      )}
                    </div>
                  )}
                </div>
                {timing && <div className="flex flex-col gap-0.5">{timing}</div>}
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
