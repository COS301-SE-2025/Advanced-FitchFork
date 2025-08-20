import { useState, useMemo } from 'react';
import { List, Typography, Tooltip, Tag, Empty } from 'antd';
import { ClockCircleOutlined, FileTextOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import type { Submission } from '@/types/modules/assignments/submissions';
import type { Module } from '@/types/modules';
import type { Assignment } from '@/types/modules/assignments';

dayjs.extend(relativeTime);

const { Text, Title } = Typography;

// ---- Mock data ----
const MOCK_MODULES: Record<number, Module> = {
  101: {
    id: 101,
    code: 'COS101',
    year: 2025,
    description: '',
    credits: 16,
    created_at: '',
    updated_at: '',
  },
  344: {
    id: 344,
    code: 'COS344',
    year: 2025,
    description: '',
    credits: 24,
    created_at: '',
    updated_at: '',
  },
};

const MOCK_ASSIGNMENTS: Record<number, Assignment> = {
  701: {
    id: 701,
    module_id: 101,
    name: 'A1: Basics',
    description: '',
    assignment_type: 'assignment' as any,
    available_from: '',
    due_date: '',
    status: 'open' as any,
    created_at: '',
    updated_at: '',
  },
  702: {
    id: 702,
    module_id: 344,
    name: 'Prac 2: Sockets',
    description: '',
    assignment_type: 'assignment' as any,
    available_from: '',
    due_date: '',
    status: 'open' as any,
    created_at: '',
    updated_at: '',
  },
};

const MOCK_SUBMISSIONS: Submission[] = [
  {
    id: 1,
    attempt: 1,
    filename: 'solution.zip',
    hash: 'abc123',
    mark: { earned: 42, total: 50 },
    is_practice: false,
    is_late: false,
    created_at: dayjs().subtract(2, 'hour').toISOString(),
    updated_at: dayjs().subtract(2, 'hour').toISOString(),
    user: { id: 55, username: 'alice', email: '' },
    assignment_id: 701,
  } as any,
  {
    id: 2,
    attempt: 1,
    filename: 'prac2.tar.gz',
    hash: 'def456',
    mark: { earned: 31, total: 50 },
    is_practice: false,
    is_late: true,
    created_at: dayjs().subtract(1, 'day').toISOString(),
    updated_at: dayjs().subtract(1, 'day').toISOString(),
    user: { id: 56, username: 'bob', email: '' },
    assignment_id: 702,
  } as any,
];

const SubmissionsPanel = () => {
  const [submissions] = useState<Submission[]>(MOCK_SUBMISSIONS);

  const visible = useMemo(() => {
    return [...submissions]
      .sort(
        (a, b) =>
          dayjs(b.updated_at || b.created_at).valueOf() -
          dayjs(a.updated_at || a.created_at).valueOf(),
      )
      .slice(0, 10);
  }, [submissions]);

  const calcPercent = (earned?: number, total?: number) => {
    if (typeof earned !== 'number' || typeof total !== 'number' || total === 0) return null;
    return Math.round((earned / total) * 100);
  };

  return (
    <div className="h-full min-h-0 flex flex-col w-full bg-white dark:bg-gray-900 rounded-md border border-gray-200 dark:border-gray-800">
      {/* Header */}
      <div className="px-3 py-2 border-b border-gray-200 dark:border-gray-800">
        <div className="flex items-center gap-2">
          <FileTextOutlined className="text-gray-500" />
          <Title level={5} className="!mb-0">
            Submissions
          </Title>
        </div>
      </div>

      {/* List */}
      <List
        className="flex-1 overflow-y-auto"
        locale={{
          emptyText: (
            <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="No recent submissions." />
          ),
        }}
        dataSource={visible}
        renderItem={(s) => {
          const a = MOCK_ASSIGNMENTS[(s as any).assignment_id];
          const moduleCode = a ? (MOCK_MODULES[a.module_id]?.code ?? `M-${a.module_id}`) : '—';
          const when = dayjs(s.updated_at || s.created_at);
          const percent = calcPercent(s.mark?.earned, s.mark?.total);

          return (
            <List.Item
              className="!px-3 cursor-pointer"
              onClick={() => console.log('Open submission', s.id)}
            >
              <div className="flex flex-col gap-1.5 w-full">
                {/* Title + tags */}
                <div className="flex items-center justify-between gap-2 min-w-0">
                  <Text strong className="truncate">
                    {a?.name ?? 'Unknown Assignment'}
                  </Text>
                  <div className="flex items-center gap-1 shrink-0">
                    {s.is_late && <Tag color="orange">Late</Tag>}
                    {percent !== null && (
                      <Tag color={percent >= 50 ? 'green' : 'red'}>{percent}%</Tag>
                    )}
                  </div>
                </div>

                {/* Meta */}
                <div className="flex flex-wrap items-center gap-x-2 gap-y-1 text-xs text-gray-500">
                  <Text type="secondary" className="!text-[12px]">
                    {moduleCode}
                  </Text>
                  {s.user?.username && (
                    <>
                      <Text type="secondary" className="!text-[12px]">
                        •
                      </Text>
                      <Text type="secondary" className="!text-[12px]">
                        {s.user.username}
                      </Text>
                    </>
                  )}
                  <Text type="secondary" className="!text-[12px]">
                    •
                  </Text>
                  <Text type="secondary" className="inline-flex items-center !text-[12px]">
                    <Tooltip title={when.format('YYYY-MM-DD HH:mm')}>
                      <ClockCircleOutlined className="mr-1" />
                    </Tooltip>
                    {when.fromNow()}
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

export default SubmissionsPanel;
