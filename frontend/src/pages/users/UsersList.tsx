import {
  Table,
  Space,
  Button,
  Input,
  Popconfirm,
  Select,
  Empty,
  Tooltip,
  Tag,
  Dropdown,
} from 'antd';
import {
  EditOutlined,
  DeleteOutlined,
  CheckOutlined,
  CloseOutlined,
  ReloadOutlined,
  EyeOutlined,
  MoreOutlined,
} from '@ant-design/icons';
import { useEffect, useState } from 'react';
import type { ColumnsType } from 'antd/es/table';
import type { User } from '@/types/users';
import type { SortOption } from '@/types/common';
import { useTableQuery } from '@/hooks/useTableQuery';
import TableControlBar from '@/components/TableControlBar';
import TableTagSummary from '@/components/TableTagSummary';
import TableCreateModal from '@/components/TableCreateModal';
import { useNotifier } from '@/components/Notifier';
import { useNavigate } from 'react-router-dom';
import PageHeader from '@/components/PageHeader';
import { listUsers, editUser, deleteUser } from '@/services/users';
import { register } from '@/services/auth';

const UsersList = () => {
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
    username: '',
    email: '',
    admin: false,
  });

  // ======================================================================
  // ============================ Fetch Users =============================
  // ======================================================================

  const fetchUsers = async () => {
    setLoading(true);

    const sort: SortOption[] = sorterState.map(({ field, order }) => ({ field, order }));

    const res = await listUsers({
      page: pagination.current,
      per_page: pagination.pageSize,
      query: searchTerm || undefined,
      email: filterState.email?.[0],
      username: filterState.username?.[0],
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
    setNewUser({ username: '', email: '', admin: false });
    setIsAddModalOpen(true);
  };

  const handleSubmitNewUser = async (formValues: Record<string, any>) => {
    const { username, email } = formValues;

    try {
      const res = await AuthService.register({
        username,
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
    if (!user || !editCache.email || !editCache.username) {
      notifyError('Incomplete user info');
      return;
    }

    const updated: User = {
      ...user,
      ...editCache,
      updated_at: new Date().toISOString(),
    };

    const res = await editUser(updated.id, updated);
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
    const res = await deleteUser(id);
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
      dataIndex: 'username',
      key: 'username',
      render: (_, record) =>
        editingRowId === record.id ? (
          <Input
            value={editCache.username}
            onChange={(e) => setEditCache((prev) => ({ ...prev, username: e.target.value }))}
          />
        ) : (
          record.username
        ),
      sorter: { multiple: 1 },
      sortOrder: sorterState.find((s) => s.field === 'username')?.order ?? null,
      filteredValue: filterState.username || null,
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
      width: 100,
      render: (_, record) => {
        if (editingRowId === record.id) {
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
                    handleEditSave();
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
                    setEditingRowId(null);
                  }}
                />
              </Tooltip>
            </Space>
          );
        }

        return (
          <div
            onClick={(e) => {
              e.stopPropagation(); // Prevent row navigation when interacting with the dropdown
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
                        title="Delete this user?"
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

                  if (key === 'view') navigate(`/users/${record.id}`);
                  else if (key === 'edit') {
                    setEditingRowId(record.id);
                    setEditCache(record);
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
        );
      },
    },
  ];

  // ======================================================================
  // ============================== Render ================================
  // ======================================================================

  return (
    <div className="bg-white dark:bg-gray-950 p-4 sm:p-6 h-full">
      <PageHeader title="Users" description="Manage all registered users in the system." />
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
        onRow={(record) => ({
          onClick: () => {
            if (editingRowId === null) {
              navigate(`/users/${record.id}`);
            }
          },
        })}
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
          { name: 'username', label: 'Student Number', type: 'text', required: true },
          { name: 'email', label: 'Email', type: 'email', required: true },
        ]}
        initialValues={newUser}
      />
    </div>
  );
};

export default UsersList;
