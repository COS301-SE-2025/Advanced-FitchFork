import { useEffect, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import dayjs from 'dayjs';
import {
  PlusOutlined,
  EditOutlined,
  DeleteOutlined,
  UnlockOutlined,
  LockOutlined,
  ToolOutlined,
} from '@ant-design/icons';

import PageHeader from '@/components/PageHeader';
import AssignmentCard from '@/components/assignments/AssignmentCard';
import AssignmentTypeTag from '@/components/assignments/AssignmentTypeTag';
import AssignmentStatusTag from '@/components/assignments/AssignmentStatusTag';

import {
  listAssignments,
  deleteAssignment,
  editAssignment,
  bulkUpdateAssignments,
  bulkDeleteAssignments,
  createAssignment,
  closeAssignment,
  openAssignment,
} from '@/services/modules/assignments';

import {
  type Assignment,
  type AssignmentType,
  ASSIGNMENT_STATUSES,
  ASSIGNMENT_TYPES,
} from '@/types/modules/assignments';
import type { SortOption } from '@/types/common';
import { type EntityListHandle, EntityList } from '@/components/EntityList';
import { message } from '@/utils/message';

import EditModal from '@/components/common/EditModal';
import { useModule } from '@/context/ModuleContext';
import ConfirmModal from '@/components/utils/ConfirmModal';
import { useAuth } from '@/context/AuthContext';
import CreateModal from '@/components/common/CreateModal';
import AssignmentSetup from './steps/AssignmentSetup';
import { Typography } from 'antd';
import { useViewSlot } from '@/context/ViewSlotContext';
import AssignmentListItem from '@/components/assignments/AssignmentListItem';
import { AssignmentsEmptyState } from '@/components/assignments';
import { formatModuleCode } from '@/utils/modules';

const AssignmentsList = () => {
  const auth = useAuth();
  const module = useModule();
  const navigate = useNavigate();
  const { setValue } = useViewSlot();

  const listRef = useRef<EntityListHandle>(null);

  const [setupOpen, setSetupOpen] = useState(false);
  const [editOpen, setEditOpen] = useState(false);
  const [editingItem, setEditingItem] = useState<Assignment | null>(null);
  const [bulkEditOpen, setBulkEditOpen] = useState(false);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [setupAssignmentId, setSetupAssignmentId] = useState<number | null>(null);

  const isAdminOrLecturer = auth.isAdmin || auth.isLecturer(module.id);
  const isStudent = auth.isStudent(module.id);

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Assignments
      </Typography.Text>,
    );
  }, []);

  const fetchAssignments = async ({
    page,
    per_page,
    query,
    sort,
    filters,
  }: {
    page: number;
    per_page: number;
    query?: string;
    filters: Record<string, string[]>;
    sort: SortOption[];
  }): Promise<{ items: Assignment[]; total: number }> => {
    const res = await listAssignments(module.id, {
      page,
      per_page,
      query,
      sort,
      name: filters.name?.[0],
      assignment_type: filters.assignment_type?.[0] as AssignmentType | undefined,
    });

    if (res.success) {
      return {
        items: res.data.assignments,
        total: res.data.total,
      };
    } else {
      message.error(`Failed to fetch assignments: ${res.message}`);
      return { items: [], total: 0 };
    }
  };

  const handleCreate = async (values: Record<string, any>) => {
    const res = await createAssignment(module.id, {
      name: values.name,
      assignment_type: values.assignment_type,
      available_from: values.available_from.toISOString(),
      due_date: values.due_date.toISOString(),
      description: values.description || '',
    });

    if (res.success) {
      message.success('Assignment created');
      setSetupOpen(false);
      listRef.current?.refresh();
    } else {
      message.error(res.message);
    }
  };

  const handleEdit = async (values: Record<string, any>) => {
    if (!editingItem) return;
    const res = await editAssignment(module.id, editingItem.id, values);

    if (res.success) {
      message.success(res.message || 'Assignment updated');
      setEditOpen(false);
      setEditingItem(null);
      listRef.current?.refresh();
    } else {
      message.error(res.message);
    }
  };

  const handleBulkEdit = async (values: Record<string, any>) => {
    const selectedIds = (listRef.current?.getSelectedRowKeys() ?? [])
      .map(Number)
      .filter((id) => !isNaN(id));
    const res = await bulkUpdateAssignments(module.id, {
      assignment_ids: selectedIds,
      available_from: values.available_from || undefined,
      due_date: values.due_date || undefined,
    });

    if (res.success) {
      message.success(res.message || 'Assignments updated');
      setBulkEditOpen(false);
      listRef.current?.refresh();
    } else {
      message.error(res.message);
    }
  };

  const handleDelete = async (assignment: Assignment, refresh: () => void) => {
    const res = await deleteAssignment(module.id, assignment.id);
    if (res.success) {
      message.success('Assignment deleted successfully');
      refresh();
    } else {
      message.error(`Delete failed: ${res.message}`);
    }
  };

  const handleBulkDeleteConfirm = async () => {
    const selectedIds = (listRef.current?.getSelectedRowKeys() ?? [])
      .map(Number)
      .filter((id) => !isNaN(id));
    const res = await bulkDeleteAssignments(module.id, {
      assignment_ids: selectedIds,
    });

    if (res.success) {
      message.success(res.message || `Deleted ${selectedIds.length} assignments`);
      listRef.current?.refresh();
      listRef.current?.clearSelection();
    } else {
      message.error(`Bulk delete failed: ${res.message}`);
    }
    setConfirmOpen(false);
  };

  const handleBulkDeleteCancel = () => setConfirmOpen(false);

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto p-4">
        <div className="flex h-full flex-col gap-4">
          <PageHeader
            title="Assignments"
            description={`All the assignments for ${formatModuleCode(module.code)}`}
          />

          <EntityList<Assignment>
            ref={listRef}
            name="Assignments"
            defaultViewMode={isStudent ? 'grid' : 'table'}
            fetchItems={fetchAssignments}
            getRowKey={(a) => a.id}
            onRowClick={(a) => navigate(`/modules/${module.id}/assignments/${a.id}`)}
            renderGridItem={(assignment, actions) => (
              <AssignmentCard key={assignment.id} assignment={assignment} actions={actions} />
            )}
            listMode={auth.isStudent(module.id) || auth.isTutor(module.id)}
            renderListItem={(assignment) => (
              <AssignmentListItem
                assignment={assignment}
                onClick={(a) => navigate(`/modules/${module.id}/assignments/${a.id}`)}
              />
            )}
            columnToggleEnabled
            columns={[
              { title: 'ID', dataIndex: 'id', key: 'id', defaultHidden: true },
              { title: 'Name', dataIndex: 'name', key: 'name', sorter: { multiple: 1 } },
              {
                title: 'Description',
                dataIndex: 'description',
                key: 'description',
                sorter: { multiple: 2 },
                defaultHidden: true,
              },
              {
                title: 'Type',
                dataIndex: 'assignment_type',
                key: 'assignment_type',
                sorter: { multiple: 3 },
                filters: ASSIGNMENT_TYPES.map((t) => ({ text: t, value: t })),
                render: (_, r) => <AssignmentTypeTag type={r.assignment_type} />,
              },
              {
                title: 'Available From',
                dataIndex: 'available_from',
                key: 'available_from',
                sorter: { multiple: 4 },
                render: (_, r) => dayjs(r.available_from).format('YYYY-MM-DD HH:mm'),
              },
              {
                title: 'Due Date',
                dataIndex: 'due_date',
                key: 'due_date',
                sorter: { multiple: 5 },
                render: (_, r) => dayjs(r.due_date).format('YYYY-MM-DD HH:mm'),
              },
              {
                title: 'Status',
                key: 'status',
                render: (_, r) => <AssignmentStatusTag status={r.status} />,
              },
              {
                title: 'Created At',
                dataIndex: 'created_at',
                key: 'created_at',
                sorter: { multiple: 6 },
                defaultHidden: true,
                render: (_, r) => dayjs(r.created_at).format('YYYY-MM-DD HH:mm'),
              },
              {
                title: 'Updated At',
                dataIndex: 'updated_at',
                key: 'updated_at',
                sorter: { multiple: 7 },
                defaultHidden: true,
                render: (_, r) => dayjs(r.updated_at).format('YYYY-MM-DD HH:mm'),
              },
            ]}
            actions={
              isAdminOrLecturer
                ? {
                    control: [
                      {
                        key: 'create',
                        label: 'Add Assignment',
                        icon: <PlusOutlined />,
                        isPrimary: true,
                        handler: () => setSetupOpen(true),
                      },
                    ],
                    entity: (entity) => [
                      {
                        key: 'edit',
                        label: 'Edit',
                        icon: <EditOutlined />,
                        handler: () => {
                          setEditingItem(entity);
                          setEditOpen(true);
                        },
                      },
                      {
                        key: 'delete',
                        label: 'Delete',
                        icon: <DeleteOutlined />,
                        confirm: true,
                        handler: ({ refresh }) => handleDelete(entity, refresh),
                      },
                      ...(entity.status === 'ready' ||
                      entity.status === 'closed' ||
                      entity.status === 'archived'
                        ? [
                            {
                              key: 'open',
                              label: 'Open',
                              icon: <UnlockOutlined />,
                              handler: async ({ refresh }: { refresh: () => void }) => {
                                const res = await openAssignment(module.id, entity.id);
                                res.success
                                  ? message.success(`${entity.name} opened`)
                                  : message.error(res.message);
                                refresh();
                              },
                            },
                          ]
                        : entity.status === 'open'
                          ? [
                              {
                                key: 'close',
                                label: 'Close',
                                icon: <LockOutlined />,
                                handler: async ({ refresh }: { refresh: () => void }) => {
                                  const res = await closeAssignment(module.id, entity.id);
                                  res.success
                                    ? message.success(`${entity.name} closed`)
                                    : message.error(res.message);
                                  refresh();
                                },
                              },
                            ]
                          : []),
                      ...(entity.status === 'setup'
                        ? [
                            {
                              key: 'setup',
                              label: 'Setup',
                              isPrimary: true,
                              icon: <ToolOutlined />,
                              handler: () => setSetupAssignmentId(entity.id),
                            },
                          ]
                        : []),
                    ],
                    bulk: [
                      {
                        key: 'bulk-delete',
                        label: 'Bulk Delete',
                        icon: <DeleteOutlined />,
                        handler: () => {
                          const selected = listRef.current?.getSelectedRowKeys() ?? [];
                          if (selected.length === 0) {
                            message.warning('No assignments selected');
                            return;
                          }
                          setConfirmOpen(true);
                        },
                      },
                      {
                        key: 'bulk-edit',
                        label: 'Bulk Edit',
                        icon: <EditOutlined />,
                        isPrimary: true,
                        handler: () => {
                          const selected = listRef.current?.getSelectedRowKeys() ?? [];
                          if (selected.length === 0) {
                            message.warning('No assignments selected');
                            return;
                          }
                          setBulkEditOpen(true);
                        },
                      },
                    ],
                  }
                : undefined
            }
            emptyNoEntities={
              <AssignmentsEmptyState
                moduleLabel={formatModuleCode(module.code)}
                canCreate={isAdminOrLecturer}
                onCreate={() => setSetupOpen(true)}
                onRefresh={() => listRef.current?.refresh()}
              />
            }
          />
        </div>

        <CreateModal
          open={setupOpen}
          onCancel={() => setSetupOpen(false)}
          onCreate={handleCreate}
          fields={[
            { name: 'name', label: 'Name', type: 'text', required: true },
            {
              name: 'assignment_type',
              label: 'Type',
              type: 'select',
              required: true,
              options: ASSIGNMENT_TYPES.map((t) => ({ label: t, value: t })),
            },
            { name: 'available_from', label: 'Available From', type: 'datetime', required: true },
            { name: 'due_date', label: 'Due Date', type: 'datetime', required: true },
            { name: 'description', label: 'Description', type: 'textarea' },
          ]}
          initialValues={{
            name: '',
            assignment_type: 'assignment',
            available_from: dayjs(),
            due_date: dayjs().add(7, 'day'),
            description: '',
          }}
          title="Create Assignment"
        />

        <EditModal
          open={editOpen}
          onCancel={() => {
            setEditOpen(false);
            setEditingItem(null);
          }}
          onEdit={handleEdit}
          title="Edit Assignment"
          initialValues={{
            name: editingItem?.name ?? '',
            description: editingItem?.description ?? '',
            assignment_type: editingItem?.assignment_type ?? 'assignment',
            available_from: editingItem?.available_from ?? dayjs().toISOString(),
            due_date: editingItem?.due_date ?? dayjs().add(7, 'day').toISOString(),
          }}
          fields={[
            { name: 'name', label: 'Name', type: 'text', required: true },
            {
              name: 'assignment_type',
              label: 'Type',
              type: 'select',
              required: true,
              options: ASSIGNMENT_TYPES.map((t) => ({ label: t, value: t })),
            },
            { name: 'available_from', label: 'Available From', type: 'datetime', required: true },
            { name: 'due_date', label: 'Due Date', type: 'datetime', required: true },
            { name: 'description', label: 'Description', type: 'text' },
          ]}
        />

        <EditModal
          open={bulkEditOpen}
          onCancel={() => {
            setBulkEditOpen(false);
          }}
          onEdit={handleBulkEdit}
          title="Bulk Edit Assignments"
          initialValues={{}}
          fields={[
            {
              name: 'status',
              label: 'Status',
              type: 'select',
              options: ASSIGNMENT_STATUSES.map((s) => ({
                label: s,
                value: s,
              })),
            },
            { name: 'available_from', label: 'Available From', type: 'datetime' },
            { name: 'due_date', label: 'Due Date', type: 'datetime' },
          ]}
        />

        <ConfirmModal
          open={confirmOpen}
          title={`Delete ${listRef.current?.getSelectedRowKeys().length ?? 0} selected assignments?`}
          onOk={handleBulkDeleteConfirm}
          onCancel={handleBulkDeleteCancel}
        />

        {setupAssignmentId !== null && (
          <AssignmentSetup
            open={true}
            onClose={() => setSetupAssignmentId(null)}
            assignmentId={setupAssignmentId}
            module={module}
            onDone={() => listRef.current?.refresh()}
          />
        )}
      </div>
    </div>
  );
};

export default AssignmentsList;
