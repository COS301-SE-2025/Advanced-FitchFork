import { Switch, Tag, Typography } from 'antd';
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
import { getSubmissions, setSubmissionIgnored } from '@/services/modules/assignments/submissions';
import EventBus from '@/utils/EventBus';

import type { Submission } from '@/types/modules/assignments/submissions';
import SubmissionCard from '@/components/submissions/SubmissionCard';
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
import {
  bulkDeleteSubmissions,
  deleteSubmission,
} from '@/services/modules/assignments/submissions/delete';
import useApp from 'antd/es/app/useApp';
import SubmissionStatistics from './SubmissionStatistics';

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
  const { modal, message } = useApp();

  const { assignment, policy, assignmentStats, refreshAssignment, refreshAssignmentStats } =
    useAssignment();
  const auth = useAuth();
  const hasStats =
    auth.isLecturer(module.id) || auth.isAssistantLecturer(module.id) || auth.isAdmin;
  const canToggleIgnored =
    auth.isLecturer(module.id) || auth.isAssistantLecturer(module.id) || auth.isAdmin;

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
      title: 'Ignored',
      dataIndex: 'ignored',
      key: 'ignored',
      // show to staff OR admin; students keep it hidden by default
      defaultHidden: !(auth.isStaff(module.id) || auth.isAdmin),
      render: (_: boolean, record: StudentSubmission) => {
        if (canToggleIgnored) {
          return (
            <Switch
              size="small"
              checked={record.ignored}
              onClick={(nextChecked, e) => {
                e?.preventDefault();
                e?.stopPropagation();

                const id = record.id;
                // optimistic update
                entityListRef.current?.updateRow(id, { ignored: nextChecked });

                (async () => {
                  try {
                    const res = await setSubmissionIgnored(
                      module.id,
                      assignment.id,
                      id,
                      nextChecked,
                    );
                    if (!res.success) {
                      // rollback
                      entityListRef.current?.updateRow(id, { ignored: !nextChecked });
                      message.error(res.message || 'Failed to update ignored flag');
                    } else {
                      await refreshAssignmentStats();
                    }
                  } catch (err) {
                    entityListRef.current?.updateRow(id, { ignored: !nextChecked });
                    console.error(err);
                    message.error('Failed to update ignored flag');
                  }
                })();
              }}
            />
          );
        }

        // read-only for non-toggle roles (e.g., Tutor)
        return record.ignored ? <Tag color="red">Yes</Tag> : <Tag>No</Tag>;
      },
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
            confirm: true,
            handler: async () => {
              try {
                const res = await deleteSubmission(module.id, assignment.id, entity.id);
                if (res.success) {
                  message.success(res.message || `Deleted submission ${entity.id}`);
                  entityListRef.current?.removeRows([entity.id]);
                } else {
                  message.error(res.message || `Failed to delete submission ${entity.id}`);
                }
              } catch (err) {
                console.error(err);
                message.error(`Failed to delete submission ${entity.id}`);
              }
            },
          },

          {
            key: 'remark',
            label: 'Re-mark',
            icon: <ReloadOutlined />,
            handler: async () => {
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
            handler: async () => {
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
            handler: async ({ selected, refresh }) => {
              const ids = (selected as number[]) ?? [];
              if (!ids.length) {
                message.warning('No submissions selected');
                return;
              }

              modal.confirm({
                title: `Delete ${ids.length} submission${ids.length === 1 ? '' : 's'}?`,
                icon: null, // no yellow warning icon
                centered: true,
                okText: `Delete ${ids.length}`,
                cancelText: 'Cancel',
                okButtonProps: { danger: true },
                content: (
                  <div style={{ marginTop: 8 }}>
                    <Typography.Paragraph>
                      You&apos;re about to <b>delete</b> <Tag>{ids.length}</Tag>
                      submission
                      {ids.length === 1 ? '' : 's'}.
                    </Typography.Paragraph>
                    <Typography.Paragraph type="danger" style={{ marginBottom: 0 }}>
                      This cannot be undone.
                    </Typography.Paragraph>
                  </div>
                ),
                onOk: async () => {
                  try {
                    const res = await bulkDeleteSubmissions(module.id, assignment.id, ids);
                    if (res.success) {
                      const { deleted, failed } = res.data || {};
                      const failCount = failed?.length ?? 0;
                      message.success(
                        res.message || `Deleted ${deleted}/${ids.length} submissions`,
                      );
                      if (failCount > 0) {
                        console.warn('Bulk delete failures:', failed);
                      }
                      EventBus.emit('submission:updated');
                      refresh();
                      entityListRef.current?.clearSelection();
                    } else {
                      message.error(res.message || 'Bulk delete failed');
                    }
                  } catch (err) {
                    console.error(err);
                    message.error('Bulk delete failed');
                  }
                },
              });
            },
          },
          {
            key: 'bulk-remark',
            label: 'Bulk Re-mark',
            icon: <ReloadOutlined />,
            handler: async ({ selected, refresh }) => {
              const ids = (selected as number[]) ?? [];
              if (!ids.length) {
                message.warning('No submissions selected');
                return;
              }

              modal.confirm({
                title: `Re-mark ${ids.length} submission${ids.length === 1 ? '' : 's'}?`,
                icon: null,
                centered: true,
                okText: `Re-mark ${ids.length}`,
                cancelText: 'Cancel',
                content: (
                  <Typography.Paragraph type="secondary">
                    This will queue re-marking for the selected submission
                    {ids.length === 1 ? '' : 's'}.
                  </Typography.Paragraph>
                ),
                onOk: async () => {
                  try {
                    const res = await remarkSubmissions(module.id, assignment.id, {
                      submission_ids: ids,
                    });
                    if (res.success) {
                      message.success(
                        res.message || `Queued re-mark for ${ids.length} submission(s)`,
                      );
                    } else {
                      message.error(res.message || 'Failed to re-mark some submissions');
                    }
                    EventBus.emit('submission:updated');
                    refresh();
                  } catch (err) {
                    console.error(err);
                    message.error('Failed to re-mark some submissions');
                  }
                },
              });
            },
          },
          {
            key: 'bulk-resubmit',
            label: 'Bulk Resubmit',
            icon: <RedoOutlined />,
            handler: async ({ selected, refresh }) => {
              const ids = (selected as number[]) ?? [];
              if (!ids.length) {
                message.warning('No submissions selected');
                return;
              }

              modal.confirm({
                title: `Re-run ${ids.length} submission${ids.length === 1 ? '' : 's'}?`,
                icon: null,
                centered: true,
                okText: `Re-run ${ids.length}`,
                cancelText: 'Cancel',
                content: (
                  <Typography.Paragraph type="secondary">
                    This will re-run the selected submission{ids.length === 1 ? '' : 's'}.
                  </Typography.Paragraph>
                ),
                onOk: async () => {
                  try {
                    const res = await resubmitSubmissions(module.id, assignment.id, {
                      submission_ids: ids,
                    });
                    if (res.success) {
                      message.success(res.message || `Re-ran ${ids.length} submission(s)`);
                    } else {
                      message.error(res.message || 'Failed to re-run some submissions');
                    }
                    EventBus.emit('submission:updated');
                    refresh();
                  } catch (err) {
                    console.error(err);
                    message.error('Failed to re-run some submissions');
                  }
                },
              });
            },
          },
        ],
        control: [
          {
            key: 'remark-all',
            label: 'Re-mark All',
            icon: <ReloadOutlined />,
            handler: async ({ refresh }) => {
              modal.confirm({
                title: 'Re-mark all submissions?',
                icon: null,
                centered: true,
                okText: 'Re-mark All',
                cancelText: 'Cancel',
                content: (
                  <Typography.Paragraph type="secondary">
                    This will queue re-marking for <b>all</b> submissions in this assignment.
                  </Typography.Paragraph>
                ),
                onOk: async () => {
                  try {
                    const res = await remarkSubmissions(module.id, assignment.id, { all: true });
                    if (res.success) {
                      message.success(res.message || 'All submissions queued for re-mark');
                    } else {
                      message.error(res.message || 'Failed to re-mark all submissions');
                    }
                    EventBus.emit('submission:updated');
                    refresh();
                  } catch (err) {
                    console.error(err);
                    message.error('Failed to re-mark all submissions');
                  }
                },
              });
            },
          },
          {
            key: 'resubmit-all',
            label: 'Resubmit All',
            icon: <RedoOutlined />,
            handler: async ({ refresh }) => {
              modal.confirm({
                title: 'Re-run all submissions?',
                icon: null,
                centered: true,
                okText: 'Re-run All',
                cancelText: 'Cancel',
                content: (
                  <Typography.Paragraph type="secondary">
                    This will re-run <b>all</b> submissions in this assignment.
                  </Typography.Paragraph>
                ),
                onOk: async () => {
                  try {
                    const res = await resubmitSubmissions(module.id, assignment.id, { all: true });
                    if (res.success) {
                      message.success(res.message || 'All submissions re-ran successfully');
                    } else {
                      message.error(res.message || 'Failed to re-run all submissions');
                    }
                    EventBus.emit('submission:updated');
                    refresh();
                  } catch (err) {
                    console.error(err);
                    message.error('Failed to re-run all submissions');
                  }
                },
              });
            },
          },
        ],
      }
    : undefined;

  return (
    <div className="grid gap-6 2xl:grid-cols-5 items-stretch h-full">
      {/* Stats: only for lecturers, assistants, or admin */}
      {hasStats && (
        <div className="order-1 2xl:order-2 2xl:col-span-2 h-full flex flex-col">
          <SubmissionStatistics stats={assignmentStats} className="flex-1 min-h-0" />
        </div>
      )}

      {/* List: span full width when stats are hidden */}
      <div
        className={`order-2 2xl:order-1 ${hasStats ? '2xl:col-span-3' : '2xl:col-span-5'} min-w-0 h-full flex flex-col`}
      >
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
              onSubmit={isAssignmentOpen && isStudent ? handleOpenSubmit : undefined}
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
          allowPractice={policy?.allow_practice_submissions && !auth.isStaff(module.id)}
        />
      </div>
    </div>
  );
}
