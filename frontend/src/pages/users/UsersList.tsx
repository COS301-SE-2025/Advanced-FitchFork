import { Table, Space, Button, Input, Popconfirm, Select, Empty, Tooltip, Tag } from 'antd';
import {
  EditOutlined,
  DeleteOutlined,
  CheckOutlined,
  CloseOutlined,
  ReloadOutlined,
  EyeOutlined,
} from '@ant-design/icons';
import { useEffect, useState } from 'react';
import type { ColumnsType } from 'antd/es/table';
import { UsersService } from '@/services/users';
import type { User } from '@/types/users';
import type { SortOption } from '@/types/common';
import { useTableQuery } from '@/hooks/useTableQuery';
import TableControlBar from '@/components/TableControlBar';
import TableTagSummary from '@/components/TableTagSummary';
import AppLayout from '@/layouts/AppLayout';
import TableCreateModal from '@/components/TableCreateModal';
import { AuthService } from '@/services/auth';
import { useNotifier } from '@/components/Notifier';
import { useNavigate } from 'react-router-dom';

export default function UsersList() {
  // ======================================================================
  // =========================== State & Hooks ============================
  // ======================================================================
  const navigate = useNavigate();

  const { notifySuccess, notifyError } = useNotifier();
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

  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(false);
  const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([]);
  const [editingRowId, setEditingRowId] = useState<number | null>(null);
  const [editCache, setEditCache] = useState<Partial<User>>({});
  const [isAddModalOpen, setIsAddModalOpen] = useState(false);
  const [newUser, setNewUser] = useState({
    student_number: '',
    email: '',
    admin: false,
  });

  // ======================================================================
  // ============================ Fetch Users =============================
  // ======================================================================

  const fetchUsers = async () => {
    setLoading(true);

    const sort: SortOption[] = sorterState.map(({ field, order }) => ({ field, order }));

    const res = await UsersService.listUsers({
      page: pagination.current,
      per_page: pagination.pageSize,
      query: searchTerm || undefined,
      email: filterState.email?.[0],
      student_number: filterState.student_number?.[0],
      admin:
        filterState.admin?.[0] === 'true'
          ? true
          : filterState.admin?.[0] === 'false'
            ? false
            : undefined,

      sort,
    });

    if (res.success) {
      setUsers(res.data.users);
      setPagination({ total: res.data.total });
    } else {
      notifyError('Failed to fetch users');
    }

    setLoading(false);
  };

  useEffect(() => {
    fetchUsers();
  }, [searchTerm, filterState, sorterState, pagination.current, pagination.pageSize]);

  // ======================================================================
  // ============================= Add User ===============================
  // ======================================================================

  const handleAddUser = () => {
    setNewUser({ student_number: '', email: '', admin: false });
    setIsAddModalOpen(true);
  };

  const handleSubmitNewUser = async (formValues: Record<string, any>) => {
    const { student_number, email } = formValues;

    try {
      const res = await AuthService.register({
        student_number,
        email,
        password: 'changeme123',
      });

      if (res.success) {
        notifySuccess('User registered successfully', res.message);
        setIsAddModalOpen(false);
        fetchUsers();
      } else {
        notifyError(res.message || 'Failed to register user', res.message);
      }
    } catch (err) {
      notifyError('Registration failed', 'Registration failed due to network/server error');
    }
  };

  // ======================================================================
  // ======================== Edit & Delete User ==========================
  // ======================================================================

  const handleEditSave = async () => {
    const user = users.find((u) => u.id === editingRowId);
    if (!user || !editCache.email || !editCache.student_number) {
      notifyError('Incomplete user info');
      return;
    }

    const updated: User = {
      ...user,
      ...editCache,
      updated_at: new Date().toISOString(),
    };

    const res = await UsersService.editUser(updated.id, updated);
    if (res.success) {
      notifySuccess('User updated successfully', res.message);
      setUsers((prev) => prev.map((u) => (u.id === updated.id ? res.data : u)));
      setEditingRowId(null);
      setEditCache({});
    } else {
      notifyError('Failed to update user', res.message);
    }
  };

  const handleDelete = async (id: number) => {
    const res = await UsersService.deleteUser(id);
    if (res.success) {
      setUsers((prev) => prev.filter((u) => u.id !== id));
      setSelectedRowKeys((prev) => prev.filter((k) => k !== id));
      notifySuccess('User deleted successfully', res.message);
    } else {
      notifyError('Failed to delete user', res.message);
    }
  };

  const handleBulkDelete = async () => {
    for (const id of selectedRowKeys) {
      await handleDelete(Number(id));
    }
    setSelectedRowKeys([]);
    if (selectedRowKeys.length > 0) {
      notifySuccess(
        'Selected users deleted',
        `${selectedRowKeys.length} user(s) have been deleted.`,
      );
    }
  };

  // ======================================================================
  // =========================== Table Columns ============================
  // ======================================================================

  const columns: ColumnsType<User> = [
    {
      title: 'Student Number',
      dataIndex: 'student_number',
      key: 'student_number',
      render: (_, record) =>
        editingRowId === record.id ? (
          <Input
            value={editCache.student_number}
            onChange={(e) => setEditCache((prev) => ({ ...prev, student_number: e.target.value }))}
          />
        ) : (
          record.student_number
        ),
      sorter: { multiple: 1 },
      sortOrder: sorterState.find((s) => s.field === 'student_number')?.order ?? null,
      filteredValue: filterState.student_number || null,
      onFilter: () => true,
      filterDropdown: ({ setSelectedKeys, selectedKeys, confirm, clearFilters }) => (
        <div className="flex flex-col gap-2 p-2 w-56">
          <Input
            placeholder="Search student number"
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
      title: 'Email',
      dataIndex: 'email',
      key: 'email',
      render: (_, record) =>
        editingRowId === record.id ? (
          <Input
            value={editCache.email}
            onChange={(e) => setEditCache((prev) => ({ ...prev, email: e.target.value }))}
          />
        ) : (
          <a href={`mailto:${record.email}`}>{record.email}</a>
        ),
      sorter: { multiple: 2 },
      sortOrder: sorterState.find((s) => s.field === 'email')?.order ?? null,
      filteredValue: filterState.email || null,
      onFilter: () => true,
      filterDropdown: ({ setSelectedKeys, selectedKeys, confirm, clearFilters }) => (
        <div className="flex flex-col gap-2 p-2 w-56">
          <Input
            placeholder="Search email"
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
      title: 'Admin',
      dataIndex: 'admin',
      key: 'admin',
      filters: [
        { text: 'Admin', value: 'true' },
        { text: 'Regular', value: 'false' },
      ],
      filteredValue: filterState.admin || null,
      sorter: { multiple: 3 },
      sortOrder: sorterState.find((s) => s.field === 'admin')?.order ?? null,
      onFilter: () => true,
      render: (_, record) =>
        editingRowId === record.id ? (
          <Select
            value={editCache.admin ? 'admin' : 'regular'}
            onChange={(val) => setEditCache((prev) => ({ ...prev, admin: val === 'admin' }))}
            style={{ width: 120 }}
          >
            <Select.Option value="admin">Admin</Select.Option>
            <Select.Option value="regular">Regular</Select.Option>
          </Select>
        ) : (
          <Tag color={record.admin ? 'green' : undefined}>{record.admin ? 'Admin' : 'Regular'}</Tag>
        ),
    },
    {
      title: 'Actions',
      key: 'actions',
      align: 'right',
      width: 120,
      render: (_, record) =>
        editingRowId === record.id ? (
          <Space>
            <Tooltip title="Save">
              <Button
                icon={<CheckOutlined />}
                type="primary"
                shape="circle"
                onClick={handleEditSave}
                size="small"
              />
            </Tooltip>
            <Tooltip title="Cancel">
              <Button
                icon={<CloseOutlined />}
                shape="circle"
                onClick={() => setEditingRowId(null)}
                size="small"
              />
            </Tooltip>
          </Space>
        ) : (
          <Space>
            <Tooltip title="View">
              <Button
                icon={<EyeOutlined />}
                size="small"
                type="text"
                onClick={() => navigate(`/users/${record.id}`)}
              />
            </Tooltip>
            <Tooltip title="Edit">
              <Button
                icon={<EditOutlined />}
                size="small"
                type="text"
                onClick={() => {
                  setEditingRowId(record.id);
                  setEditCache(record);
                }}
              />
            </Tooltip>

            <Tooltip title="Delete">
              <Popconfirm
                title="Delete this user?"
                onConfirm={() => handleDelete(record.id)}
                okText="Yes"
                cancelText="No"
              >
                <Button icon={<DeleteOutlined />} danger type="text" size="small" />
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
    <AppLayout title="Users" description="Manage all registered users in the system.">
      <TableControlBar
        handleSearch={setSearchTerm}
        searchTerm={searchTerm}
        handleAdd={handleAddUser}
        addButtonText="Add User"
        handleBulkDelete={handleBulkDelete}
        selectedRowKeys={selectedRowKeys}
        clearMenuItems={[
          { key: 'clear-search', label: 'Clear Search', onClick: clearSearch },
          { key: 'clear-sort', label: 'Clear Sort', onClick: clearSorters },
          { key: 'clear-filters', label: 'Clear Filters', onClick: clearFilters },
          { key: 'clear-all', label: 'Clear All', onClick: clearAll },
        ]}
        searchPlaceholder="Search email or student number"
        bulkDeleteConfirmMessage="Delete selected users?"
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

      <Table<User>
        rowKey="id"
        columns={columns}
        dataSource={users}
        rowSelection={{
          selectedRowKeys,
          onChange: setSelectedRowKeys,
        }}
        loading={loading}
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
          setFilterState({ ...filters } as Record<string, string[]>);

          setPagination({
            current: pagination.current ?? 1,
            pageSize: pagination.pageSize ?? 10,
          });
        }}
        locale={{
          emptyText: (
            <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="No users found.">
              <Button icon={<ReloadOutlined />} onClick={clearAll}>
                Clear All Filters
              </Button>
            </Empty>
          ),
        }}
        className="border-1 border-gray-200 dark:border-gray-800 rounded-lg overflow-hidden"
      />

      <TableCreateModal
        open={isAddModalOpen}
        onCancel={() => setIsAddModalOpen(false)}
        onCreate={handleSubmitNewUser} // now gets the modal form values
        title="Add User"
        fields={[
          { name: 'student_number', label: 'Student Number', type: 'text', required: true },
          { name: 'email', label: 'Email', type: 'email', required: true },
        ]}
        initialValues={newUser}
      />
    </AppLayout>
  );
}
