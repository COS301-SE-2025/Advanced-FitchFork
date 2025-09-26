import { useState, useRef, useEffect } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import {
  type PlagiarismCaseStatus,
  type PlagiarismCaseItem,
  type MossReport,
} from '@/types/modules/assignments/plagiarism';
import {
  listPlagiarismCases,
  deletePlagiarismCase,
  createPlagiarismCase,
  updatePlagiarismCase,
  bulkDeletePlagiarismCases,
  flagPlagiarismCase,
  reviewPlagiarismCase,
} from '@/services/modules/assignments/plagiarism';
import { listMossReports } from '@/services/modules/assignments/plagiarism/get';
import { getSubmissions } from '@/services/modules/assignments/submissions';
import type { Submission } from '@/types/modules/assignments/submissions';
import {
  DeleteOutlined,
  DeploymentUnitOutlined,
  EditOutlined,
  PlusOutlined,
  CheckCircleOutlined,
  FlagOutlined,
} from '@ant-design/icons';
import { EntityList, type EntityListHandle, type EntityListProps } from '@/components/EntityList';
import { message } from '@/utils/message';
import { Typography, type TreeSelectProps } from 'antd';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { useViewSlot } from '@/context/ViewSlotContext';
import {
  PlagiarismCaseCard,
  PlagiarismCaseListItem,
  PlagiarismEmptyState,
  MossRunModal,
  MossReportsCard,
  PlagiarismGraph,
  HashScanModal,
} from '@/components/plagiarism';
import PlagiarismStatusTag from '@/components/plagiarism/PlagiarismStatusTag';
import { formatModuleCode } from '@/utils/modules';
import { DateTime, IdTag, PercentageTag } from '@/components/common';
import FormModal, { type FormModalField } from '@/components/common/FormModal';
import ConfirmModal from '@/components/utils/ConfirmModal';

// Create-case fields (select two submissions, then metadata)
const createCaseFields: FormModalField[] = [
  {
    name: 'submission_id_1',
    label: 'Submission #1',
    type: 'tree-select',
    constraints: { required: true },
    ui: {
      props: {
        showSearch: true,
        filterTreeNode: false,
        placeholder: 'Search by username…',
        treeNodeLabelProp: 'title',
        dropdownMatchSelectWidth: 440,
        // dynamic bits set where you render the modal (see below)
      } as TreeSelectProps,
    },
  },
  {
    name: 'submission_id_2',
    label: 'Submission #2',
    type: 'tree-select',
    constraints: { required: true },
    ui: {
      props: {
        showSearch: true,
        filterTreeNode: false,
        placeholder: 'Search by username…',
        treeNodeLabelProp: 'title',
        dropdownMatchSelectWidth: 440,
      } as TreeSelectProps,
    },
  },
  {
    name: 'description',
    label: 'Description',
    type: 'textarea',
    constraints: { required: true, length: { min: 3, max: 4000 } },
    ui: { props: { rows: 4, showCount: true, maxLength: 4000 } },
  },
  {
    name: 'similarity',
    label: 'Similarity (%)',
    type: 'number',
    constraints: { number: { min: 0, max: 100, step: 0.1, precision: 1 } },
  },
  {
    name: 'lines_matched',
    label: 'Lines Matched',
    type: 'number',
    constraints: { number: { min: 0, integer: true, step: 1, precision: 0 } },
  },
];

// Edit-case fields (no submission changes here)
const editCaseFields: FormModalField[] = [
  {
    name: 'description',
    label: 'Description',
    type: 'textarea',
    constraints: { length: { max: 4000 } },
    ui: { props: { rows: 4, showCount: true, maxLength: 4000 } },
  },
  {
    name: 'similarity',
    label: 'Similarity (%)',
    type: 'number',
    constraints: { number: { min: 0, max: 100, step: 0.1, precision: 1 } },
  },
  {
    name: 'status',
    label: 'Status',
    type: 'select',
    constraints: { required: true },
    options: [
      { label: 'Review', value: 'review' },
      { label: 'Flagged', value: 'flagged' },
      { label: 'Reviewed', value: 'reviewed' },
    ],
  },
];

/** Build TreeSelect nodes grouped by user */
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
      value: s.id,
      title: (
        <>
          <span className="text-gray-400">#{s.id}</span>
          {' • '}attempt {s.attempt}
        </>
      ),
    };

    byUser.get(userKey)!.children!.push(child);
  }

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

  // Run MOSS modal
  const [mossOpen, setMossOpen] = useState(false);
  const [hashOpen, setHashOpen] = useState(false);

  // Report list (source of truth)
  const [reports, setReports] = useState<MossReport[]>([]);
  const [reportsLoading, setReportsLoading] = useState(false);

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Plagiarism Cases
      </Typography.Text>,
    );
  }, [setValue]);

  const loadReports = async () => {
    setReportsLoading(true);
    try {
      const res = await listMossReports(moduleId, assignmentId);
      if (res.success) {
        setReports(res.data?.reports ?? []);
      } else {
        setReports([]);
      }
    } catch {
      setReports([]);
    } finally {
      setReportsLoading(false);
    }
  };

  useEffect(() => {
    loadReports();
  }, [moduleId, assignmentId]);

  // TreeSelect data for submissions (used in Create modal)
  const [subTree, setSubTree] = useState<{ treeData: any[]; loading: boolean }>({
    treeData: [],
    loading: false,
  });

  // Load initial submissions when Create modal opens
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
      lines_matched: Number(values.lines_matched ?? 0),
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
    const similarity =
      values.similarity === undefined || values.similarity === null
        ? undefined
        : Number(values.similarity);
    const res = await updatePlagiarismCase(moduleId, assignmentId, editingItem.id, {
      description: values.description ?? undefined,
      status: values.status as PlagiarismCaseStatus | undefined,
      similarity,
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
      listRef.current?.clearSelection();
    } else {
      message.error(res.message || 'Bulk delete failed');
    }

    setConfirmOpen(false);
  };

  const handleBulkDeleteCancel = () => setConfirmOpen(false);

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
        key: 'graph',
        label: 'View Graph',
        isPrimary: true,
        icon: <DeploymentUnitOutlined />,
        handler: () => setGraphOpen(true),
      },
      {
        key: 'hash-scan',
        label: 'Run Hash Scan',
        isPrimary: false,
        icon: <CheckCircleOutlined />, // pick an icon you like
        handler: () => setHashOpen(true),
      },
    ],
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
    bulk: [
      {
        key: 'bulk-delete',
        label: 'Bulk Delete',
        icon: <DeleteOutlined />,
        handler: handleBulkDeleteClick,
      },
    ],
  };

  const showReportsSidebar = reportsLoading || (reports?.length ?? 0) > 0;

  return (
    <>
      {/* Two-column layout: left expands to full width if no reports */}
      <div className="grid grid-cols-1 lg:grid-cols-5 gap-4 h-full">
        {/* LEFT: cases (span all 5 cols when there's no sidebar) */}
        <div
          className={`min-h-0 space-y-3 ${showReportsSidebar ? 'lg:col-span-3' : 'lg:col-span-5'}`}
        >
          <EntityList<PlagiarismCaseItem>
            ref={listRef}
            name="Plagiarism Cases"
            fetchItems={async ({ page, per_page, query, filters, sort }) => {
              const status = filters.status?.[0] as PlagiarismCaseStatus | undefined;

              const reportIdFilterRaw = filters.report_id?.[0];
              const report_id =
                reportIdFilterRaw === undefined || reportIdFilterRaw === null
                  ? undefined
                  : Number(reportIdFilterRaw);

              const res = await listPlagiarismCases(moduleId, assignmentId, {
                page,
                per_page,
                query,
                status,
                sort,
                report_id,
              });
              if (res.success) {
                return { items: res.data.cases, total: res.data.total };
              }
              message.error(`Failed to fetch plagiarism cases: ${res.message}`);
              return { items: [], total: 0 };
            }}
            getRowKey={(c) => c.id}
            // onRowClick={(c) =>
            //   navigate(`/modules/${moduleId}/assignments/${assignmentId}/plagiarism/${c.id}`)
            // }
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
                render: (_: any, c: PlagiarismCaseItem) => {
                  const s1Path = `/modules/${moduleId}/assignments/${assignmentId}/submissions/${c.submission_1.id}`;
                  const s2Path = `/modules/${moduleId}/assignments/${assignmentId}/submissions/${c.submission_2.id}`;
                  const stopRow = (e: React.SyntheticEvent) => {
                    e.stopPropagation();
                  };
                  // stopPropagation so clicking the link doesn't open the case row route
                  return (
                    // wrapper also stops bubbling just in case
                    <span onClick={stopRow} onMouseDown={stopRow} onKeyDown={stopRow}>
                      <Link
                        to={s1Path}
                        onClick={stopRow}
                        onMouseDown={stopRow}
                        className="font-medium"
                      >
                        {c.submission_1.user.username}
                      </Link>{' '}
                      vs{' '}
                      <Link
                        to={s2Path}
                        onClick={stopRow}
                        onMouseDown={stopRow}
                        className="font-medium"
                      >
                        {c.submission_2.user.username}
                      </Link>
                    </span>
                  );
                },
              },
              {
                title: 'Similarity',
                dataIndex: 'similarity',
                key: 'similarity',
                align: 'right',
                width: 160,
                sorter: { multiple: 2 },
                render: (_, c) => (
                  <PercentageTag value={c.similarity} decimals={1} scheme="green-red" />
                ),
              },
              // NEW: Lines matched
              {
                title: 'Lines',
                dataIndex: 'lines_matched',
                key: 'lines_matched',
                align: 'right',
                width: 120,
                sorter: { multiple: 2 },
                render: (_, c) => c.lines_matched?.toLocaleString?.() ?? c.lines_matched,
              },
              {
                title: 'Report',
                dataIndex: 'report_id',
                key: 'report_id',
                width: 140,
                defaultHidden: true,
                // build options from loaded reports
                filters: reports.map((r) => ({
                  text: r.description?.trim()
                    ? `#${r.id} — ${r.description}`
                    : `#${r.id} — ${new Date(r.generated_at).toLocaleString()}`,
                  value: String(r.id), // keep as string (EntityList gives filters as strings)
                })),
                filterMultiple: false,
                render: (_, c) =>
                  c.report_id ? <>#{c.report_id}</> : <span className="text-gray-400">—</span>,
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
                render: (_, c) => <DateTime value={c.created_at} variant="datetime" />,
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
                onHashScan={() => setHashOpen(true)}
              />
            }
            filtersStorageKey={`modules:${moduleId}:assignments:${assignmentId}:plagiarism:filters:v1`}
          />
        </div>

        {/* RIGHT: Reports card (only render when we actually have reports or are loading) */}
        {showReportsSidebar && (
          <aside className="lg:col-span-2 !space-y-4">
            <MossReportsCard
              moduleId={moduleId}
              assignmentId={assignmentId}
              reports={reports}
              loading={reportsLoading}
              onOpenRunMoss={() => setMossOpen(true)}
              onRefresh={loadReports}
            />
          </aside>
        )}
      </div>

      {/* Create */}
      <FormModal
        open={createOpen}
        onCancel={() => setCreateOpen(false)}
        onSubmit={handleCreate}
        title="Add Plagiarism Case"
        submitText="Create"
        initialValues={{
          submission_id_1: undefined,
          submission_id_2: undefined,
          description: '',
          similarity: 0,
          lines_matched: 0,
        }}
        // inject dynamic tree data + search handlers for both TreeSelects
        fields={createCaseFields.map((f) =>
          f.name === 'submission_id_1' || f.name === 'submission_id_2'
            ? {
                ...f,
                ui: {
                  ...f.ui,
                  props: {
                    ...(f.ui?.props || {}),
                    treeData: subTree.treeData,
                    notFoundContent: subTree.loading ? 'Searching…' : 'No submissions',
                    onSearch: (v: string) => searchSubmissions(v),
                  } as TreeSelectProps,
                },
              }
            : f,
        )}
      />

      {/* Edit */}
      <FormModal
        open={editOpen}
        onCancel={() => {
          setEditOpen(false);
          setEditingItem(null);
        }}
        onSubmit={handleEdit}
        title="Edit Plagiarism Case"
        submitText="Save"
        initialValues={{
          description: editingItem?.description ?? '',
          status: editingItem?.status ?? 'review',
          similarity: editingItem?.similarity ?? 0,
        }}
        fields={editCaseFields}
      />

      {/* bulk delete */}
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
        moduleId={moduleId}
        assignmentId={assignmentId}
        title={`Plagiarism Graph (${formatModuleCode(moduleDetails.code)} • ${assignment.name})`}
      />

      {/* Run MOSS modal */}
      <MossRunModal
        open={mossOpen}
        onClose={() => setMossOpen(false)}
        moduleId={moduleId}
        assignmentId={assignmentId}
        onRan={loadReports}
        latestReportUrl={reports?.[0]?.report_url}
        latestGeneratedAt={reports?.[0]?.generated_at ?? null}
        hasArchive={Boolean(reports?.[0]?.has_archive)}
        latestArchiveAt={reports?.[0]?.archive_generated_at ?? null}
      />

      {/* Run Hash Scan modal */}
      <HashScanModal
        open={hashOpen}
        onClose={() => setHashOpen(false)}
        moduleId={moduleId}
        assignmentId={assignmentId}
        onRan={() => {
          // refresh cases if create_cases=true created any
          listRef.current?.refresh();
        }}
      />
    </>
  );
};

export default PlagiarismCases;
