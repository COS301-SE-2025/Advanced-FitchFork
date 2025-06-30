import PageHeader from '@/components/PageHeader';
import { useNotifier } from '@/components/Notifier';
import { useNavigate } from 'react-router-dom';
import type { User } from '@/types/users';
import type { SortOption } from '@/types/common';
import { listUsers, editUser, deleteUser } from '@/services/users';
import { register } from '@/services/auth';
import { EntityList } from '@/components/EntityList';
import UserCard from '@/components/users/UserCard';
import UserAdminTag from '@/components/users/UserAdminTag';

const UsersList = () => {
  const navigate = useNavigate();
  const { notifyError, notifySuccess } = useNotifier();

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
    sort: SortOption[];
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

    notifyError('Failed to fetch users', res.message);
    return { items: [], total: 0 };
  };

  const handleAddUser = async (formValues: Record<string, any>) => {
    const { username, email } = formValues;

    const res = await register(username, email, 'changeme123');
    if (res.success) {
      notifySuccess('User registered successfully', res.message);
    } else {
      notifyError(res.message || 'Failed to register user', res.message);
    }
  };

  const handleEditUser = async (item: User, values: any) => {
    const updated: User = {
      ...item,
      ...values,
      updated_at: new Date().toISOString(),
    };

    const res = await editUser(item.id, updated);
    if (res.success) {
      notifySuccess('User updated successfully', res.message);
    } else {
      notifyError('Failed to update user', res.message);
    }
  };

  const handleDeleteUser = async (user: User) => {
    const res = await deleteUser(user.id);
    if (res.success) {
      notifySuccess('User deleted successfully', res.message);
    } else {
      notifyError('Failed to delete user', res.message);
    }
  };

  return (
    <div className="bg-gray-50 dark:bg-gray-950 p-4 sm:p-6 h-full">
      <PageHeader title="Users" description="Manage all registered users in the system." />
      <EntityList<User>
        name="Users"
        fetchItems={fetchUsers}
        renderGridItem={(user, actions) => <UserCard user={user} actions={actions} />}
        columns={[
          {
            title: 'Username',
            dataIndex: 'username',
            key: 'username',
            sorter: { multiple: 1 },
            render: (_, record) => record.username,
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
            render: (_, record) => <UserAdminTag admin={record.admin} />,
          },
        ]}
        sortOptions={[
          { label: 'Username', field: 'username' },
          { label: 'Email', field: 'email' },
          { label: 'Admin', field: 'admin' },
        ]}
        getRowKey={(item) => item.id}
        onRowClick={(item) => navigate(`/users/${item.id}`)}
        createModal={{
          title: 'Add User',
          onCreate: handleAddUser,
          fields: [
            { name: 'username', label: 'Username', type: 'text', required: true },
            { name: 'email', label: 'Email', type: 'email', required: true },
          ],
          getInitialValues: () => ({
            username: '',
            email: '',
            admin: false,
          }),
        }}
        editModal={{
          title: 'Edit User',
          onEdit: handleEditUser,
          fields: [
            { name: 'username', label: 'Username', type: 'text', required: true },
            { name: 'email', label: 'Email', type: 'email', required: true },
            { name: 'admin', label: 'Admin', type: 'boolean' },
          ],
        }}
        onDelete={handleDeleteUser}
        filterGroups={[
          {
            key: 'username',
            label: 'Username',
            type: 'text',
          },
          {
            key: 'email',
            label: 'Email',
            type: 'text',
          },
          {
            key: 'admin',
            label: 'Admin Status',
            type: 'select',
            options: [
              { label: 'Admin', value: 'true' },
              { label: 'Regular', value: 'false' },
            ],
          },
        ]}
      />
    </div>
  );
};

export default UsersList;
