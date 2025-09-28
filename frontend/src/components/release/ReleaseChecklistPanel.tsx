import { useCallback, useEffect, useMemo, useState } from 'react';
import { Alert, Empty, List, Tag, Tooltip, Typography } from 'antd';
import { FileDoneOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import { useNavigate } from 'react-router-dom';

import { getMyAssignments, type MyAssignmentItem } from '@/services/me/assignments/get';
import type { AssignmentReadiness, AssignmentStatus } from '@/types/modules/assignments';
import type { ModuleRole } from '@/types/modules';

dayjs.extend(relativeTime);

const { Title, Text } = Typography;

type Props = {
  role: ModuleRole;
  status?: AssignmentStatus;
};

type ReadinessFlagKey =
  | 'config_present'
  | 'tasks_present'
  | 'main_present'
  | 'interpreter_present'
  | 'memo_present'
  | 'makefile_present'
  | 'memo_output_present'
  | 'mark_allocator_present';

const REQUIREMENTS: Array<{
  key: ReadinessFlagKey;
  label: string;
}> = [
  { key: 'config_present', label: 'Config file' },
  { key: 'tasks_present', label: 'Tasks' },
  { key: 'main_present', label: 'Main file' },
  { key: 'interpreter_present', label: 'Interpreter' },
  { key: 'makefile_present', label: 'Makefile' },
  { key: 'memo_present', label: 'Memo file' },
  { key: 'memo_output_present', label: 'Memo output' },
  { key: 'mark_allocator_present', label: 'Mark allocator' },
];

type ReleaseChecklistItem = {
  assignment: MyAssignmentItem;
  readiness: AssignmentReadiness | null;
};

function summariseReadiness(readiness: AssignmentReadiness | null) {
  const submissionMode = readiness?.submission_mode ?? 'manual';

  const relevant = REQUIREMENTS.filter((req) => {
    if (req.key === 'main_present') {
      return submissionMode === 'manual';
    }
    if (req.key === 'interpreter_present') {
      return submissionMode !== 'manual';
    }
    return true;
  });

  const completed = relevant.filter((req) => readiness?.[req.key] === true).length;
  const missingLabels = relevant
    .filter((req) => readiness?.[req.key] !== true)
    .map((req) => req.label);

  return {
    total: relevant.length,
    completed,
    missingLabels,
    ready: readiness?.is_ready === true,
  };
}

const ReleaseChecklistPanel: React.FC<Props> = ({ role, status }) => {
  const navigate = useNavigate();
  const [items, setItems] = useState<ReleaseChecklistItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchAssignments = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await getMyAssignments({
        role,
        status,
        page: 1,
        per_page: 20,
      });

      if (!res.success) {
        throw new Error(res.message || 'Failed to load assignments');
      }

      const assignments = Array.isArray(res.data?.assignments) ? res.data.assignments : [];

      const mapped = assignments.map((assignment) => ({
        assignment,
        readiness: assignment.readiness ?? null,
      }));

      setItems(mapped);
    } catch (err: any) {
      setError(err?.message ?? 'Failed to load assignments');
      setItems([]);
    } finally {
      setLoading(false);
    }
  }, [role, status]);

  useEffect(() => {
    void fetchAssignments();
  }, [fetchAssignments]);

  const ordered = useMemo(() => {
    return [...items].sort((a, b) => {
      const aTs = dayjs(a.assignment.updated_at || a.assignment.created_at).valueOf();
      const bTs = dayjs(b.assignment.updated_at || b.assignment.created_at).valueOf();
      return bTs - aTs;
    });
  }, [items]);

  return (
    <div className="h-full min-h-0 flex flex-col w-full bg-white dark:bg-gray-900 rounded-md border border-gray-200 dark:border-gray-800">
      <div className="px-3 py-2 border-b border-gray-200 dark:border-gray-800">
        <div className="flex items-center gap-2">
          <FileDoneOutlined className="text-gray-500" />
          <Title level={5} className="!mb-0">
            Release Checklist
          </Title>
        </div>
      </div>

      {error && (
        <div className="px-3 py-2">
          <Alert
            type="error"
            showIcon
            message="Failed to load release checklist"
            description={error}
          />
        </div>
      )}

      <List
        className="flex-1 overflow-y-auto"
        loading={loading}
        locale={{
          emptyText: (
            <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="No assignments found." />
          ),
        }}
        dataSource={ordered}
        renderItem={({ assignment, readiness }) => {
          const moduleId = assignment.module?.id ?? assignment.module_id;
          const moduleCode =
            assignment.module?.code ?? `M-${assignment.module?.id ?? assignment.module_id}`;
          const summary = summariseReadiness(readiness);
          const updated = dayjs(assignment.updated_at || assignment.created_at);
          const missingPreview = summary.missingLabels.slice(0, 3);
          const remaining = summary.missingLabels.length - missingPreview.length;
          const canNavigate = moduleId != null;

          return (
            <List.Item
              className="!px-3 cursor-pointer"
              onClick={
                canNavigate
                  ? () => navigate(`/modules/${moduleId}/assignments/${assignment.id}`)
                  : undefined
              }
            >
              <div className="flex flex-col gap-1.5 w-full">
                <div className="flex items-center justify-between gap-2 min-w-0">
                  <Text strong className="truncate">
                    {assignment.name}
                  </Text>
                  <Tag color={summary.ready ? 'green' : 'orange'}>
                    {summary.ready ? 'Ready to release' : 'Needs setup'}
                  </Tag>
                </div>

                <div className="flex flex-wrap items-center gap-x-2 gap-y-1">
                  <Text type="secondary" className="!text-[12px]">
                    {moduleCode}
                  </Text>
                  <Text type="secondary" className="!text-[12px]">
                    •
                  </Text>
                  <Text type="secondary" className="!text-[12px]">
                    {summary.completed}/{summary.total} checklist items complete
                  </Text>
                  <Text type="secondary" className="!text-[12px]">
                    •
                  </Text>
                  <Text type="secondary" className="inline-flex items-center !text-[12px]">
                    <Tooltip title={updated.format('YYYY-MM-DD HH:mm')}>
                      Updated {updated.fromNow()}
                    </Tooltip>
                  </Text>
                </div>

                {summary.missingLabels.length > 0 && (
                  <Text type="secondary" className="!text-[12px]">
                    Missing: {missingPreview.join(', ')}
                    {remaining > 0 ? ` +${remaining} more` : ''}
                  </Text>
                )}
              </div>
            </List.Item>
          );
        }}
      />
    </div>
  );
};

export default ReleaseChecklistPanel;
