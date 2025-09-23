import { useCallback, useEffect, useMemo, useState } from 'react';
import { Alert, Empty, List, Typography, Segmented, Button } from 'antd';
import { WarningOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';

import { getMyPlagiarismCases, type PlagiarismCaseItem } from '@/services/me/plagiarism/get';
import type { ModuleRole } from '@/types/modules';
import type { PlagiarismCaseStatus } from '@/types/modules/assignments/plagiarism';
import PlagiarismStatusTag from './PlagiarismStatusTag';

dayjs.extend(relativeTime);

const { Text, Title } = Typography;

type FilterStatus = Extract<PlagiarismCaseStatus, 'review' | 'flagged'>;

export type PlagiarismCasesPanelProps = {
  role?: ModuleRole;
  moduleId?: number;
  perPage?: number;
};

const PlagiarismCasesPanel: React.FC<PlagiarismCasesPanelProps> = ({
  moduleId,
  perPage = 20,
  role,
}) => {
  const [cases, setCases] = useState<PlagiarismCaseItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();
  const [statusFilter, setStatusFilter] = useState<FilterStatus>('review');

  const fetchCases = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await getMyPlagiarismCases({
        module_id: moduleId,
        per_page: perPage,
        page: 1,
        status: statusFilter,
        role,
      });
      if (!res.success) {
        throw new Error(res.message || 'Failed to load plagiarism cases');
      }
      const data = Array.isArray(res.data.cases) ? res.data.cases : [];
      setCases(data);
    } catch (err: any) {
      setError(err?.message ?? 'Failed to load plagiarism cases');
      setCases([]);
    } finally {
      setLoading(false);
    }
  }, [moduleId, perPage, statusFilter]);

  useEffect(() => {
    fetchCases();
  }, [fetchCases]);

  const items = useMemo(() => cases, [cases]);

  return (
    <div className="h-full min-h-0 flex flex-col w-full bg-white dark:bg-gray-900 rounded-md border border-gray-200 dark:border-gray-800">
      <div className="px-3 py-2 border-b border-gray-200 dark:border-gray-800">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <WarningOutlined className="text-gray-500" />
            <Title level={5} className="!mb-0">
              Plagiarism Cases
            </Title>
          </div>
          <Segmented
            size="small"
            value={statusFilter}
            onChange={(value) => setStatusFilter(value as FilterStatus)}
            options={[
              { label: 'Review', value: 'review' },
              { label: 'Flagged', value: 'flagged' },
            ]}
          />
        </div>
      </div>

      {error && (
        <div className="px-3 py-2">
          <Alert
            type="error"
            showIcon
            message="Failed to load plagiarism cases"
            description={error}
          />
        </div>
      )}

      <List
        className="flex-1 overflow-y-auto"
        size="small" // denser list
        split
        loading={loading}
        locale={{
          emptyText: (
            <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="No plagiarism cases found." />
          ),
        }}
        dataSource={items}
        renderItem={(item) => {
          const created = dayjs(item.created_at);

          const handleNavigate = (submissionId: number) => {
            navigate(
              `/modules/${item.module.id}/assignments/${item.assignment.id}/submissions/${submissionId}`,
            );
          };

          const similarityTag = `${Math.round(item.similarity)}%`;
          const linesTag = `${item.lines_matched} lines`;

          return (
            <List.Item className="!px-3 !py-2">
              {/* --- Mobile (no date) --- */}
              <div className="sm:hidden w-full">
                <div className="flex items-start justify-between gap-2 min-w-0">
                  <div className="min-w-0">
                    <div className="text-sm font-medium truncate">
                      <Button
                        type="link"
                        className="align-middle text-primary-600 dark:text-primary-400 !p-0"
                        onClick={() => handleNavigate(item.submission_1.submission_id)}
                      >
                        {item.submission_1.username}
                      </Button>
                      <span className="mx-1 text-gray-400">vs</span>
                      <Button
                        type="link"
                        className="align-middle text-primary-600 dark:text-primary-400 !p-0"
                        onClick={() => handleNavigate(item.submission_2.submission_id)}
                      >
                        {item.submission_2.username}
                      </Button>
                    </div>

                    {/* line 2: module•assignment (NO date on mobile) */}
                    <div className="text-xs text-gray-500 dark:text-gray-400 truncate">
                      {item.module.code}
                      <span className="mx-1">•</span>
                      <span className="truncate inline-block max-w-[80%] align-bottom">
                        {item.assignment.name}
                      </span>
                    </div>
                  </div>

                  {/* right-side chips: Status + Similarity only */}
                  <div className="flex items-center gap-1 shrink-0">
                    <PlagiarismStatusTag status={item.status as PlagiarismCaseStatus} />
                    <span className="text-xs font-semibold px-1.5 py-0.5 rounded-md bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-300">
                      {similarityTag}
                    </span>
                  </div>
                </div>
              </div>

              {/* --- Desktop (unchanged: includes date + lines) --- */}
              <div className="hidden sm:block w-full">
                <div className="flex items-start justify-between gap-2 min-w-0">
                  <div className="flex flex-col min-w-0">
                    <Text strong className="truncate">
                      <Button
                        type="link"
                        className="text-primary-600 dark:text-primary-400 font-medium hover:underline !p-0"
                        onClick={() => handleNavigate(item.submission_1.submission_id)}
                      >
                        {item.submission_1.username}
                      </Button>
                      <span className="mx-1 text-gray-400">vs</span>
                      <Button
                        type="link"
                        className="text-primary-600 dark:text-primary-400 font-medium hover:underline !p-0"
                        onClick={() => handleNavigate(item.submission_2.submission_id)}
                      >
                        {item.submission_2.username}
                      </Button>
                    </Text>
                    <div className="flex items-center gap-1 text-xs text-gray-500 dark:text-gray-400">
                      <span className="truncate">
                        {item.module.code}
                        <span className="mx-1">•</span>
                        {item.assignment.name}
                      </span>
                      <span>•</span>
                      <span>{created.fromNow()}</span>
                    </div>
                  </div>
                  <div className="flex flex-wrap items-center justify-end gap-1 shrink-0">
                    <PlagiarismStatusTag status={item.status as PlagiarismCaseStatus} />
                    <span className="text-xs font-medium text-blue-600 dark:text-blue-400">
                      {similarityTag}
                    </span>
                    <span className="text-xs font-medium text-purple-600 dark:text-purple-300">
                      {linesTag}
                    </span>
                  </div>
                </div>
              </div>
            </List.Item>
          );
        }}
      />
    </div>
  );
};

export default PlagiarismCasesPanel;
