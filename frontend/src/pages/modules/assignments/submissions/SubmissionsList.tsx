import { Badge, Button, Descriptions, Divider, Modal, Switch, Tag, Typography } from 'antd';
import { AuditOutlined, DeleteOutlined, RedoOutlined } from '@ant-design/icons';
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

import {
  SUBMISSION_STATUSES,
  type ResubmitResponse,
  type Submission,
} from '@/types/modules/assignments/submissions';
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
  SubmissionStatusTag,
  SubmitAssignmentModal,
} from '@/components/submissions';
import {
  bulkDeleteSubmissions,
  deleteSubmission,
} from '@/services/modules/assignments/submissions/delete';
import useApp from 'antd/es/app/useApp';
import { PercentageTag } from '@/components/common';
import { useSubmissionStaffProgressWs } from '@/hooks/useSubmissionStaffProgressWs';

// ─────────────────────────────────────────────────────────────
// Separate, simple Summary Modal
// ─────────────────────────────────────────────────────────────
function ResubmitSummaryModal({
  open,
  title,
  data,
  requestedCount,
  onClose,
}: {
  open: boolean;
  title: string;
  data?: ResubmitResponse;
  requestedCount?: number;
  onClose: () => void;
}) {
  const started = data?.started ?? 0;
  const skipped = data?.skipped_in_progress ?? 0;
  const failed = data?.failed ?? [];
  const requested =
    typeof requestedCount === 'number' && !Number.isNaN(requestedCount)
      ? requestedCount
      : Math.max(0, started + skipped + failed.length);

  let status: React.ReactNode = <Badge status="default" text="No actions" />;
  if (started > 0 && failed.length === 0) status = <Badge status="success" text="Started" />;
  else if (started > 0 && failed.length > 0)
    status = <Badge status="warning" text="Started with errors" />;
  else if (started === 0 && skipped > 0 && failed.length === 0)
    status = <Badge status="processing" text="Skipped (in progress)" />;
  else if (failed.length > 0) status = <Badge status="error" text="Failed to start" />;

  return (
    <Modal
      open={open}
      title={title}
      centered
      width={560}
      onCancel={onClose}
      footer={[
        <Button key="close" onClick={onClose} className="ant-btn ant-btn-primary">
          Close
        </Button>,
      ]}
    >
      <Descriptions column={1} size="small" bordered>
        <Descriptions.Item label="Requested">{requested}</Descriptions.Item>
        <Descriptions.Item label="Started">{started}</Descriptions.Item>
        <Descriptions.Item label="Skipped (already in progress)">{skipped}</Descriptions.Item>
        <Descriptions.Item label="Failed to start">{failed.length}</Descriptions.Item>
        <Descriptions.Item label="Status">{status}</Descriptions.Item>
      </Descriptions>

      {failed.length > 0 && (
        <>
          <Divider style={{ margin: '12px 0' }} />
          <Typography.Text type="secondary">Errors</Typography.Text>
          <div style={{ maxHeight: 180, overflow: 'auto', marginTop: 8 }}>
            {failed.map((f, i) => (
              <Typography.Paragraph key={i} style={{ marginBottom: 6 }}>
                <Tag color="red" style={{ marginRight: 8 }}>
                  {f.id ?? '—'}
                </Tag>
                {f.error}
              </Typography.Paragraph>
            ))}
          </div>
        </>
      )}
    </Modal>
  );
}

// Extend with *extra* UI-only fields; DO NOT override `status`
type StudentSubmission = Submission & {
  path: string;
  percentageMark?: number;
};

export default function SubmissionsList() {
  const navigate = useNavigate();
  const module = useModule();
  const { setValue } = useViewSlot();
  const { modal, message } = useApp();

  const { assignment, policy, refreshAssignment, refreshAssignmentStats } = useAssignment();
  const auth = useAuth();
  const canToggleIgnored =
    auth.isLecturer(module.id) || auth.isAssistantLecturer(module.id) || auth.isAdmin;

  const [submitModalOpen, setSubmitModalOpen] = useState(false);
  const [submitting, setSubmitting] = useState(false);

  // summary modal state
  const [summaryOpen, setSummaryOpen] = useState(false);
  const [summaryData, setSummaryData] = useState<ResubmitResponse | undefined>(undefined);
  const [summaryRequested, setSummaryRequested] = useState<number | undefined>(undefined);
  const [summaryTitle, setSummaryTitle] = useState('Resubmission summary');

  // staff stream (auto no-op for non-staff via the hook’s internal role check)
  const staffWs = useSubmissionStaffProgressWs(module.id, assignment.id, { singleLatest: false });

  const isAssignmentOpen = assignment.status === 'open';
  const isStudent = auth.isStudent(module.id);

  const entityListRef = useRef<EntityListHandle>(null);

  // helper to recompute percentage
  const pctOf = (mark?: { earned: number; total: number }) =>
    mark && mark.total > 0 ? Math.round((mark.earned / mark.total) * 100) : undefined;

  // ── Live WS → table: patch existing rows and insert new ones (with username if provided) ──
  useEffect(() => {
    if (!staffWs?.progressById) return;
    const rows = Object.values(staffWs.progressById);

    rows.forEach((p) => {
      const patch: Partial<StudentSubmission> = {
        status: p.status,
        updated_at: p.ts ?? undefined,
      };
      if (p.mark) {
        patch.mark = { earned: p.mark.earned, total: p.mark.total } as any;
        patch.percentageMark = pctOf(p.mark);
      }

      const username = p.userUsername ?? (p.userId != null ? String(p.userId) : undefined);

      // minimal, safe stub for brand-new rows
      const stub: StudentSubmission = {
        id: p.submissionId,
        attempt: 1,
        filename: '',
        hash: '',
        created_at: p.ts ?? new Date().toISOString(),
        updated_at: p.ts ?? new Date().toISOString(),
        mark: patch.mark ?? { earned: 0, total: 0 },
        is_practice: false,
        is_late: false,
        ignored: false,
        status: (patch.status as any) ?? 'queued',
        tasks: [],
        user: p.userId != null || username ? ({ id: p.userId, username } as any) : undefined,
        // UI-only
        percentageMark: patch.percentageMark,
        path: `/api/modules/${module.id}/assignments/${assignment.id}/submissions/${p.submissionId}/file`,
      };

      // Upsert (replace if exists, prepend if new)
      entityListRef.current?.upsertRows([{ ...stub, ...patch } as StudentSubmission], 'prepend');
    });

    if (rows.some((r) => r.status === 'graded')) {
      refreshAssignmentStats?.();
    }
  }, [staffWs.progressById, module.id, assignment.id, refreshAssignmentStats]);

  useEffect(() => {
    setValue(
      <Typography.Text
        className="text-base font-medium text-gray-900 dark:text-gray-100 truncate"
        title="Submissions"
      >
        Submissions
      </Typography.Text>,
    );
  }, [setValue]);

  useEffect(() => {
    const listener = () => {
      entityListRef.current?.refresh();
    };
    EventBus.on('submission:updated', listener);
    return () => {
      EventBus.off('submission:updated', listener);
    };
  }, []);

  const handleOpenSubmit = () => setSubmitModalOpen(true);

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
    if (!module.id || !assignment.id) return { items: [], total: 0 };

    const res = await getSubmissions(module.id, assignment.id, {
      page,
      per_page,
      query,
      sort,
      username: filters['user.username']?.[0],
      status:
        filters['status'] && filters['status'].length ? filters['status'].join(',') : undefined,
    });

    const { submissions, total } = res.data;

    const items: StudentSubmission[] = submissions.map((s) => {
      const pct =
        s.mark && typeof s.mark.total === 'number' && s.mark.total > 0
          ? Math.round((s.mark.earned / s.mark.total) * 100)
          : undefined;

      return {
        ...s,
        percentageMark: pct,
        path: `/api/modules/${module.id}/assignments/${assignment.id}/submissions/${s.id}/file`,
      };
    });

    return { items, total };
  };

  const handleSubmitAssignment = async (
    file: File,
    isPractice: boolean,
    attestOwnership: boolean,
  ) => {
    setSubmitModalOpen(false);
    setSubmitting(true);
    const hide = message.loading('Submitting assignment...');
    try {
      await submitAssignment(module.id, assignment.id, file, isPractice, attestOwnership);
      await refreshAssignment();
      message.success('Submission successful');
      EventBus.emit('submission:updated');
      entityListRef.current?.refresh();
    } catch {
      message.error('Submission failed');
    } finally {
      hide();
      setSubmitting(false);
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
      title: 'Status',
      dataIndex: 'status',
      key: 'status',
      // show nice tags in the filter dropdown too
      filters: SUBMISSION_STATUSES.map((s) => ({
        text: <SubmissionStatusTag status={s} />,
        value: s,
      })),
      filterMultiple: true,
      render: (_: unknown, record) => <SubmissionStatusTag status={record.status} />,
    },
    {
      title: 'Is Practice',
      dataIndex: 'is_practice',
      key: 'is_practice',
      render: (v) => (v ? <Tag color="gold">Yes</Tag> : <Tag>No</Tag>),
    },
    {
      title: 'Mark',
      key: 'mark_pct',
      sorter: { multiple: 3 },
      render: (_, r) => {
        const earned = r.mark?.earned ?? 0;
        const total = r.mark?.total ?? 0;
        const pct = total > 0 ? Math.round((earned / total) * 100) : null;

        return pct != null ? (
          <PercentageTag value={pct} scheme="red-green" />
        ) : (
          <Tag>Not marked</Tag>
        );
      },
    },
    {
      title: 'Ignored',
      dataIndex: 'ignored',
      key: 'ignored',
      defaultHidden: !(auth.isStaff(module.id) || auth.isAdmin),
      render: (_: boolean, record) => {
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
                      await refreshAssignmentStats?.();
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
        entity: (entity): EntityAction<StudentSubmission>[] => [
          {
            key: 'resubmit',
            label: 'Resubmit',
            icon: <RedoOutlined />,
            isPrimary: true,
            confirm: true,
            handler: async () => {
              try {
                const res = await resubmitSubmissions(module.id, assignment.id, {
                  submission_ids: [entity.id],
                });
                if (res.success) {
                  message.success(res.message || 'Resubmission started');
                  EventBus.emit('submission:updated');
                } else {
                  message.error(res.message || `Failed to resubmit submission ${entity.id}`);
                }
              } catch (err) {
                console.error(err);
                message.error(`Failed to resubmit submission ${entity.id}`);
              }
            },
          },
          {
            key: 'remark',
            label: 'Re-mark',
            icon: <AuditOutlined />,
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
        ],
        bulk: [
          {
            key: 'bulk-resubmit',
            label: 'Bulk Resubmit',
            icon: <RedoOutlined />,
            handler: async ({ selected, refresh }) => {
              const ids = (selected as number[]) ?? [];
              if (!ids.length) return message.warning('No submissions selected');

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
                    setSummaryTitle('Bulk resubmission started');
                    setSummaryRequested(ids.length);
                    setSummaryData(res.data);
                    setSummaryOpen(true);

                    if (res.success) {
                      EventBus.emit('submission:updated');
                      refresh();
                    } else {
                      message.error(res.message || 'Failed to re-run some submissions');
                    }
                  } catch (err) {
                    console.error(err);
                    message.error('Failed to re-run some submissions');
                  }
                },
              });
            },
          },
          {
            key: 'bulk-remark',
            label: 'Bulk Re-mark',
            icon: <AuditOutlined />,
            handler: async ({ selected, refresh }) => {
              const ids = (selected as number[]) ?? [];
              if (!ids.length) return message.warning('No submissions selected');
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
            key: 'bulk-delete',
            label: 'Bulk Delete',
            icon: <DeleteOutlined />,
            handler: async ({ selected, refresh }) => {
              const ids = (selected as number[]) ?? [];
              if (!ids.length) return message.warning('No submissions selected');
              modal.confirm({
                title: `Delete ${ids.length} submission${ids.length === 1 ? '' : 's'}?`,
                icon: null,
                centered: true,
                okText: `Delete ${ids.length}`,
                cancelText: 'Cancel',
                okButtonProps: { danger: true },
                content: (
                  <div style={{ marginTop: 8 }}>
                    <Typography.Paragraph>
                      You&apos;re about to <b>delete</b> <Tag>{ids.length}</Tag>
                      submission{ids.length === 1 ? '' : 's'}.
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
                      const { deleted } = res.data || {};
                      message.success(
                        res.message || `Deleted ${deleted ?? ids.length}/${ids.length} submissions`,
                      );
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
        ],
        control: [
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
                    setSummaryTitle('Resubmit all started');
                    setSummaryRequested(undefined);
                    setSummaryData(res.data);
                    setSummaryOpen(true);

                    if (res.success) {
                      EventBus.emit('submission:updated');
                      refresh();
                    } else {
                      message.error(res.message || 'Failed to re-run all submissions');
                    }
                  } catch (err) {
                    console.error(err);
                    message.error('Failed to re-run all submissions');
                  }
                },
              });
            },
          },
          {
            key: 'remark-all',
            label: 'Re-mark All',
            icon: <AuditOutlined />,
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
        ],
      }
    : undefined;

  return (
    <div className="min-w-0 h-full flex flex-col">
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
        open={submitModalOpen}
        onClose={() => setSubmitModalOpen(false)}
        onSubmit={handleSubmitAssignment}
        loading={submitting}
        title="Submit Assignment"
        accept=".zip,.tar,.gz,.tgz"
        maxSizeMB={50}
        defaultIsPractice={false}
        allowPractice={policy?.allow_practice_submissions && !auth.isStaff(module.id)}
      />

      {/* Separate Summary Modal */}
      <ResubmitSummaryModal
        open={summaryOpen}
        title={summaryTitle}
        data={summaryData}
        requestedCount={summaryRequested}
        onClose={() => setSummaryOpen(false)}
      />
    </div>
  );
}
