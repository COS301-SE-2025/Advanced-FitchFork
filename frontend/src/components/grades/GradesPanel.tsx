import { useCallback, useEffect, useMemo, useState } from 'react';
import { List, Typography, Empty, Tooltip, Alert } from 'antd';
import { ClockCircleOutlined, FileDoneOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import { useUI } from '@/context/UIContext';
import { useNavigate } from 'react-router-dom';
import { getMyGrades, type MyGradeItem } from '@/services/me/grades/get';
import ScoreTag from './ScoreTag';

dayjs.extend(relativeTime);

const { Text, Title } = Typography;
const GradesPanel = () => {
  const { isSm } = useUI();
  const navigate = useNavigate();

  const [grades, setGrades] = useState<MyGradeItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchGrades = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await getMyGrades({ page: 1, per_page: 50 });
      if (!res.success) throw new Error(res.message || 'Failed to load grades');

      const rows = Array.isArray(res.data?.grades) ? res.data?.grades : [];
      setGrades(rows ?? []);
    } catch (err: any) {
      setError(err?.message ?? 'Failed to load grades');
      setGrades([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void fetchGrades();
  }, [fetchGrades]);

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

      {error && (
        <div className="px-3 py-2">
          <Alert type="error" showIcon message="Failed to load grades" description={error} />
        </div>
      )}

      {/* List */}
      <List
        className="flex-1 overflow-y-auto"
        loading={loading}
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
          const moduleId = g.module?.id;
          const assignmentId = g.assignment?.id;
          const moduleCode = g.module?.code ?? (moduleId ? `M-${moduleId}` : '—');
          const when = dayjs(g.updated_at || g.created_at);
          const percentage = typeof g.percentage === 'number' ? g.percentage : 0;
          const roundedPercentage = Math.round(percentage * 10) / 10;
          const earned = g.score?.earned;
          const total = g.score?.total;
          const canNavigate = moduleId != null && assignmentId != null;

          return (
            <List.Item
              className="!px-3 cursor-pointer"
              onClick={canNavigate ? () => navigate(`/modules/${moduleId}/assignments/${assignmentId}`) : undefined}
            >
              <div className="flex flex-col gap-1.5 w-full">
                {/* Title + score */}
                <div className="flex items-center justify-between gap-2 min-w-0">
                  <Text strong className="truncate">
                    {g.assignment?.title ?? 'Unknown assignment'}
                  </Text>
                  <ScoreTag score={roundedPercentage} />
                </div>

                {/* Meta */}
                <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
                  <Text type="secondary" className="!text-[12px]">
                    {moduleCode}
                  </Text>
                  {Number.isFinite(earned) && Number.isFinite(total) && (
                    <>
                      <Text type="secondary" className="!text-[12px]">
                        •
                      </Text>
                      <Text type="secondary" className="!text-[12px]">
                        {earned}/{total} marks
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
