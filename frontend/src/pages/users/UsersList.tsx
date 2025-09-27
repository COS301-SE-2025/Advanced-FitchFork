import { useState, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import PageHeader from '@/components/PageHeader';
import { message } from '@/utils/message';
import { EntityList, type EntityListHandle, type EntityListProps } from '@/components/EntityList';
import { DeleteOutlined, EditOutlined, PlusOutlined } from '@ant-design/icons';
import UserCard from '@/components/users/UserCard';
import UserAdminTag from '@/components/users/UserAdminTag';
import { listUsers, editUser, deleteUser } from '@/services/users';
import { createUser } from '@/services/users/post';
import type { CreateUserPayload, User } from '@/types/users';
import dayjs from 'dayjs';
import UserListItem from '@/components/users/UserListItem';
import { UsersEmptyState } from '@/components/users';
import FormModal, { type FormModalField } from '@/components/common/FormModal';

const userEditFields: FormModalField[] = [
  {
    name: 'username',
    label: 'Username',
    type: 'text',
    constraints: {
      required: true,
      length: { min: 3, max: 32 },
      pattern: { regex: /^[A-Za-z0-9_\-.]+$/, message: 'Only letters, numbers, _ - .' },
    },
  },
  { name: 'email', label: 'Email', type: 'email', constraints: { required: true, email: {} } },
  { name: 'admin', label: 'Admin', type: 'boolean' },
];

const userCreateFields: FormModalField[] = [
  {
    name: 'username',
    label: 'Username',
    type: 'text',
    constraints: {
      required: true,
      length: { min: 3, max: 32 },
      pattern: { regex: /^[A-Za-z0-9_\-.]+$/, message: 'Only letters, numbers, _ - .' },
    },
  },
  { name: 'email', label: 'Email', type: 'email', constraints: { required: true, email: {} } },
  {
    name: 'password',
    label: 'Password',
    type: 'password',
    constraints: {
      required: true,
      length: { min: 6, max: 128, messageMin: 'At least 6 characters' },
    },
  },
];

const UsersList = () => {
  const navigate = useNavigate();
  const listRef = useRef<EntityListHandle>(null);

  const [createOpen, setCreateOpen] = useState(false);
  const [editOpen, setEditOpen] = useState(false);
  const [editingItem, setEditingItem] = useState<User | null>(null);

  const fetchUsers = async ({
    page,
    per_page,
    query,
    sort,
    filters,
  }: {
    page: number;
    per_page: number;
    query?: string;
    sort: { field: string; order: 'ascend' | 'descend' }[];
    filters: Record<string, string[]>;
  }) => {
    const res = await listUsers({
      page,
      per_page,
      query,
      sort,
      username: filters.username?.[0],
      email: filters.email?.[0],
      admin:
        filters.admin?.[0] === 'true' ? true : filters.admin?.[0] === 'false' ? false : undefined,
    });

    if (res.success) {
      return {
        items: res.data.users,
        total: res.data.total,
      };
    }

    message.error(`Failed to fetch users: ${res.message}`);
    return { items: [], total: 0 };
  };

  const handleAddUser = async (values: Record<string, any>) => {
    const { username, email, password } = values; // ← no admin here

    const payload: CreateUserPayload = { username, email, password };
    const res = await createUser(payload);

    if (res.success) {
      message.success(res.message || 'User created successfully');
      setCreateOpen(false);
      listRef.current?.refresh();
    } else {
      message.error(`Failed to create user: ${res.message}`);
    }
  };

  const handleEditUser = async (values: Record<string, any>) => {
    if (!editingItem) return;

    const updated: User = {
      ...editingItem,
      ...values,
      admin: !!values.admin,
      updated_at: new Date().toISOString(),
    };

    const res = await editUser(editingItem.id, updated);
    if (res.success) {
      message.success(res.message || 'User updated successfully');
      setEditOpen(false);
      setEditingItem(null);
      listRef.current?.refresh();
    } else {
      message.error(`Failed to update user: ${res.message}`);
    }
  };

  const handleDeleteUser = async (user: User, refresh: () => void) => {
    const res = await deleteUser(user.id);
    if (res.success) {
      message.success(res.message || 'User deleted successfully');
      refresh();
    } else {
      message.error(`Failed to delete user: ${res.message}`);
    }
  };

  const actions: EntityListProps<User>['actions'] = {
    control: [
      {
        key: 'create',
        label: 'Add User',
        icon: <PlusOutlined />,
        isPrimary: true,
        handler: () => setCreateOpen(true),
      },
    ],
    entity: (user: User) => [
      {
        key: 'edit',
        label: 'Edit',
        icon: <EditOutlined />,
        handler: () => {
          setEditingItem(user);
          setEditOpen(true);
        },
      },
      {
        key: 'delete',
        label: 'Delete',
        icon: <DeleteOutlined />,
        confirm: true,
        handler: ({ refresh }) => handleDeleteUser(user, refresh),
      },
    ],
    bulk: [
      {
        key: 'bulk-delete',
        label: 'Bulk Delete',
        icon: <DeleteOutlined />,
        isPrimary: true,
        handler: ({ selected, refresh }) => {
          if (!selected || selected.length === 0) {
            message.warning('No users selected');
            return;
          }
          message.info(`Bulk delete not implemented yet. ${selected.length} users selected.`);
          refresh();
        },
      },
      {
        key: 'bulk-edit',
        label: 'Bulk Edit',
        icon: <EditOutlined />,
        handler: ({ selected, refresh }) => {
          if (!selected || selected.length === 0) {
            message.warning('No users selected');
            return;
          }
          message.info(`Bulk edit not implemented yet. ${selected.length} users selected.`);
          refresh();
        },
      },
    ],
  };

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto p-4">
        <div className="flex h-full flex-col gap-4">
          <PageHeader title="Users" description="Manage all registered users in the system." />

          <EntityList<User>
            ref={listRef}
            name="Users"
            fetchItems={fetchUsers}
            renderGridItem={(user, actions) => <UserCard user={user} actions={actions} />}
            renderListItem={(u) => (
              <UserListItem user={u} onClick={(user) => navigate(`/users/${user.id}`)} />
            )}
            getRowKey={(item) => item.id}
            onRowClick={(item) => navigate(`/users/${item.id}`)}
            columnToggleEnabled
            columns={[
              {
                title: 'ID',
                dataIndex: 'id',
                key: 'id',
                sorter: { multiple: 0 },
                defaultHidden: true,
              },
              {
                title: 'Username',
                dataIndex: 'username',
                key: 'username',
                sorter: { multiple: 1 },
              },
              {
                title: 'Email',
                dataIndex: 'email',
                key: 'email',
                sorter: { multiple: 2 },
                render: (_, record) => <a href={`mailto:${record.email}`}>{record.email}</a>,
              },
              {
                title: 'Admin',
                dataIndex: 'admin',
                key: 'admin',
                sorter: { multiple: 3 },
                filters: [
                  { text: 'Admin', value: 'true' },
                  { text: 'Regular', value: 'false' },
                ],
                render: (_, record) => <UserAdminTag admin={record.admin} />,
              },
              {
                title: 'Created At',
                dataIndex: 'created_at',
                key: 'created_at',
                sorter: { multiple: 4 },
                defaultHidden: true,
                render: (value: string) => dayjs(value).format('YYYY-MM-DD HH:mm'),
              },
              {
                title: 'Updated At',
                dataIndex: 'updated_at',
                key: 'updated_at',
                sorter: { multiple: 5 },
                defaultHidden: true,
                render: (value: string) => dayjs(value).format('YYYY-MM-DD HH:mm'),
              },
            ]}
            actions={actions}
            emptyNoEntities={
              <UsersEmptyState
                onCreate={() => setCreateOpen(true)}
                onRefresh={() => listRef.current?.refresh()}
              />
            }
            filtersStorageKey="users:filters:v1"
          />

          {/* Create User */}
          <FormModal
            open={createOpen}
            onCancel={() => setCreateOpen(false)}
            onSubmit={handleAddUser}
            title="Add User"
            submitText="Create"
            initialValues={{
              username: '',
              email: '',
              // password is typed by the user; no default recommended
            }}
            fields={userCreateFields}
          />

          {/* Edit User */}
          <FormModal
            open={editOpen}
            onCancel={() => {
              setEditOpen(false);
              setEditingItem(null);
            }}
            onSubmit={handleEditUser}
            title="Edit User"
            submitText="Save"
            initialValues={{
              username: editingItem?.username ?? '',
              email: editingItem?.email ?? '',
              admin: editingItem?.admin ?? false,
            }}
            fields={userEditFields}
          />
        </div>
      </div>
    </div>
  );
};

export default UsersList;
