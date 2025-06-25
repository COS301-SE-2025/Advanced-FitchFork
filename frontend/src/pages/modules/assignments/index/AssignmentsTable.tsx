import {
  Typography,
  Table,
  Button,
  Space,
  Popconfirm,
  Tooltip,
  Tag,
  DatePicker,
  Input,
  Select,
  Empty,
  Dropdown,
} from 'antd';
import {
  EditOutlined,
  DeleteOutlined,
  FileTextOutlined,
  CheckOutlined,
  CloseOutlined,
  ReloadOutlined,
  MoreOutlined,
} from '@ant-design/icons';
import { useEffect, useState } from 'react';
import type { ColumnsType } from 'antd/es/table';
import TableControlBar from '@/components/TableControlBar';
import TableTagSummary from '@/components/TableTagSummary';
import type { SortOption } from '@/types/common';
import { useTableQuery } from '@/hooks/useTableQuery';
import dayjs from 'dayjs';
import weekday from 'dayjs/plugin/weekday';
import localeData from 'dayjs/plugin/localeData';
import isSameOrBefore from 'dayjs/plugin/isSameOrBefore';
import isSameOrAfter from 'dayjs/plugin/isSameOrAfter';
import TableCreateModal from '@/components/TableCreateModal';
import { useNotifier } from '@/components/Notifier';
import { useModule } from '@/context/ModuleContext';
import { useNavigate } from 'react-router-dom';
import {
  listAssignments,
  createAssignment,
  editAssignment,
  deleteAssignment,
} from '@/services/modules/assignments';
import type {
  AssignmentType,
  Assignment,
  PostAssignmentRequest,
  PutAssignmentRequest,
} from '@/types/modules/assignments';

dayjs.extend(weekday);
dayjs.extend(localeData);
dayjs.extend(isSameOrBefore);
dayjs.extend(isSameOrAfter);

const { Title, Text } = Typography;

const ASSIGNMENT_TYPES: AssignmentType[] = ['Assignment', 'Practical'];

const AssignmentsTable = () => {
  // ======================================================================
  // =========================== State and Hooks ==========================
  // ======================================================================

  const { notifySuccess, notifyError } = useNotifier();
  const navigate = useNavigate();

  const {
    searchTerm,
    setSearchTerm,
    sorterState,
    setSorterState,
    filterState,
    setFilterState,
    pagination,
    setPagination,
    clearSearch,
    clearSorters,
    clearFilters,
    clearAll,
  } = useTableQuery();

  const module = useModule();

  const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([]);
  const [assignments, setAssignments] = useState<Assignment[]>([]);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editedRow, setEditedRow] = useState<Partial<Assignment>>({});
  const [loading, setLoading] = useState(false);
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [newAssignment, setNewAssignment] = useState<Partial<Assignment>>({
    name: 'New Assignment',
    assignment_type: 'Assignment',
    available_from: dayjs().toISOString(),
    due_date: dayjs().toISOString(),
  });

  // ======================================================================
  // =========================== Data Fetching ============================
  // ======================================================================

  const fetchAssignments = async () => {
    setLoading(true);
    const sort: SortOption[] = sorterState;

    const res = await listAssignments(module.id, {
      page: pagination.current,
      per_page: pagination.pageSize,
      query: searchTerm,
      sort,
      name: filterState.name?.[0],
      assignment_type: filterState.assignment_type?.[0] as AssignmentType | undefined,
    });

    if (res.success) {
      setAssignments(res.data.assignments);
      setPagination({ total: res.data.total });
    } else {
      notifyError('Fetch Failed', 'Could not load assignment data');
    }

    setLoading(false);
  };

  useEffect(() => {
    fetchAssignments();
  }, [module.id, pagination.current, pagination.pageSize, sorterState, searchTerm, filterState]);

  // ======================================================================
  // ========================== Add Item Handlers =========================
  // ======================================================================

  const handleAddAssignment = () => {
    clearAll();
    setNewAssignment({
      name: '',
      assignment_type: 'Assignment',
      available_from: dayjs().toISOString(),
      due_date: dayjs().toISOString(),
    });
    setIsAddModalOpen(true);
  };

  const handleSubmitNewAssignment = async (values: Partial<PostAssignmentRequest>) => {
    const payload: PostAssignmentRequest = {
      name: values.name!,
      assignment_type: values.assignment_type!,
      available_from: dayjs(values.available_from).toISOString(),
      due_date: dayjs(values.due_date).toISOString(),
      description: values.description ?? '',
    };

    const res = await createAssignment(module.id, payload);

    if (res.success) {
      notifySuccess('Created', 'Assignment successfully created');
      setIsAddModalOpen(false);
      fetchAssignments();
    } else {
      notifyError('Creation Failed', 'Unable to create the new assignment');
    }
  };

  // ======================================================================
  // ================== Edit & Delete Logic (Assignment) ==================
  // ======================================================================

  const saveEdit = async (id: number) => {
    const payload: PutAssignmentRequest = {
      name: editedRow.name,
      assignment_type: editedRow.assignment_type,
      available_from: editedRow.available_from
        ? dayjs(editedRow.available_from).toISOString()
        : undefined,
      due_date: editedRow.due_date ? dayjs(editedRow.due_date).toISOString() : undefined,
      description: editedRow.description,
    };
    const res = await editAssignment(module.id, id, payload);

    if (res.success) {
      notifySuccess('Updated', 'Assignment changes have been saved');
      setEditingId(null);
      setEditedRow({});
      fetchAssignments();
    } else {
      notifyError('Update Failed', 'Could not update the assignment');
    }
  };

  const handleDeleteAssignment = async (assignmentId: number) => {
    const res = await deleteAssignment(module.id, assignmentId);
    if (res.success) {
      notifySuccess('Deleted', 'Assignment removed successfully');
      fetchAssignments();
    } else {
      notifyError('Delete Failed', 'Could not delete the assignment');
    }
  };

  const handleBulkDelete = async () => {
    for (const key of selectedRowKeys) {
      await handleDeleteAssignment(Number(key));
    }
    setSelectedRowKeys([]);
  };

  // ======================================================================
  // ==================== Table Columns Configuration =====================
  // ======================================================================

  const columns: ColumnsType<Assignment> = [
    {
      title: 'Name',
      dataIndex: 'name',
      key: 'name',
      filteredValue: filterState.name || null,
      sorter: { multiple: 1 },
      sortOrder: sorterState.find((s) => s.field === 'name')?.order ?? null,
      onFilter: (value, record) =>
        typeof value === 'string' &&
        record.name.toLowerCase().includes(value.toLowerCase().replace(/\s+/g, '')),
      filterDropdown: ({ setSelectedKeys, selectedKeys, confirm, clearFilters }) => (
        <div className="flex flex-col gap-2 p-2 w-56">
          <Input
            placeholder="Search name"
            value={selectedKeys[0]}
            onChange={(e) => setSelectedKeys([e.target.value])}
            onPressEnter={() => confirm()}
          />
          <div className="flex justify-between gap-2">
            <Button type="primary" size="small" onClick={() => confirm()}>
              Filter
            </Button>
            <Button
              size="small"
              onClick={() => {
                clearFilters?.();
                confirm({ closeDropdown: true });
              }}
            >
              Reset
            </Button>
          </div>
        </div>
      ),
      render: (_, record) =>
        editingId === record.id ? (
          <Input
            value={editedRow.name}
            onChange={(e) => setEditedRow((r) => ({ ...r, name: e.target.value }))}
          />
        ) : (
          <Space>
            <FileTextOutlined />
            {record.name}
          </Space>
        ),
    },
    {
      title: 'Type',
      dataIndex: 'assignment_type',
      key: 'assignment_type',
      sorter: { multiple: 2 },
      sortOrder: sorterState.find((s) => s.field === 'assignment_type')?.order ?? null,
      filters: ASSIGNMENT_TYPES.map((type) => ({ text: type, value: type })),
      filteredValue: filterState.assignment_type || null,
      onFilter: () => true,
      render: (_, record) =>
        editingId === record.id ? (
          <Select
            value={editedRow.assignment_type}
            onChange={(val) => setEditedRow((r) => ({ ...r, assignment_type: val }))}
            options={ASSIGNMENT_TYPES.map((t) => ({ value: t, label: t }))}
            style={{ width: 120 }}
          />
        ) : (
          <Tag color={record.assignment_type === 'Practical' ? 'blue' : 'green'}>
            {record.assignment_type}
          </Tag>
        ),
    },
    {
      title: 'Available From',
      dataIndex: 'available_from',
      key: 'available_from',
      sorter: { multiple: 3 },
      sortOrder: sorterState.find((s) => s.field === 'available_from')?.order ?? null,
      render: (_, record) =>
        editingId === record.id ? (
          <DatePicker
            showTime={{ format: 'HH:mm' }}
            value={editedRow.available_from ? dayjs(editedRow.available_from) : undefined}
            onChange={(d) => setEditedRow((r) => ({ ...r, available_from: d?.toISOString() }))}
            allowClear={false}
            format="YYYY-MM-DD HH:mm"
          />
        ) : (
          dayjs(record.available_from).format('YYYY-MM-DD HH:mm')
        ),
    },
    {
      title: 'Due Date',
      dataIndex: 'due_date',
      key: 'due_date',
      sorter: { multiple: 4 },
      sortOrder: sorterState.find((s) => s.field === 'due_date')?.order ?? null,
      render: (_, record) =>
        editingId === record.id ? (
          <DatePicker
            showTime={{ format: 'HH:mm' }}
            value={editedRow.due_date ? dayjs(editedRow.due_date) : undefined}
            onChange={(d) => setEditedRow((r) => ({ ...r, due_date: d?.toISOString() }))}
            allowClear={false}
            format="YYYY-MM-DD HH:mm"
          />
        ) : (
          dayjs(record.due_date).format('YYYY-MM-DD HH:mm')
        ),
    },
    {
      title: 'Actions',
      key: 'actions',
      align: 'right',
      width: 100,
      render: (_, record) => {
        const isEditing = editingId === record.id;

        if (isEditing) {
          return (
            <Space>
              <Tooltip title="Save">
                <Button
                  type="primary"
                  shape="default"
                  icon={<CheckOutlined />}
                  size="small"
                  onClick={(e) => {
                    e.stopPropagation();
                    saveEdit(record.id);
                  }}
                />
              </Tooltip>
              <Tooltip title="Cancel">
                <Button
                  shape="default"
                  icon={<CloseOutlined />}
                  size="small"
                  onClick={(e) => {
                    e.stopPropagation();
                    setEditingId(null);
                    setEditedRow({});
                  }}
                />
              </Tooltip>
            </Space>
          );
        }

        return (
          <div onClick={(e) => e.stopPropagation()}>
            <Dropdown
              trigger={['click']}
              menu={{
                items: [
                  {
                    key: 'edit',
                    icon: <EditOutlined />,
                    label: 'Edit',
                  },
                  {
                    key: 'delete',
                    icon: <DeleteOutlined />,
                    danger: true,
                    label: (
                      <Popconfirm
                        title="Delete this assignment?"
                        onConfirm={(e) => {
                          e?.stopPropagation();
                          handleDeleteAssignment(record.id);
                        }}
                        onCancel={(e) => e?.stopPropagation()}
                        okText="Yes"
                        cancelText="No"
                      >
                        <span onClick={(e) => e.stopPropagation()}>Delete</span>
                      </Popconfirm>
                    ),
                  },
                ],
                onClick: ({ key, domEvent }) => {
                  domEvent.preventDefault();
                  domEvent.stopPropagation();

                  if (key === 'edit') {
                    setEditingId(record.id);
                    setEditedRow(record);
                  }
                },
              }}
            >
              <Button icon={<MoreOutlined />} style={{ borderRadius: 6 }} />
            </Dropdown>
          </div>
        );
      },
    },
  ];

  // ======================================================================
  // =============================== Render ===============================
  // ======================================================================

  return (
    <div className="p-4 sm:p-6">
      <Title level={4}>Assignments</Title>
      <Text className="block mb-4 text-gray-500 dark:text-gray-400">
        Manage all assignments for this module below.
      </Text>

      <TableControlBar
        handleSearch={setSearchTerm}
        searchTerm={searchTerm}
        handleAdd={handleAddAssignment}
        addButtonText="New Assignment"
        handleBulkDelete={handleBulkDelete}
        selectedRowKeys={selectedRowKeys}
        clearMenuItems={[
          { key: 'clear-search', label: 'Clear Search', onClick: clearSearch },
          { key: 'clear-sort', label: 'Clear Sort', onClick: clearSorters },
          { key: 'clear-filters', label: 'Clear Filters', onClick: clearFilters },
          { key: 'clear-all', label: 'Clear All', onClick: clearAll },
        ]}
      />

      <TableCreateModal
        open={isAddModalOpen}
        onCancel={() => setIsAddModalOpen(false)}
        onCreate={(values) => {
          setNewAssignment(values); // ensure latest state just in case
          handleSubmitNewAssignment(values); // pass the up-to-date form values
        }}
        title="Create Assignment"
        fields={[
          {
            label: 'Assignment Name',
            name: 'name',
            type: 'text',
            required: true,
            placeholder: 'e.g., Assignment 1',
          },
          {
            label: 'Type',
            name: 'assignment_type',
            type: 'select',
            required: true,
            options: ASSIGNMENT_TYPES.map((t) => ({ label: t, value: t })),
          },
          {
            label: 'Available From',
            name: 'available_from',
            type: 'datetime',
            required: true,
          },
          {
            label: 'Due Date',
            name: 'due_date',
            type: 'datetime',
            required: true,
          },
        ]}
        initialValues={newAssignment}
        onChange={(values) => setNewAssignment(values)}
      />

      <TableTagSummary
        searchTerm={searchTerm}
        onClearSearch={clearSearch}
        filters={filterState}
        onClearFilter={(key) => {
          const updated = { ...filterState };
          delete updated[key];
          setFilterState(updated);
        }}
        sorters={sorterState.map((s) => ({ columnKey: s.field, order: s.order }))}
        onClearSorter={(key) => setSorterState(sorterState.filter((s) => s.field !== key))}
      />

      <Table<Assignment>
        columns={columns}
        dataSource={assignments}
        rowKey="id"
        loading={loading}
        rowSelection={{
          selectedRowKeys,
          onChange: setSelectedRowKeys,
        }}
        pagination={{
          ...pagination,
          showSizeChanger: true,
          showQuickJumper: true,
          onChange: (page, pageSize) => setPagination({ current: page, pageSize }),
        }}
        onChange={(pagination, filters, sorter) => {
          const sorterArray = (Array.isArray(sorter) ? sorter : [sorter])
            .filter(
              (s): s is { columnKey: string; order: 'ascend' | 'descend' } =>
                !!s.columnKey && !!s.order,
            )
            .map((s) => ({
              field: String(s.columnKey),
              order: s.order,
            }));

          setSorterState(sorterArray);
          setFilterState(filters as Record<string, string[]>);
          setPagination({
            current: pagination.current || 1,
            pageSize: pagination.pageSize || 10,
          });
        }}
        onRow={(record) => ({
          onClick: () => {
            if (editingId === null) {
              navigate(`/modules/${module.id}/assignments/${record.id}/submissions`);
            }
          },
        })}
        locale={{
          emptyText: (
            <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="No assignments found.">
              <Button
                icon={<ReloadOutlined />}
                onClick={() => {
                  clearAll();
                }}
              >
                Clear All Filters
              </Button>
            </Empty>
          ),
        }}
        className="border-1 border-gray-200 dark:border-gray-800 rounded-lg overflow-hidden"
      />
    </div>
  );
};

export default AssignmentsTable;
