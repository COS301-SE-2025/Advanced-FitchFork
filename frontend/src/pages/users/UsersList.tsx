import {
  Table,
  Tag,
  Space,
  Button,
  Input,
  Popconfirm,
  message,
  Select,
  Dropdown,
  Card,
  Statistic,
  Empty,
} from 'antd';
import type { MenuProps, TableColumnsType, TablePaginationConfig, TableProps } from 'antd';
import { ReloadOutlined, EditOutlined, DeleteOutlined } from '@ant-design/icons';
import { useState } from 'react';
import type { FilterValue } from 'antd/es/table/interface';
import AppLayout from '@/layouts/AppLayout';
import { mockUsers } from '@/mocks/users';
import type { User } from '@/types/users';
import TableTagSummary from '@/components/TableTagSummary';

const { Search } = Input;

const UsersList: React.FC = () => {
  const [users, setUsers] = useState<User[]>(mockUsers);
  const [searchTerm, setSearchTerm] = useState('');
  const [filteredUsers, setFilteredUsers] = useState<User[]>(mockUsers);
  const [editingRowId, setEditingRowId] = useState<number | null>(null);
  const [editCache, setEditCache] = useState<Partial<User>>({});
  const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([]);
  const [filterState, setFilterState] = useState<Record<string, FilterValue | null>>({});
  const [sorterState, setSorterState] = useState<any[]>([]);
  const [pagination, setPagination] = useState<TablePaginationConfig>({
    current: 1,
    pageSize: 5,
    pageSizeOptions: ['5', '10', '20', '50', '100'],
    showSizeChanger: true,
    showQuickJumper: true,
    showTotal: (total, range) => `${range[0]}-${range[1]} of ${total} users`,
    style: {
      paddingRight: '20px',
    },
  });

  const handleSearch = (value: string) => {
    setSearchTerm(value);
    const lower = value.toLowerCase();
    const filtered = users.filter(
      (user) =>
        user.email.toLowerCase().includes(lower) ||
        user.student_number.toLowerCase().includes(lower),
    );
    setFilteredUsers(filtered);
    setPagination((prev) => ({ ...prev, current: 1 }));
  };

  const clearFilters = () => {
    setFilterState({});
    setFilteredUsers(users);
  };

  const clearSorts = () => {
    setSorterState([]);
  };

  const handleEdit = (record: User) => {
    setEditingRowId(record.id);
    setEditCache({ ...record });
  };

  const handleEditSave = () => {
    if (!editCache.student_number || !editCache.email) {
      message.error('All fields must be filled');
      return;
    }
    const updatedUsers = users.map((user) =>
      user.id === editingRowId ? { ...user, ...editCache } : user,
    );
    setUsers(updatedUsers);
    setFilteredUsers(updatedUsers);
    setEditingRowId(null);
    setEditCache({});
    message.success('User updated');
  };

  const handleDelete = (id: number) => {
    const updated = users.filter((u) => u.id !== id);
    setUsers(updated);
    setFilteredUsers(updated);
    message.success('User deleted');
  };

  const handleBulkDelete = () => {
    const updated = users.filter((u) => !selectedRowKeys.includes(u.id));
    setUsers(updated);
    setFilteredUsers(updated);
    setSelectedRowKeys([]);
    message.success('Selected users deleted');
  };

  const columns: TableColumnsType<User> = [
    {
      title: 'Student Number',
      dataIndex: 'student_number',
      sorter: {
        compare: (a, b) => a.student_number.localeCompare(b.student_number),
        multiple: 2,
      },
      sortOrder: sorterState.find((s) => s.columnKey === 'student_number')?.order || null,
      key: 'student_number',
      filterDropdown: ({ setSelectedKeys, selectedKeys, confirm, clearFilters }) => (
        <div className="flex flex-col gap-2 p-2 space-y-2 w-56">
          <Input
            placeholder="Search student number"
            value={selectedKeys[0]}
            onChange={(e) => setSelectedKeys([e.target.value])}
            onPressEnter={() => confirm()}
            className=""
          />
          <div className="flex justify-between gap-2">
            <Button type="primary" onClick={() => confirm()} size="small">
              Filter
            </Button>
            <Button onClick={() => clearFilters?.()} size="small">
              Reset
            </Button>
          </div>
        </div>
      ),
      onFilter: (value, record) =>
        typeof value === 'string' &&
        record.student_number.toLowerCase().includes(value.toLowerCase()),
      filteredValue: filterState?.student_number || null,

      render: (_, record) =>
        editingRowId === record.id ? (
          <Input
            value={editCache.student_number}
            onChange={(e) => setEditCache({ ...editCache, student_number: e.target.value })}
          />
        ) : (
          record.student_number
        ),
    },
    {
      title: 'Email',
      dataIndex: 'email',
      sorter: {
        compare: (a, b) => a.email.localeCompare(b.email),
        multiple: 1,
      },
      sortOrder: sorterState.find((s) => s.columnKey === 'email')?.order || null,
      key: 'email',
      render: (_, record) =>
        editingRowId === record.id ? (
          <Input
            value={editCache.email}
            onChange={(e) => setEditCache({ ...editCache, email: e.target.value })}
          />
        ) : (
          <a href={`mailto:${record.email}`} className="text-blue-600 hover:underline">
            {record.email}
          </a>
        ),
    },
    {
      title: 'Admin',
      dataIndex: 'is_admin',
      key: 'is_admin',
      render: (_, record) =>
        editingRowId === record.id ? (
          <Select
            value={editCache.is_admin ? 'admin' : 'regular'}
            onChange={(value) => setEditCache({ ...editCache, is_admin: value === 'admin' })}
            style={{ width: 120 }}
          >
            <Select.Option value="admin">Admin</Select.Option>
            <Select.Option value="regular">Regular</Select.Option>
          </Select>
        ) : (
          <Tag color={record.is_admin ? 'green' : undefined}>
            {record.is_admin ? 'Admin' : 'Regular'}
          </Tag>
        ),
      filters: [
        { text: 'Admin', value: true },
        { text: 'Regular', value: false },
      ],
      filteredValue: filterState?.is_admin || null,
      onFilter: (value, record) => record.is_admin === value,
      sorter: {
        compare: (a, b) => Number(a.is_admin) - Number(b.is_admin),
        multiple: 3,
      },
      sortOrder: sorterState.find((s) => s.columnKey === 'is_admin')?.order || null,
    },
    {
      title: 'Actions',
      key: 'actions',
      render: (_, record) =>
        editingRowId === record.id ? (
          <Space>
            <Button type="primary" size="small" onClick={handleEditSave}>
              Save
            </Button>
            <Button size="small" onClick={() => setEditingRowId(null)}>
              Cancel
            </Button>
          </Space>
        ) : (
          <Space>
            <Button icon={<EditOutlined />} size="small" onClick={() => handleEdit(record)} />
            <Popconfirm
              title="Delete this user?"
              onConfirm={() => handleDelete(record.id)}
              okText="Yes"
              cancelText="No"
            >
              <Button icon={<DeleteOutlined />} danger size="small" />
            </Popconfirm>
          </Space>
        ),
    },
  ];

  const onChange: TableProps<User>['onChange'] = (pagination, filters, sorter) => {
    setPagination(pagination);
    setFilterState(filters);
    const sorterArray = Array.isArray(sorter)
      ? sorter
      : sorter && sorter.columnKey && sorter.order
        ? [sorter]
        : [];
    setSorterState(sorterArray);
  };

  const handleAddUser = () => {
    const newUser: User = {
      id: Date.now(), // temporary unique ID
      student_number: 'u00000000',
      email: 'new.user@up.ac.za',
      is_admin: false,
      module_roles: [],
    };
    setUsers([newUser, ...users]);
    setFilteredUsers([newUser, ...filteredUsers]);
    setEditingRowId(newUser.id);
    setEditCache({ ...newUser });
  };

  const clearAllFilters = () => {
    setFilterState({});
    setSorterState([]);
    setSearchTerm('');
    setFilteredUsers(users);
    setPagination((prev) => ({ ...prev, current: 1 }));
  };

  const rowSelection: TableProps<User>['rowSelection'] = {
    selectedRowKeys,
    onChange: (selectedKeys) => setSelectedRowKeys(selectedKeys),
  };

  const clearMenuItems: MenuProps['items'] = [
    {
      key: 'clear-column-filters',
      label: 'Clear Column Filters',
      onClick: clearFilters,
    },
    {
      key: 'clear-all',
      label: 'Clear All Filters',
      onClick: clearAllFilters,
    },
    {
      key: 'clear-sorts',
      label: 'Clear Sorts',
      onClick: clearSorts,
    },
  ];

  return (
    <AppLayout title="Users" description="A list of all the users.">
      {/* User Summary Stats */}
      <div className="mb-6 flex flex-wrap gap-4">
        <Card className="flex-1 min-w-[200px]">
          <Statistic title="Total Users" value={128} />
        </Card>
        <Card className="flex-1 min-w-[200px]">
          <Statistic title="Admins" value={12} />
        </Card>
        <Card className="flex-1 min-w-[200px]">
          <Statistic title="Regular Users" value={116} />
        </Card>
      </div>
      {/* Control Bar */}
      <div className="mb-4 flex flex-wrap items-center justify-between gap-4">
        <Search
          placeholder="Search student number or email"
          allowClear
          onChange={(e) => handleSearch(e.target.value)}
          value={searchTerm}
          style={{ maxWidth: 320 }}
          className="w-full sm:w-auto"
        />
        <div className="flex flex-wrap gap-2 items-center">
          <Button type="primary" onClick={handleAddUser}>
            Add User
          </Button>

          <Dropdown menu={{ items: clearMenuItems }} placement="bottomRight" trigger={['click']}>
            <Button icon={<ReloadOutlined />}>Clear Options</Button>
          </Dropdown>

          {selectedRowKeys.length > 0 && (
            <Popconfirm
              title="Delete selected users?"
              onConfirm={handleBulkDelete}
              okText="Yes"
              cancelText="No"
            >
              <Button danger icon={<DeleteOutlined />}>
                Delete Selected
              </Button>
            </Popconfirm>
          )}
        </div>
      </div>

      <TableTagSummary
        searchTerm={searchTerm}
        onClearSearch={() => {
          setSearchTerm('');
          setFilteredUsers(users);
        }}
        filters={filterState}
        onClearFilter={(key) =>
          setFilterState((prev) => {
            const updated = { ...prev };
            delete updated[key];
            return updated;
          })
        }
        sorters={sorterState}
        onClearSorter={(key) =>
          setSorterState((prev) =>
            Array.isArray(prev) ? prev.filter((s) => s.columnKey !== key) : [],
          )
        }
      />

      {/* Table */}
      <Table<User>
        rowKey="id"
        columns={columns}
        dataSource={filteredUsers}
        rowSelection={rowSelection}
        pagination={pagination}
        onChange={onChange}
        locale={{
          emptyText: (
            <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="No users found.">
              <Button
                icon={<ReloadOutlined />}
                onClick={() => {
                  clearAllFilters();
                }}
              >
                Clear All Filters
              </Button>
            </Empty>
          ),
        }}
        className="border-1 border-gray-100 dark:border-gray-800 rounded-lg"
      />
    </AppLayout>
  );
};

export default UsersList;
