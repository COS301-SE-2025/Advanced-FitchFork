import { useState, useRef, useEffect } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import type {
  PlagiarismCaseStatus,
  PlagiarismCaseItem,
  PlagiarismGraphLink,
} from '@/types/modules/assignments/plagiarism';
import {
  listPlagiarismCases,
  deletePlagiarismCase,
  createPlagiarismCase,
  updatePlagiarismCase,
  getPlagiarismGraph,
  runMossCheck,
  getMossReport,
  bulkDeletePlagiarismCases,
  flagPlagiarismCase,
  reviewPlagiarismCase,
} from '@/services/modules/assignments/plagiarism';
import { getSubmissions } from '@/services/modules/assignments/submissions';
import type { Submission } from '@/types/modules/assignments/submissions';
import {
  DeleteOutlined,
  DeploymentUnitOutlined,
  EditOutlined,
  PlusOutlined,
  ExperimentOutlined,
  CheckCircleOutlined,
  FlagOutlined,
} from '@ant-design/icons';
import { EntityList, type EntityListHandle, type EntityListProps } from '@/components/EntityList';
import CreateModal from '@/components/common/CreateModal';
import EditModal from '@/components/common/EditModal';
import { message } from '@/utils/message';
import dayjs from 'dayjs';
import { Space, Typography, Modal, Alert } from 'antd'; // <-- add Modal, Select
import type { TreeSelectProps } from 'antd';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { useViewSlot } from '@/context/ViewSlotContext';

import {
  PlagiarismCaseCard,
  PlagiarismCaseListItem,
  PlagiarismEmptyState,
} from '@/components/plagiarism';
import PlagiarismStatusTag from '@/components/plagiarism/PlagiarismStatusTag';
import PlagiarismGraph from '@/components/plagiarism/PlagiarismGraph';
import { formatModuleCode } from '@/utils/modules';
import { DateTime, IdTag, PercentageTag } from '@/components/common';
import ConfirmModal from '@/components/utils/ConfirmModal';
import { dateTimeString } from '@/utils/dateTimeString';

/** Build TreeSelect nodes grouped by user: parents (users) are not selectable; children are submissions */
function buildSubmissionTree(subs: Submission[]) {
  type Node = {
    title: React.ReactNode;
    value: string | number;
    selectable?: boolean;
    children?: Node[];
  };

  const byUser = new Map<string, Node>();

  for (const s of subs) {
    const username = s.user?.username ?? 'unknown';
    const userKey = `user-${s.user?.id ?? username}`;

    if (!byUser.has(userKey)) {
      byUser.set(userKey, {
        title: <span className="font-medium">{username}</span>,
        value: userKey,
        selectable: false,
        children: [],
      });
    }

    const child: Node = {
      value: s.id, // actual submission id to submit
      title: (
        <>
          <span className="text-gray-400">#{s.id}</span>
          {' • '}attempt {s.attempt}
        </>
      ),
    };

    byUser.get(userKey)!.children!.push(child);
  }

  // sort submissions by id desc, users alphabetically
  const nodes = Array.from(byUser.values()).map((u) => ({
    ...u,
    children: (u.children ?? []).sort((a, b) => Number(b.value) - Number(a.value)),
  }));

  nodes.sort((a, b) => {
    const at = (a.title as any)?.props?.children ?? '';
    const bt = (b.title as any)?.props?.children ?? '';
    return String(at).localeCompare(String(bt));
  });

  return nodes;
}

const PlagiarismCases = () => {
  const { setValue } = useViewSlot();
  const navigate = useNavigate();
  const moduleDetails = useModule();
  const { assignment } = useAssignment();
  const listRef = useRef<EntityListHandle>(null);

  const moduleId = moduleDetails.id;
  const assignmentId = assignment.id;

  // modal state
  const [createOpen, setCreateOpen] = useState(false);
  const [editOpen, setEditOpen] = useState(false);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [editingItem, setEditingItem] = useState<PlagiarismCaseItem | null>(null);

  const [graphOpen, setGraphOpen] = useState(false);
  const [, setGraphLoading] = useState(false);
  const [graphLinks, setGraphLinks] = useState<PlagiarismGraphLink[]>([]);

  // MOSS modal/state
  const [mossOpen, setMossOpen] = useState(false);
  const [mossRunning, setMossRunning] = useState(false);
  const [mossReportUrl, setMossReportUrl] = useState<string | null>(null);
  const [mossGeneratedAt, setMossGeneratedAt] = useState<string | null>(null);

  const openGraph = async () => {
    try {
      setGraphLoading(true);
      const res = await getPlagiarismGraph(moduleId, assignmentId);
      if (res.success) {
        setGraphLinks(res.data.links || []);
        setGraphOpen(true);
      } else {
        message.error(res.message || 'Failed to load graph');
      }
    } catch (e) {
      message.error('Failed to load graph');
    } finally {
      setGraphLoading(false);
    }
  };

  const loadMossReport = async () => {
    try {
      const res = await getMossReport(moduleId, assignmentId);
      if (res.success && (res.data as any)?.report_url) {
        setMossReportUrl((res.data as any).report_url);
        setMossGeneratedAt((res.data as any).generated_at ?? null);
      } else {
        setMossReportUrl(null);
        setMossGeneratedAt(null);
      }
    } catch {
      setMossReportUrl(null);
      setMossGeneratedAt(null);
    }
  };

  useEffect(() => {
    loadMossReport();
  }, [moduleId, assignmentId]);

  // TreeSelect data for submissions (used in Create modal)
  const [subTree, setSubTree] = useState<{ treeData: any[]; loading: boolean }>({
    treeData: [],
    loading: false,
  });

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Plagiarism Cases
      </Typography.Text>,
    );
  }, [setValue]);

  // Fetch plagiarism cases for EntityList
  const fetchCases = async ({
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
  }): Promise<{ items: PlagiarismCaseItem[]; total: number }> => {
    const status = filters.status?.[0] as PlagiarismCaseStatus | undefined;
    const res = await listPlagiarismCases(moduleId, assignmentId, {
      page,
      per_page,
      query,
      status,
      sort,
    });

    if (res.success) {
      return { items: res.data.cases, total: res.data.total };
    } else {
      message.error(`Failed to fetch plagiarism cases: ${res.message}`);
      return { items: [], total: 0 };
    }
  };

  // Load an initial page of submissions when Create modal opens
  useEffect(() => {
    if (!createOpen) return;
    (async () => {
      setSubTree((s) => ({ ...s, loading: true }));
      const res = await getSubmissions(moduleId, assignmentId, { page: 1, per_page: 50 });
      if (res.success) {
        setSubTree({ treeData: buildSubmissionTree(res.data.submissions), loading: false });
      } else {
        setSubTree({ treeData: [], loading: false });
        message.error(`Failed to load submissions: ${res.message}`);
      }
    })();
  }, [createOpen, moduleId, assignmentId]);

  // Server-side search by username for submissions TreeSelect
  const searchSubmissions = async (username: string) => {
    setSubTree((s) => ({ ...s, loading: true }));
    const res = await getSubmissions(moduleId, assignmentId, {
      page: 1,
      per_page: 50,
      username: username?.trim() ? username : undefined,
    });
    if (res.success) {
      setSubTree({ treeData: buildSubmissionTree(res.data.submissions), loading: false });
    } else {
      setSubTree({ treeData: [], loading: false });
    }
  };

  // Create case
  const handleCreate = async (values: Record<string, any>) => {
    const res = await createPlagiarismCase(moduleId, assignmentId, {
      submission_id_1: Number(values.submission_id_1),
      submission_id_2: Number(values.submission_id_2),
      description: String(values.description ?? ''),
      similarity: Number(values.similarity ?? 0),
    });
    if (res.success) {
      message.success(res.message || 'Plagiarism case created');
      setCreateOpen(false);
      listRef.current?.refresh();
    } else {
      message.error(res.message);
    }
  };

  // Edit case
  const handleEdit = async (values: Record<string, any>) => {
    if (!editingItem) return;
    const res = await updatePlagiarismCase(moduleId, assignmentId, editingItem.id, {
      description: values.description ?? undefined,
      status: values.status as PlagiarismCaseStatus | undefined,
      similarity: Number(values.similarity) ?? undefined,
    });
    if (res.success) {
      message.success(res.message || 'Plagiarism case updated');
      setEditOpen(false);
      setEditingItem(null);
      listRef.current?.refresh();
    } else {
      message.error(res.message);
    }
  };

  // Delete case
  const handleDelete = async (caseItem: PlagiarismCaseItem, refresh: () => void) => {
    const res = await deletePlagiarismCase(moduleId, assignmentId, caseItem.id);
    if (res.success) {
      message.success(res.message || 'Plagiarism case deleted');
      refresh();
    } else {
      message.error(`Delete failed: ${res.message}`);
    }
  };

  // Single-item actions
  const handleFlag = async (caseItem: PlagiarismCaseItem, refresh: () => void) => {
    const res = await flagPlagiarismCase(moduleId, assignmentId, caseItem.id);
    if (res.success) {
      message.success('Case flagged');
      refresh();
    } else {
      message.error(res.message || 'Failed to flag case');
    }
  };

  const handleMarkReviewed = async (caseItem: PlagiarismCaseItem, refresh: () => void) => {
    const res = await reviewPlagiarismCase(moduleId, assignmentId, caseItem.id);
    if (res.success) {
      message.success('Case marked as reviewed');
      refresh();
    } else {
      message.error(res.message || 'Failed to mark as reviewed');
    }
  };

  // OPEN confirm if there are selected rows; warn if not
  const handleBulkDeleteClick = () => {
    const ids = (listRef.current?.getSelectedRowKeys() ?? []) as number[];
    if (!ids.length) {
      message.warning('No cases selected');
      return;
    }
    setConfirmOpen(true);
  };

  // Do the actual delete on confirm
  const handleBulkDeleteConfirm = async () => {
    const ids = (listRef.current?.getSelectedRowKeys() ?? [])
      .map(Number)
      .filter((n) => !Number.isNaN(n));

    if (!ids.length) {
      setConfirmOpen(false);
      return;
    }

    const res = await bulkDeletePlagiarismCases(moduleId, assignmentId, ids);
    if (res.success) {
      message.success(res.message || `Deleted ${ids.length} case(s)`);
      listRef.current?.refresh();
      listRef.current?.clearSelection(); // important
    } else {
      message.error(res.message || 'Bulk delete failed');
    }

    setConfirmOpen(false);
  };

  const handleBulkDeleteCancel = () => setConfirmOpen(false);

  // Run MOSS (all students' latest submissions)
  const doRunMoss = async () => {
    try {
      setMossRunning(true);
      const res = await runMossCheck(moduleId, assignmentId);
      if (res.success) {
        const url =
          (res.data as any)?.report_url ?? (typeof res.data === 'string' ? res.data : '') ?? '';
        if (url) setMossReportUrl(url);
        message.success(res.message || 'MOSS check completed successfully');
        listRef.current?.refresh();
        await loadMossReport();
      } else {
        message.error(res.message || 'Failed to run MOSS check');
      }
    } catch (e) {
      message.error('Failed to run MOSS check');
    } finally {
      setMossRunning(false);
    }
  };

  // Actions for EntityList
  const actions: EntityListProps<PlagiarismCaseItem>['actions'] = {
    control: [
      {
        key: 'create',
        label: 'Add Case',
        icon: <PlusOutlined />,
        handler: ({ refresh }) => {
          setCreateOpen(true);
          refresh();
        },
      },
      {
        key: 'moss',
        label: 'Run MOSS',
        icon: <ExperimentOutlined />,
        handler: () => setMossOpen(true),
      },
      {
        key: 'graph',
        label: 'View Graph',
        isPrimary: true,
        icon: <DeploymentUnitOutlined />,
        handler: async () => {
          await openGraph();
        },
      },
    ],

    // Primary entity action depends on status.
    // - review    -> Flag (primary)
    // - flagged   -> Mark Reviewed (primary)
    // - reviewed  -> Edit (primary). No "Reopen".
    entity: (entity: PlagiarismCaseItem) => {
      const editAction = {
        key: 'edit',
        label: 'Edit',
        icon: <EditOutlined />,
        handler: (ctx: { refresh: () => void }) => {
          setEditingItem(entity);
          setEditOpen(true);
          ctx.refresh();
        },
      };

      const deleteAction = {
        key: 'delete',
        label: 'Delete',
        icon: <DeleteOutlined />,
        confirm: true,
        handler: (ctx: { refresh: () => void }) => handleDelete(entity, ctx.refresh),
      };

      let primaryAction: {
        key: string;
        label: string;
        icon: React.ReactNode;
        isPrimary: true;
        handler: (ctx: { refresh: () => void }) => void | Promise<void>;
      } | null = null;

      // By default, Edit lives in the dropdown.
      let putEditInDropdown = true;

      if (entity.status === 'review') {
        primaryAction = {
          key: 'flag',
          label: 'Flag',
          icon: <FlagOutlined />,
          isPrimary: true,
          handler: (ctx) => handleFlag(entity, ctx.refresh),
        };
      } else if (entity.status === 'flagged') {
        primaryAction = {
          key: 'mark-reviewed',
          label: 'Mark Reviewed',
          icon: <CheckCircleOutlined />,
          isPrimary: true,
          handler: (ctx) => handleMarkReviewed(entity, ctx.refresh),
        };
      } else if (entity.status === 'reviewed') {
        // No "Reopen" — make Edit the primary action.
        primaryAction = {
          ...editAction,
          isPrimary: true,
        };
        putEditInDropdown = false;
      }

      const result = [];
      if (primaryAction) result.push(primaryAction);
      if (putEditInDropdown) result.push(editAction);
      result.push(deleteAction);

      return result;
    },

    // Bulk actions
    bulk: [
      {
        key: 'bulk-delete',
        label: 'Bulk Delete',
        icon: <DeleteOutlined />,
        handler: handleBulkDeleteClick,
      },
    ],
  };

  return (
    <>
      <div className="flex h-full flex-col gap-4">
        {/* Latest MOSS report banner */}
        {mossReportUrl && (
          <Alert
            type="info"
            showIcon
            message="Latest MOSS report"
            description={
              <span>
                <a href={mossReportUrl} target="_blank" rel="noreferrer" className="text-blue-600">
                  Open MOSS Report
                </a>
                {mossGeneratedAt && (
                  <>
                    {' • '}
                    <Typography.Text type="secondary">
                      Generated {dateTimeString(mossGeneratedAt, 'relative')}
                    </Typography.Text>
                  </>
                )}
              </span>
            }
          />
        )}

        <EntityList<PlagiarismCaseItem>
          ref={listRef}
          name="Plagiarism Cases"
          fetchItems={fetchCases}
          getRowKey={(c) => c.id}
          onRowClick={(c) =>
            navigate(`/modules/${moduleId}/assignments/${assignmentId}/plagiarism/${c.id}`)
          }
          renderGridItem={(c, actions) => (
            <PlagiarismCaseCard
              key={c.id}
              caseItem={c}
              actions={actions}
              onClick={() =>
                navigate(`/modules/${moduleId}/assignments/${assignmentId}/plagiarism/${c.id}`)
              }
            />
          )}
          renderListItem={(c) => (
            <PlagiarismCaseListItem
              caseItem={c}
              onClick={(caseItem) =>
                navigate(
                  `/modules/${moduleId}/assignments/${assignmentId}/plagiarism/${caseItem.id}`,
                )
              }
            />
          )}
          columnToggleEnabled
          actions={actions}
          columns={[
            {
              title: 'ID',
              dataIndex: 'id',
              key: 'id',
              defaultHidden: true,
              render: (id: number) => <IdTag id={id} />,
            },
            {
              title: 'Case',
              dataIndex: 'id',
              key: 'case',
              render: (_, c) =>
                `${c.submission_1.user.username} vs ${c.submission_2.user.username}`,
            },
            {
              title: 'Similarity',
              dataIndex: 'similarity',
              key: 'similarity',
              align: 'right',
              width: 160,
              sorter: { multiple: 2 },
              render: (_, c) => (
                <PercentageTag
                  value={c.similarity}
                  decimals={1}
                  palette="greenRed" // red → amber → green
                />
              ),
            },
            {
              title: 'Status',
              dataIndex: 'status',
              key: 'status',
              sorter: { multiple: 1 },
              filters: [
                { text: 'Review', value: 'review' },
                { text: 'Flagged', value: 'flagged' },
                { text: 'Reviewed', value: 'reviewed' },
              ],
              render: (_, c) => <PlagiarismStatusTag status={c.status} />,
            },
            {
              title: 'Description',
              dataIndex: 'description',
              key: 'description',
              defaultHidden: true,
              render: (_, c) =>
                c.description ? (
                  <div className="max-w-[48ch] line-clamp-2 text-gray-700 dark:text-neutral-300">
                    {c.description}
                  </div>
                ) : (
                  'No description'
                ),
            },
            {
              title: 'Created At',
              dataIndex: 'created_at',
              key: 'created_at',
              sorter: { multiple: 2 },
              render: (_, c) => <DateTime value={c.updated_at} variant="datetime" />,
            },
            {
              title: 'Updated At',
              dataIndex: 'updated_at',
              key: 'updated_at',
              defaultHidden: true,
              render: (_, c) => <DateTime value={c.updated_at} variant="datetime" />,
            },
          ]}
          emptyNoEntities={
            <PlagiarismEmptyState
              onCreate={() => setCreateOpen(true)}
              onRefresh={() => listRef.current?.refresh()}
              onGenerate={() => setMossOpen(true)}
            />
          }
        />
      </div>

      {/* Create */}
      <CreateModal
        open={createOpen}
        onCancel={() => setCreateOpen(false)}
        onCreate={handleCreate}
        title="Add Plagiarism Case"
        initialValues={{
          submission_id_1: undefined,
          submission_id_2: undefined,
          description: '',
          similarity: 0,
        }}
        fields={[
          {
            name: 'submission_id_1',
            label: 'Submission #1',
            type: 'tree-select',
            required: true,
            treeData: subTree.treeData, // grouped by user
            treeSelectProps: {
              showSearch: true,
              filterTreeNode: false, // server-side search
              onSearch: (v) => searchSubmissions(v),
              placeholder: 'Search by username…',
              notFoundContent: subTree.loading ? 'Searching…' : 'No submissions',
              treeNodeLabelProp: 'title', // what shows when selected
              dropdownMatchSelectWidth: 440,
            } as TreeSelectProps,
          },
          {
            name: 'submission_id_2',
            label: 'Submission #2',
            type: 'tree-select',
            required: true,
            treeData: subTree.treeData,
            treeSelectProps: {
              showSearch: true,
              filterTreeNode: false,
              onSearch: (v) => searchSubmissions(v),
              placeholder: 'Search by username…',
              notFoundContent: subTree.loading ? 'Searching…' : 'No submissions',
              treeNodeLabelProp: 'title',
              dropdownMatchSelectWidth: 440,
            } as TreeSelectProps,
          },
          { name: 'description', label: 'Description', type: 'textarea', required: true },
          { name: 'similarity', label: 'Similarity', type: 'number' },
        ]}
      />

      {/* Edit */}
      <EditModal
        open={editOpen}
        onCancel={() => {
          setEditOpen(false);
          setEditingItem(null);
        }}
        onEdit={handleEdit}
        title="Edit Plagiarism Case"
        initialValues={{
          description: editingItem?.description ?? '',
          status: editingItem?.status ?? 'review',
          similarity: editingItem?.similarity ?? 0,
        }}
        fields={[
          { name: 'description', label: 'Description', type: 'textarea' },
          { name: 'similarity', label: 'Similarity', type: 'number' },
          {
            name: 'status',
            label: 'Status',
            type: 'select',
            options: [
              { label: 'Review', value: 'review' },
              { label: 'Flagged', value: 'flagged' },
              { label: 'Reviewed', value: 'reviewed' },
            ],
          },
        ]}
      />

      <ConfirmModal
        open={confirmOpen}
        title={`Delete ${listRef.current?.getSelectedRowKeys().length ?? 0} selected case(s)?`}
        onOk={handleBulkDeleteConfirm}
        onCancel={handleBulkDeleteCancel}
      />

      {/* Graph Modal */}
      <PlagiarismGraph
        open={graphOpen}
        onClose={() => setGraphOpen(false)}
        links={graphLinks}
        title={`Plagiarism Graph (${formatModuleCode(moduleDetails.code)} • ${assignment.name})`}
      />

      {/* Run MOSS modal */}
      <Modal
        title="Run MOSS on Latest Submissions"
        open={mossOpen}
        onCancel={() => {
          setMossOpen(false);
        }}
        width={650}
        onOk={doRunMoss}
        okText={mossReportUrl ? 'Run Again' : 'Run MOSS'}
        confirmLoading={mossRunning}
      >
        <Space direction="vertical" className="w-full">
          <Typography.Paragraph type="secondary" className="mb-1">
            This runs MOSS on the latest attempt for every student in{' '}
            <strong>{formatModuleCode(moduleDetails.code)}</strong> •{' '}
            <strong>{assignment.name}</strong>.
          </Typography.Paragraph>

          <Alert
            type="warning"
            showIcon
            message="MOSS uses the language from Assignment Config"
            description={
              <span>
                Make sure the correct language is set in{' '}
                <Link
                  to={`/modules/${moduleId}/assignments/${assignmentId}/config/assignment`}
                  className="text-blue-600"
                >
                  Assignment Config
                </Link>{' '}
                before running.
              </span>
            }
          />

          {mossReportUrl && (
            <div className="mt-3">
              <Typography.Text>Report URL:&nbsp;</Typography.Text>
              <a href={mossReportUrl} target="_blank" rel="noreferrer" className="text-blue-600">
                Open MOSS Report
              </a>
              {mossGeneratedAt && (
                <>
                  {' • '}
                  <Typography.Text type="secondary">
                    Generated {dayjs(mossGeneratedAt).format('YYYY-MM-DD HH:mm')}
                  </Typography.Text>
                </>
              )}
            </div>
          )}
        </Space>
      </Modal>
    </>
  );
};

export default PlagiarismCases;
