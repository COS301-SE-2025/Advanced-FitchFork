import { useMemo, useState } from 'react';
import { List, Typography, Empty, Tooltip } from 'antd';
import { ClockCircleOutlined, FileDoneOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import type { Assignment } from '@/types/modules/assignments';
import type { Module } from '@/types/modules';
import type { Grade } from '@/types/modules/grades';
import { useUI } from '@/context/UIContext';
import ScoreTag from './ScoreTag';

dayjs.extend(relativeTime);

const { Text, Title } = Typography;

// ---------- Mock data ----------
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
};

const mkA = (id: number, module_id: number, name: string): Assignment =>
  ({
    id,
    module_id,
    name,
    description: '',
    assignment_type: 'assignment',
    available_from: dayjs().subtract(14, 'day').toISOString(),
    due_date: dayjs().add(3, 'day').toISOString(),
    status: 'open',
    created_at: '',
    updated_at: '',
  }) as any;

const MOCK_ASSIGNMENTS: Record<number, Assignment> = Object.fromEntries(
  [
    mkA(701, 101, 'A1: Basics'),
    mkA(702, 101, 'A2: Data Structures'),
    mkA(703, 344, 'Prac 2: Sockets'),
    mkA(704, 344, 'Prac 3: Threads'),
  ].map((a) => [a.id, a]),
);

const CURRENT_USER_ID = 55;

const MOCK_GRADES: Grade[] = [
  {
    id: 1,
    assignment_id: 701,
    student_id: CURRENT_USER_ID,
    submission_id: 201,
    score: 78,
    created_at: dayjs().subtract(2, 'hour').toISOString(),
    updated_at: dayjs().subtract(2, 'hour').toISOString(),
  },
  {
    id: 2,
    assignment_id: 703,
    student_id: CURRENT_USER_ID,
    submission_id: 202,
    score: 44,
    created_at: dayjs().subtract(3, 'day').toISOString(),
    updated_at: dayjs().subtract(3, 'day').toISOString(),
  },
  {
    id: 3,
    assignment_id: 704,
    student_id: CURRENT_USER_ID,
    submission_id: 203,
    score: 19,
    created_at: dayjs().subtract(20, 'day').toISOString(),
    updated_at: dayjs().subtract(20, 'day').toISOString(),
  },
];

// ---------- Component ----------
const GradesPanel = () => {
  const { isSm } = useUI();

  const [grades] = useState<Grade[]>(MOCK_GRADES.filter((g) => g.student_id === CURRENT_USER_ID));

  // Show grades from the last 30 days only
  const visible = useMemo(() => {
    const cutoff = dayjs().subtract(30, 'day');
    return grades
      .filter((g) => {
        const ts = dayjs(g.updated_at || g.created_at);
        return ts.isValid() && ts.isAfter(cutoff);
      })
      .sort(
        (a, b) =>
          dayjs(b.updated_at || b.created_at).valueOf() -
          dayjs(a.updated_at || a.created_at).valueOf(),
      );
  }, [grades]);

  return (
    <div className="h-full min-h-0 flex flex-col w-full bg-white dark:bg-gray-900 rounded-md border border-gray-200 dark:border-gray-800">
      {/* Header */}
      <div className="px-3 py-2 border-b border-gray-200 dark:border-gray-800">
        <div className="flex items-center gap-2">
          <FileDoneOutlined className="text-gray-500" />
          <Title level={isSm ? 5 : 5} className="!mb-0">
            Grades
          </Title>
        </div>
      </div>

      {/* List */}
      <List
        className="flex-1 overflow-y-auto"
        locale={{
          emptyText: (
            <Empty
              image={Empty.PRESENTED_IMAGE_SIMPLE}
              description="No grades in the last 30 days."
            />
          ),
        }}
        dataSource={visible}
        renderItem={(g) => {
          const a = MOCK_ASSIGNMENTS[g.assignment_id];
          const moduleCode = a ? (MOCK_MODULES[a.module_id]?.code ?? `M-${a.module_id}`) : '—';
          const when = dayjs(g.updated_at || g.created_at);

          return (
            <List.Item
              className="!px-3 cursor-pointer"
              onClick={() => console.log('Open assignment', { id: a?.id, name: a?.name })}
            >
              <div className="flex flex-col gap-1.5 w-full">
                {/* Title + score */}
                <div className="flex items-center justify-between gap-2 min-w-0">
                  <Text strong className="truncate">
                    {a?.name ?? 'Unknown Assignment'}
                  </Text>
                  <ScoreTag score={g.score} />
                </div>

                {/* Meta */}
                <div className="flex items-center gap-2">
                  <Text type="secondary" className="!text-[12px]">
                    {moduleCode}
                  </Text>
                  <Text type="secondary" className="!text-[12px]">
                    •
                  </Text>
                  <Text type="secondary" className="inline-flex items-center !text-[12px]">
                    <Tooltip title={when.format('YYYY-MM-DD HH:mm')}>
                      <ClockCircleOutlined className="mr-1" />
                    </Tooltip>
                    graded {when.fromNow()}
                  </Text>
                </div>
              </div>
            </List.Item>
          );
        }}
      />
    </div>
  );
};

export default GradesPanel;
