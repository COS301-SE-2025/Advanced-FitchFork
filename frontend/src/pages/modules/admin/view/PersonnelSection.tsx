import {
  Typography,
  Table,
  Button,
  Space,
  Popconfirm,
  Tooltip,
  Input,
  Select,
  Tag,
  Empty,
} from 'antd';
import {
  EditOutlined,
  DeleteOutlined,
  CheckOutlined,
  CloseOutlined,
  ReloadOutlined,
} from '@ant-design/icons';
import { useEffect, useState } from 'react';
import type { ColumnsType } from 'antd/es/table';
import { ModulesService } from '@/services/modules/mock'; // TODO: Remeber to change this when services work
import { UsersService } from '@/services/users';
import TableControlBar from '@/components/TableControlBar';
import TableTagSummary from '@/components/TableTagSummary';
import TableCreateModal from '@/components/TableCreateModal';
import type { ModuleUser, User } from '@/types/users';
import { useTableQuery } from '@/hooks/useTableQuery';
import type { ModuleRole } from '@/types/modules';
import { useNotifier } from '@/components/Notifier';

const { Title, Text } = Typography;
const ROLES = ['Lecturer', 'Tutor', 'Student'] as const;

interface Props {
  moduleId: number;
}

export default function PersonnelSection({ moduleId }: Props) {
  // ======================================================================
  // =========================== State & Hooks ============================
  // ======================================================================

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

  const { notifySuccess, notifyError } = useNotifier();

  const [loading, setLoading] = useState(true);
  const [personnel, setPersonnel] = useState<ModuleUser[]>([]);
  const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([]);
  const [editingId, setEditingId] = useState<number | null>(null);
  const [editedRow, setEditedRow] = useState<Partial<ModuleUser>>({});
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [allUsers, setAllUsers] = useState<User[]>([]);

  // ======================================================================
  // ============================ Data Fetching ===========================
  // ======================================================================

  const fetchPersonnel = async () => {
    setLoading(true);
    const [lect, tut, stud] = await Promise.all([
      ModulesService.getLecturers(moduleId),
      ModulesService.getTutors(moduleId),
      ModulesService.getStudents(moduleId),
    ]);

    if (lect.success && tut.success && stud.success) {
      const all = [
        ...lect.data.users.map((u) => ({ ...u, role: 'Lecturer' as ModuleRole })),
        ...tut.data.users.map((u) => ({ ...u, role: 'Tutor' as ModuleRole })),
        ...stud.data.users.map((u) => ({ ...u, role: 'Student' as ModuleRole })),
      ];
      const filtered = all.filter(
        (p) =>
          p.email.toLowerCase().includes(searchTerm.toLowerCase()) ||
          p.student_number.toLowerCase().includes(searchTerm.toLowerCase()),
      );

      const start = ((pagination.current || 1) - 1) * (pagination.pageSize || 10);
      const paginated = filtered.slice(start, start + (pagination.pageSize || 10));

      setPersonnel(paginated);
      setPagination({ total: filtered.length });
    } else {
      notifyError('Failed to load personnel');
    }

    setLoading(false);
  };

  useEffect(() => {
    fetchPersonnel();
  }, [moduleId, pagination.current, pagination.pageSize, sorterState, searchTerm, filterState]);

  useEffect(() => {
    if (isAddModalOpen) {
      UsersService.listUsers({ page: 1, per_page: 100 }).then((res) => {
        if (res.success) {
          setAllUsers(res.data.users);
        } else {
          notifyError('Failed to load users');
        }
      });
    }
  }, [isAddModalOpen]);

  // ======================================================================
  // ============================== Handlers ==============================
  // ======================================================================

  const handleSaveEdit = (_id: number) => {
    notifySuccess('Updated locally');
    setEditingId(null);
    setEditedRow({});
    fetchPersonnel();
  };

  const handleDeleteUser = (id: number) => {
    setPersonnel((prev) => prev.filter((u) => u.id !== id));
    setSelectedRowKeys((prev) => prev.filter((k) => k !== id));
    notifySuccess('User removed from module');
  };

  const handleBulkDelete = () => {
    for (const key of selectedRowKeys) {
      handleDeleteUser(Number(key));
    }
    setSelectedRowKeys([]);
  };

  const handleAddPerson = () => {
    clearAll();
    setIsAddModalOpen(true);
  };

  const handleSubmitNewUser = async (values: Record<string, any>) => {
    const userId = Number(values.user_id);
    const res = await ModulesService.enrollStudents(moduleId, { user_ids: [userId] });
    if (res.success) {
      notifySuccess('User added to module');
      setIsAddModalOpen(false);
      fetchPersonnel();
    } else {
      notifyError('Failed to enroll user');
    }
  };

  // ======================================================================
  // =========================== Table Columns ============================
  // ======================================================================

  const columns: ColumnsType<ModuleUser> = [
    {
      title: 'Email',
      dataIndex: 'email',
      key: 'email',
      sorter: { multiple: 1 },
      sortOrder: sorterState.find((s) => s.field === 'email')?.order ?? null,
      render: (_, record) =>
        editingId === record.id ? (
          <Input
            value={editedRow.email}
            onChange={(e) => setEditedRow((r) => ({ ...r, email: e.target.value }))}
          />
        ) : (
          record.email
        ),
    },
    {
      title: 'Student Number',
      dataIndex: 'student_number',
      key: 'student_number',
      sorter: { multiple: 2 },
      sortOrder: sorterState.find((s) => s.field === 'student_number')?.order ?? null,
      render: (_, record) =>
        editingId === record.id ? (
          <Input
            value={editedRow.student_number}
            onChange={(e) => setEditedRow((r) => ({ ...r, student_number: e.target.value }))}
          />
        ) : (
          record.student_number
        ),
    },
    {
      title: 'Role',
      dataIndex: 'role',
      key: 'role',
      sorter: { multiple: 3 },
      sortOrder: sorterState.find((s) => s.field === 'role')?.order ?? null,
      filters: ROLES.map((r) => ({ text: r, value: r })),
      filteredValue: filterState.role || null,
      onFilter: (val, rec) => rec.role === val,
      render: (_, record) =>
        editingId === record.id ? (
          <Select
            value={editedRow.role}
            onChange={(val) => setEditedRow((r) => ({ ...r, role: val }))}
            options={ROLES.map((r) => ({ label: r, value: r }))}
            style={{ width: 120 }}
          />
        ) : (
          <Tag
            color={
              record.role === 'Lecturer' ? 'volcano' : record.role === 'Tutor' ? 'blue' : 'green'
            }
          >
            {record.role}
          </Tag>
        ),
    },
    {
      title: 'Actions',
      key: 'actions',
      align: 'right',
      width: 120,
      render: (_, record) =>
        editingId === record.id ? (
          <Space>
            <Button
              icon={<CheckOutlined />}
              type="text"
              onClick={() => handleSaveEdit(record.id)}
            />
            <Button
              icon={<CloseOutlined />}
              type="text"
              danger
              onClick={() => {
                setEditingId(null);
                setEditedRow({});
              }}
            />
          </Space>
        ) : (
          <Space>
            <Tooltip title="Edit">
              <Button
                type="text"
                icon={<EditOutlined />}
                size="small"
                onClick={() => {
                  setEditingId(record.id);
                  setEditedRow(record);
                }}
              />
            </Tooltip>
            <Tooltip title="Delete">
              <Popconfirm
                title="Remove this user?"
                onConfirm={() => handleDeleteUser(record.id)}
                okText="Yes"
                cancelText="No"
              >
                <Button type="text" icon={<DeleteOutlined />} danger size="small" />
              </Popconfirm>
            </Tooltip>
          </Space>
        ),
    },
  ];

  // ======================================================================
  // ============================== Render ================================
  // ======================================================================

  return (
    <div>
      <Title level={4}>Module Personnel</Title>
      <Text className="block mb-4 text-gray-500 dark:text-gray-400">
        All the Lecturers, Tutors and Students of the module.
      </Text>

      <TableControlBar
        handleSearch={setSearchTerm}
        searchTerm={searchTerm}
        handleAdd={handleAddPerson}
        addButtonText="Add Person"
        handleBulkDelete={handleBulkDelete}
        selectedRowKeys={selectedRowKeys}
        clearMenuItems={[
          { key: 'clear-search', label: 'Clear Search', onClick: clearSearch },
          { key: 'clear-sort', label: 'Clear Sorters', onClick: clearSorters },
          { key: 'clear-filters', label: 'Clear Filters', onClick: clearFilters },
          { key: 'clear-all', label: 'Clear All', onClick: clearAll },
        ]}
      />

      <TableCreateModal
        open={isAddModalOpen}
        onCancel={() => setIsAddModalOpen(false)}
        onCreate={handleSubmitNewUser}
        title="Assign User to Module"
        fields={[
          {
            label: 'Select User',
            name: 'user_id',
            type: 'select',
            required: true,
            options: allUsers.map((u) => ({
              value: String(u.id),
              label: `${u.student_number} — ${u.email}`,
            })),
          },
        ]}
        initialValues={{ user_id: '' }}
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

      <Table<ModuleUser>
        columns={columns}
        dataSource={personnel}
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
        locale={{
          emptyText: (
            <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="No personnel found.">
              <Button icon={<ReloadOutlined />} onClick={clearAll}>
                Clear All Filters
              </Button>
            </Empty>
          ),
        }}
        className="border-1 border-gray-200 dark:border-gray-800 rounded-lg overflow-hidden"
      />
    </div>
  );
}
