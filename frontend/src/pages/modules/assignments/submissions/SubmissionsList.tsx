import { Tag, Typography } from 'antd';
import { DeleteOutlined, RedoOutlined, ReloadOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useEffect, useRef, useState } from 'react';
import dayjs from 'dayjs';

import {
  EntityList,
  type EntityListHandle,
  type EntityColumnType,
  type EntityAction,
  type EntityListProps,
} from '@/components/EntityList';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { useAuth } from '@/context/AuthContext';
import { getSubmissions } from '@/services/modules/assignments/submissions';
import EventBus from '@/utils/EventBus';

import type { Submission } from '@/types/modules/assignments/submissions';
import SubmissionCard from '@/components/submissions/SubmissionCard';
import { message } from '@/utils/message';
import {
  remarkSubmissions,
  resubmitSubmissions,
  submitAssignment,
} from '@/services/modules/assignments/submissions/post';
import { useViewSlot } from '@/context/ViewSlotContext';
import {
  SubmissionListItem,
  SubmissionsEmptyState,
  SubmitAssignmentModal,
} from '@/components/submissions';

const getMarkColor = (mark: number): string => {
  if (mark >= 75) return 'green';
  if (mark >= 50) return 'orange';
  return 'red';
};

type StudentSubmission = Submission & {
  status: 'Pending' | 'Graded';
  path: string;
  percentageMark?: number;
};

export default function SubmissionsList() {
  const navigate = useNavigate();
  const module = useModule();
  const { setValue } = useViewSlot();
  const { assignment, refreshAssignment } = useAssignment();
  const auth = useAuth();

  const [modalOpen, setModalOpen] = useState(false);
  const [loading, setLoading] = useState(false);

  const isAssignmentOpen = assignment.status === 'open';
  const isStudent = auth.isStudent(module.id);

  const entityListRef = useRef<EntityListHandle>(null);

  useEffect(() => {
    setValue(
      <Typography.Text
        className="text-base font-medium text-gray-900 dark:text-gray-100 truncate"
        title={'Submissions'}
      >
        Submissions
      </Typography.Text>,
    );
  }, []);

  useEffect(() => {
    const listener = () => {
      entityListRef.current?.refresh();
    };
    EventBus.on('submission:updated', listener);

    return () => {
      EventBus.off('submission:updated', listener);
    };
  }, []);

  const handleOpenSubmit = () => {
    setModalOpen(true);
  };

  const fetchItems = async ({
    page,
    per_page,
    query,
    filters,
    sort,
  }: {
    page: number;
    per_page: number;
    query?: string;
    filters: Record<string, string[]>;
    sort: { field: string; order: 'ascend' | 'descend' }[];
  }) => {
    if (!module.id || !assignment.id) {
      return { items: [], total: 0 };
    }

    const res = await getSubmissions(module.id, assignment.id, {
      page,
      per_page,
      query,
      sort,
      username: filters['user.username']?.[0],
      status: filters['status']?.[0],
    });

    const { submissions, total } = res.data;

    const items: StudentSubmission[] = submissions.map(
      (s): StudentSubmission => ({
        ...s,
        status: s.mark ? 'Graded' : 'Pending',
        percentageMark:
          s.mark && typeof s.mark === 'object' && 'earned' in s.mark
            ? Math.round(((s.mark as any).earned / (s.mark as any).total) * 100)
            : undefined,
        path: `/api/modules/${module.id}/assignments/${assignment.id}/submissions/${s.id}/file`,
      }),
    );

    return { items, total };
  };

  const handleSubmitAssignment = async (file: File, isPractice: boolean) => {
    setModalOpen(false);
    setLoading(true);
    const hide = message.loading('Submitting assignment...');
    try {
      await submitAssignment(module.id, assignment.id, file, isPractice);
      await refreshAssignment();
      message.success('Submission successful');
      EventBus.emit('submission:updated');
      entityListRef.current?.refresh();
    } catch {
      message.error('Submission failed');
    } finally {
      hide();
      setLoading(false);
    }
  };

  const columns: EntityColumnType<StudentSubmission>[] = [
    { title: 'ID', dataIndex: 'id', key: 'id', defaultHidden: true },
    {
      title: 'Username',
      dataIndex: ['user', 'username'],
      key: 'user.username',
      sorter: { multiple: 1 },
    },
    {
      title: 'Attempt',
      dataIndex: 'attempt',
      key: 'attempt',
      sorter: { multiple: 2 },
      render: (v) => <Tag color="blue">#{v}</Tag>,
    },
    { title: 'Filename', dataIndex: 'filename', key: 'filename', defaultHidden: true },
    {
      title: 'Is Practice',
      dataIndex: 'is_practice',
      key: 'is_practice',
      defaultHidden: true,
      render: (v) => (v ? <Tag color="gold">Yes</Tag> : <Tag>No</Tag>),
    },
    {
      title: 'Mark (%)',
      key: 'percentageMark',
      sorter: { multiple: 3 },
      render: (_, r) =>
        r.status === 'Graded' && typeof r.percentageMark === 'number' ? (
          <Tag color={getMarkColor(r.percentageMark)}>{r.percentageMark}%</Tag>
        ) : (
          <Tag color="default">Not marked</Tag>
        ),
    },
    {
      title: 'Is Late',
      dataIndex: 'is_late',
      key: 'is_late',
      defaultHidden: true,
      render: (v) => (v ? <Tag color="red">Yes</Tag> : <Tag>On Time</Tag>),
    },
    {
      title: 'Created At',
      dataIndex: 'created_at',
      key: 'created_at',
      sorter: { multiple: 4 },
      defaultHidden: true,
      render: (v) => dayjs(v).format('YYYY-MM-DD HH:mm'),
    },
    {
      title: 'Updated At',
      dataIndex: 'updated_at',
      key: 'updated_at',
      defaultHidden: true,
      render: (v) => dayjs(v).format('YYYY-MM-DD HH:mm'),
    },
  ];

  const canManageSubmissions = auth.isLecturer(module.id) || auth.isAdmin;

  const actions: EntityListProps<StudentSubmission>['actions'] = canManageSubmissions
    ? {
        entity: (entity: StudentSubmission): EntityAction<StudentSubmission>[] => [
          {
            key: 'delete',
            label: 'Delete',
            icon: <DeleteOutlined />,
            handler: ({ refresh }) => {
              message.success(`Deleted submission ${entity.id}`);
              refresh();
            },
          },
          {
            key: 'remark',
            label: 'Re-mark',
            icon: <ReloadOutlined />,
            handler: async ({ refresh }) => {
              try {
                const res = await remarkSubmissions(module.id, assignment.id, {
                  submission_ids: [entity.id],
                });
                if (res.success) {
                  message.success(res.message);
                } else {
                  message.error(res.message);
                }
                EventBus.emit('submission:updated');
                refresh();
              } catch (err) {
                console.error(err);
                message.error(`Failed to re-mark submission ${entity.id}`);
              }
            },
          },
          {
            key: 'resubmit',
            label: 'Resubmit',
            icon: <RedoOutlined />,
            handler: async ({ refresh }) => {
              try {
                const res = await resubmitSubmissions(module.id, assignment.id, {
                  submission_ids: [entity.id],
                });
                if (res.success) {
                  message.success(res.message || `Resubmitted 1/1 submissions`);
                } else {
                  message.error(res.message || `Failed to resubmit submission ${entity.id}`);
                }
                EventBus.emit('submission:updated');
                refresh();
              } catch (err) {
                console.error(err);
                message.error(`Failed to resubmit submission ${entity.id}`);
              }
            },
          },
        ],
        bulk: [
          {
            key: 'bulk-delete',
            label: 'Bulk Delete',
            icon: <DeleteOutlined />,
            handler: ({ selected, refresh }) => {
              message.success(`Deleted ${selected?.length || 0} submissions`);
              refresh();
            },
          },
          {
            key: 'bulk-remark',
            label: 'Bulk Re-mark',
            icon: <ReloadOutlined />,
            handler: async ({ selected, refresh }) => {
              const ids = selected as number[];
              if (ids.length === 0) return;

              try {
                const res = await remarkSubmissions(module.id, assignment.id, {
                  submission_ids: ids,
                });

                if (res.success) {
                  message.success(res.message);
                } else {
                  message.error(res.message);
                }
                EventBus.emit('submission:updated');
                refresh();
              } catch (err) {
                console.error(err);
                message.error(`Failed to re-mark some submissions`);
              }
            },
          },
          {
            key: 'bulk-resubmit',
            label: 'Bulk Resubmit',
            icon: <RedoOutlined />,
            handler: async ({ selected, refresh }) => {
              const ids = selected as number[];
              if (!ids?.length) return;
              try {
                const res = await resubmitSubmissions(module.id, assignment.id, {
                  submission_ids: ids,
                });
                if (res.success) {
                  message.success(res.message || `Resubmitted ${ids.length} submission(s)`);
                } else {
                  message.error(res.message || 'Failed to resubmit some submissions');
                }
                EventBus.emit('submission:updated');
                refresh();
              } catch (err) {
                console.error(err);
                message.error('Failed to resubmit some submissions');
              }
            },
          },
        ],
        control: [
          {
            key: 'remark-all',
            label: 'Re-mark All',
            icon: <ReloadOutlined />,
            confirm: true,
            handler: async ({ refresh }) => {
              try {
                const res = await remarkSubmissions(module.id, assignment.id, { all: true });

                if (res.success) {
                  message.success(res.message);
                } else {
                  message.error(res.message);
                }

                EventBus.emit('submission:updated');
                refresh();
              } catch (err) {
                console.error(err);
                message.error(`Failed to re-mark all submissions`);
              }
            },
          },
          {
            key: 'resubmit-all',
            label: 'Resubmit All',
            icon: <RedoOutlined />,
            confirm: true,
            handler: async ({ refresh }) => {
              try {
                const res = await resubmitSubmissions(module.id, assignment.id, { all: true });
                if (res.success) {
                  message.success(res.message || 'Resubmitted all submissions');
                } else {
                  message.error(res.message || 'Failed to resubmit all submissions');
                }
                EventBus.emit('submission:updated');
                refresh();
              } catch (err) {
                console.error(err);
                message.error('Failed to resubmit all submissions');
              }
            },
          },
        ],
      }
    : undefined;

  return (
    <>
      <EntityList<StudentSubmission>
        ref={entityListRef}
        name="Submissions"
        listMode={auth.isStudent(module.id)}
        showControlBar={!isStudent}
        fetchItems={fetchItems}
        columns={columns}
        getRowKey={(item) => item.id}
        onRowClick={(item) =>
          navigate(`/modules/${module.id}/assignments/${assignment.id}/submissions/${item.id}`)
        }
        columnToggleEnabled
        actions={actions}
        renderGridItem={(item, itemActions) => (
          <SubmissionCard
            submission={item}
            actions={itemActions}
            onClick={(s) =>
              navigate(`/modules/${module.id}/assignments/${assignment.id}/submissions/${s.id}`)
            }
          />
        )}
        renderListItem={(submission) => (
          <SubmissionListItem
            submission={submission}
            onClick={(s) =>
              navigate(`/modules/${module.id}/assignments/${assignment.id}/submissions/${s.id}`)
            }
          />
        )}
        emptyNoEntities={
          <SubmissionsEmptyState
            assignmentName={assignment.name}
            isAssignmentOpen={isAssignmentOpen}
            onSubmit={isAssignmentOpen ? handleOpenSubmit : undefined}
            onRefresh={() => entityListRef.current?.refresh()}
          />
        }
      />

      <SubmitAssignmentModal
        open={modalOpen}
        onClose={() => setModalOpen(false)}
        onSubmit={handleSubmitAssignment}
        loading={loading}
        title="Submit Assignment"
        accept=".zip,.tar,.gz,.tgz"
        maxSizeMB={50}
        defaultIsPractice={false}
      />
    </>
  );
}
