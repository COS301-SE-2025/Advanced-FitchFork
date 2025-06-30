import { Typography, Table, Button, Popconfirm, Tooltip, Input, Empty, Dropdown } from 'antd';
import {
  EditOutlined,
  DeleteOutlined,
  ReloadOutlined,
  MoreOutlined,
  EyeOutlined,
} from '@ant-design/icons';
import { useEffect, useState } from 'react';
import type { ColumnsType } from 'antd/es/table';
import type { SortOption } from '@/types/common';
import { useTableQuery } from '@/hooks/useTableQuery';
import dayjs from 'dayjs';
import weekday from 'dayjs/plugin/weekday';
import localeData from 'dayjs/plugin/localeData';
import isSameOrBefore from 'dayjs/plugin/isSameOrBefore';
import isSameOrAfter from 'dayjs/plugin/isSameOrAfter';
import { useNotifier } from '@/components/Notifier';
import { useModule } from '@/context/ModuleContext';
import { useNavigate } from 'react-router-dom';
import {
  listAssignments,
  createAssignment,
  editAssignment,
  deleteAssignment,
} from '@/services/modules/assignments';
import {
  type AssignmentType,
  type Assignment,
  type PostAssignmentRequest,
  type PutAssignmentRequest,
  ASSIGNMENT_TYPES,
} from '@/types/modules/assignments';
import ControlBar from '@/components/ControlBar';
import AssignmentCard from '@/components/assignments/AssignmentCard';
import EditModal from '@/components/EditModal';
import CreateModal from '@/components/CreateModal';
import TagSummary from '@/components/TagSummary';
import { getRandomAssignmentStatus } from '@/constants/mock/assignment';
import AssignmentTypeTag from '@/components/assignments/AssignmentTypeTag';
import AssignmentStatusTag from '@/components/assignments/AssignmentStatusTag';

dayjs.extend(weekday);
dayjs.extend(localeData);
dayjs.extend(isSameOrBefore);
dayjs.extend(isSameOrAfter);

const { Title, Text } = Typography;

const AssignmentsList = () => {
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
  const [editModalOpen, setEditModalOpen] = useState(false);
  const [editingAssignment, setEditingAssignment] = useState<Assignment | null>(null);
  const [loading, setLoading] = useState(false);
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [newAssignment, setNewAssignment] = useState<Partial<Assignment>>({
    name: 'New Assignment',
    assignment_type: 'assignment',
    available_from: dayjs().toISOString(),
    due_date: dayjs().toISOString(),
  });
  const [viewMode, setViewMode] = useState<'table' | 'grid'>('grid');
  const [, setWindowWidth] = useState(window.innerWidth);

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
      setAssignments(
        res.data.assignments.map((a) => ({
          ...a,
          status: getRandomAssignmentStatus(), // inject mock status
        })),
      );
      setPagination({ total: res.data.total });
    } else {
      notifyError('Fetch Failed', 'Could not load assignment data');
    }

    setLoading(false);
  };

  useEffect(() => {
    fetchAssignments();
  }, [module.id, pagination.current, pagination.pageSize, sorterState, searchTerm, filterState]);

  useEffect(() => {
    const handleResize = () => {
      const width = window.innerWidth;
      setWindowWidth(width);

      if (width < 640) {
        setViewMode('grid');
      } else {
        const stored = localStorage.getItem('assignments_view_mode');
        if (stored === 'table' || stored === 'grid') {
          setViewMode(stored);
        }
      }
    };

    window.addEventListener('resize', handleResize);
    handleResize(); // run once on mount

    return () => window.removeEventListener('resize', handleResize);
  }, []);

  // ======================================================================
  // ========================== Add Item Handlers =========================
  // ======================================================================

  const handleOpenAddModal = () => {
    clearAll();
    setNewAssignment({
      name: '',
      assignment_type: 'assignment',
      available_from: dayjs().toISOString(),
      due_date: dayjs().toISOString(),
    });
    setIsAddModalOpen(true);
  };

  const handleAdd = async (values: Partial<PostAssignmentRequest>) => {
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

  const saveEdit = async (values: Record<string, any>) => {
    if (!editingAssignment) return;

    const payload: PutAssignmentRequest = {
      name: values.name,
      assignment_type: values.assignment_type,
      available_from: dayjs(values.available_from).toISOString(),
      due_date: dayjs(values.due_date).toISOString(),
      description: values.description,
    };

    const res = await editAssignment(module.id, editingAssignment.id, payload);

    if (res.success) {
      notifySuccess('Updated', 'Assignment changes have been saved');
      setEditModalOpen(false);
      setEditingAssignment(null);
      fetchAssignments();
    } else {
      notifyError('Update Failed', 'Could not update the assignment');
    }
  };

  const handleDelete = async (assignmentId: number) => {
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
      await handleDelete(Number(key));
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
      render: (_, record) => <AssignmentTypeTag type={record.assignment_type} />,
    },
    {
      title: 'Available From',
      dataIndex: 'available_from',
      key: 'available_from',
      sorter: { multiple: 3 },
      sortOrder: sorterState.find((s) => s.field === 'available_from')?.order ?? null,
      render: (_, record) => dayjs(record.available_from).format('YYYY-MM-DD HH:mm'),
    },
    {
      title: 'Due Date',
      dataIndex: 'due_date',
      key: 'due_date',
      sorter: { multiple: 4 },
      sortOrder: sorterState.find((s) => s.field === 'due_date')?.order ?? null,
      render: (_, record) => dayjs(record.due_date).format('YYYY-MM-DD HH:mm'),
    },
    {
      title: 'Status',
      key: 'status',
      render: () => <AssignmentStatusTag status={getRandomAssignmentStatus()} />,
    },

    {
      title: 'Actions',
      key: 'actions',
      align: 'right',
      width: 100,
      render: (_, record) => (
        <div
          onClick={(e) => {
            e.stopPropagation(); // fully prevent row click on dropdown interaction
          }}
        >
          <Dropdown
            trigger={['click']}
            menu={{
              items: [
                {
                  key: 'view',
                  icon: <EyeOutlined />,
                  label: 'View',
                },
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
                      title="Delete this module?"
                      onConfirm={(e) => {
                        e?.stopPropagation();
                        handleDelete(record.id);
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

                if (key === 'view') navigate(`/modules/${record.id}`);
                else if (key === 'edit') {
                  setEditingAssignment(record);
                  setEditModalOpen(true);
                }
              },
            }}
          >
            <Button
              icon={<MoreOutlined />}
              style={{ borderRadius: 6 }}
              onClick={(e) => e.stopPropagation()}
            />
          </Dropdown>
        </div>
      ),
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

      <ControlBar
        handleSearch={setSearchTerm}
        searchTerm={searchTerm}
        viewMode={viewMode}
        onViewModeChange={(val) => {
          setViewMode(val);
          localStorage.setItem('assignments_view_mode', val);
        }}
        handleAdd={handleOpenAddModal}
        addButtonText="Add Assignment"
        handleBulkDelete={handleBulkDelete}
        selectedRowKeys={selectedRowKeys}
        clearMenuItems={[
          { key: 'clear-search', label: 'Clear Search', onClick: clearSearch },
          { key: 'clear-sort', label: 'Clear Sort', onClick: clearSorters },
          { key: 'clear-filters', label: 'Clear Filters', onClick: clearFilters },
          { key: 'clear-all', label: 'Clear All', onClick: clearAll },
        ]}
        searchPlaceholder="Search name or description"
        bulkDeleteConfirmMessage="Delete selected assignments?"
        {...(viewMode === 'grid' && {
          // ---- NEW: Sorting Options ----
          sortOptions: [
            { label: 'Name', field: 'name' },
            { label: 'Type', field: 'assignment_type' },
            { label: 'Available From', field: 'available_from' },
            { label: 'Due Date', field: 'due_date' },
          ],
          currentSort: sorterState.map((s) => `${s.field}.${s.order}`),
          onSortChange: (values) => {
            const parsed = values
              .map((val) => {
                const [field, order] = val.split('.');
                return { field, order } as SortOption & { order: 'ascend' | 'descend' };
              })
              .filter((s) => s.field && s.order);
            setSorterState(parsed);
          },
          // ---- NEW: Filter Groups ----
          filterGroups: [
            {
              key: 'assignment_type',
              label: 'Type',
              type: 'select',
              options: ASSIGNMENT_TYPES.map((t) => ({ label: t, value: t })),
            },
            {
              key: 'name',
              label: 'Name',
              type: 'text',
            },
          ],
          activeFilters: Object.entries(filterState)
            .flatMap(([key, val]) => val?.map((v) => `${key}:${v}`) ?? [])
            .filter(Boolean),
          onFilterChange: (values) => {
            const newFilters: Record<string, string[]> = {};
            values.forEach((v) => {
              const [key, val] = v.split(':');
              if (!newFilters[key]) newFilters[key] = [];
              newFilters[key].push(val);
            });
            setFilterState(newFilters);
          },
        })}
      />

      <TagSummary
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

      {viewMode === 'table' ? (
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
          size="middle"
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
              navigate(`/modules/${module.id}/assignments/${record.id}/submissions`);
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
          className="bg-white dark:bg-gray-950 border-1 border-gray-200 dark:border-gray-800 rounded-lg overflow-hidden"
        />
      ) : assignments.length === 0 ? (
        <Empty
          image={Empty.PRESENTED_IMAGE_SIMPLE}
          description="No assignments found."
          className="mt-10"
        >
          <Button icon={<ReloadOutlined />} onClick={clearAll}>
            Clear All Filters
          </Button>
        </Empty>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
          {assignments.map((assignment) => (
            <AssignmentCard
              key={assignment.id}
              assignment={assignment}
              actions={[
                <Tooltip title="View" key="view">
                  <EyeOutlined
                    onClick={(e) => {
                      e.stopPropagation();
                      navigate(`/modules/${assignment.id}`);
                    }}
                  />
                </Tooltip>,
                <Tooltip title="Edit" key="edit">
                  <EditOutlined
                    onClick={(e) => {
                      e.stopPropagation();
                      setEditingAssignment(assignment);
                      setEditModalOpen(true);
                    }}
                  />
                </Tooltip>,
                <Tooltip title="Delete" key="delete">
                  <Popconfirm
                    title="Delete this module?"
                    onConfirm={(e) => {
                      e?.stopPropagation();
                      handleDelete(assignment.id);
                    }}
                    onCancel={(e) => e?.stopPropagation()}
                    okText="Yes"
                    cancelText="No"
                  >
                    <DeleteOutlined onClick={(e) => e.stopPropagation()} />
                  </Popconfirm>
                </Tooltip>,
              ]}
            />
          ))}
        </div>
      )}
      <CreateModal
        open={isAddModalOpen}
        onCancel={() => setIsAddModalOpen(false)}
        onCreate={(values) => {
          setNewAssignment(values); // ensure latest state just in case
          handleAdd(values); // pass the up-to-date form values
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

      <EditModal
        open={editModalOpen}
        onCancel={() => setEditModalOpen(false)}
        onEdit={saveEdit}
        initialValues={editingAssignment ?? {}}
        onChange={(val) => setEditingAssignment({ ...editingAssignment!, ...val })}
        title="Edit Assignment"
        fields={[
          {
            name: 'name',
            label: 'Assignment Name',
            type: 'text',
            required: true,
          },
          {
            name: 'assignment_type',
            label: 'Type',
            type: 'select',
            required: true,
            options: ASSIGNMENT_TYPES.map((t) => ({ label: t, value: t })),
          },
          {
            name: 'available_from',
            label: 'Available From',
            type: 'datetime',
            required: true,
          },
          {
            name: 'due_date',
            label: 'Due Date',
            type: 'datetime',
            required: true,
          },
          {
            name: 'description',
            label: 'Description',
            type: 'textarea',
            required: false,
          },
        ]}
      />
    </div>
  );
};

export default AssignmentsList;
