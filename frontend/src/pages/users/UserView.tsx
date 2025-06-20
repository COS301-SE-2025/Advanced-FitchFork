import { useParams } from 'react-router-dom';
import { useEffect, useState } from 'react';
import { Typography, Table, Spin, Descriptions, Select, Button, Popconfirm, Space } from 'antd';
import { EditOutlined, DeleteOutlined, CheckOutlined, CloseOutlined } from '@ant-design/icons';
import { UsersService } from '@/services/users';
import { ModulesService } from '@/services/modules';
import ModuleRoleTag from '@/components/ModuleRoleTag';
import AdminTag from '@/components/AdminTag';
import { useNotifier } from '@/components/Notifier';
import type { User } from '@/types/users';
import { type ModuleRole, type Module, MODULE_ROLES } from '@/types/modules';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

const { Title } = Typography;

interface UserModule {
  module: Module;
  role: ModuleRole;
}

const UserView = () => {
  const { id } = useParams();
  const userId = Number(id);
  const [user, setUser] = useState<User | null>(null);
  const [modules, setModules] = useState<UserModule[]>([]);
  const [loading, setLoading] = useState(true);
  const [editingModuleId, setEditingModuleId] = useState<number | null>(null);
  const [editedRole, setEditedRole] = useState<ModuleRole | null>(null);
  const { notifySuccess, notifyError } = useNotifier();
  const { setBreadcrumbLabel } = useBreadcrumbContext();

  const fetchUserData = async () => {
    setLoading(true);
    const userRes = await UsersService.getUser(userId);
    const moduleRes = await UsersService.getUserModules(userId);

    if (userRes.success) {
      setUser(userRes.data);

      // Set custom breadcrumb label using the student number
      setBreadcrumbLabel(`users/${userId}`, userRes.data.student_number);
    } else {
      notifyError('Failed to load user', userRes.message);
    }

    if (moduleRes.success) {
      const transformed = moduleRes.data.map((mod) => ({
        module: {
          id: mod.id,
          code: mod.code,
          year: mod.year,
          description: mod.description,
          credits: mod.credits,
          created_at: mod.created_at,
          updated_at: mod.updated_at,
        },
        role: mod.role as ModuleRole,
      }));
      setModules(transformed);
    } else {
      notifyError('Failed to load modules', moduleRes.message);
    }

    setLoading(false);
  };

  useEffect(() => {
    fetchUserData();
  }, [id]);

  const handleChangeRole = async (moduleId: number, oldRole: ModuleRole, newRole: ModuleRole) => {
    if (oldRole === newRole) return;

    try {
      if (oldRole === 'Lecturer')
        await ModulesService.removeLecturers(moduleId, { user_ids: [userId] });
      if (oldRole === 'Tutor') await ModulesService.removeTutors(moduleId, { user_ids: [userId] });
      if (oldRole === 'Student')
        await ModulesService.removeStudents(moduleId, { user_ids: [userId] });

      if (newRole === 'Lecturer')
        await ModulesService.assignLecturers(moduleId, { user_ids: [userId] });
      if (newRole === 'Tutor') await ModulesService.assignTutors(moduleId, { user_ids: [userId] });
      if (newRole === 'Student')
        await ModulesService.enrollStudents(moduleId, { user_ids: [userId] });

      notifySuccess('Role updated', 'The user’s role was successfully updated.');
      setEditingModuleId(null);
      setEditedRole(null);
      fetchUserData();
    } catch (err) {
      notifyError('Update failed', 'Could not change the user’s role.');
    }
  };

  const handleRemove = async (moduleId: number, role: ModuleRole) => {
    try {
      if (role === 'Lecturer')
        await ModulesService.removeLecturers(moduleId, { user_ids: [userId] });
      if (role === 'Tutor') await ModulesService.removeTutors(moduleId, { user_ids: [userId] });
      if (role === 'Student') await ModulesService.removeStudents(moduleId, { user_ids: [userId] });

      notifySuccess('User removed', 'The user was removed from this module.');
      fetchUserData();
    } catch {
      notifyError('Removal failed', 'Could not remove the user from the module.');
    }
  };

  if (loading || !user) return <Spin className="mt-12 ml-6" />;

  return (
    <div className="p-4 sm:p-6">
      <Descriptions layout="vertical" bordered column={3} className="mb-8">
        <Descriptions.Item label="User ID">{user.id}</Descriptions.Item>
        <Descriptions.Item label="Student Number">{user.student_number}</Descriptions.Item>
        <Descriptions.Item label="Email">{user.email}</Descriptions.Item>
        <Descriptions.Item label="Admin">
          <AdminTag isAdmin={user.admin} />
        </Descriptions.Item>
      </Descriptions>

      <Title level={4} className="!mb-4 !mt-6">
        Module Roles
      </Title>
      <Table
        rowKey={(record) => `${record.module.id}-${record.role}`}
        dataSource={modules}
        pagination={false}
        columns={[
          {
            title: 'Code',
            dataIndex: ['module', 'code'],
            width: '15%',
          },
          {
            title: 'Year',
            dataIndex: ['module', 'year'],
            width: '10%',
          },
          {
            title: 'Description',
            dataIndex: ['module', 'description'],
            width: '40%',
            ellipsis: true,
          },
          {
            title: 'Role',
            dataIndex: 'role',
            width: '20%',
            render: (role: ModuleRole, record) =>
              editingModuleId === record.module.id ? (
                <Select
                  value={editedRole ?? role}
                  onChange={(value) => setEditedRole(value)}
                  options={MODULE_ROLES.map((r) => ({
                    value: r,
                    label: r.charAt(0).toUpperCase() + r.slice(1),
                  }))}
                  style={{ width: '100%' }}
                />
              ) : (
                <ModuleRoleTag role={role} />
              ),
          },
          {
            title: 'Actions',
            dataIndex: 'actions',
            width: '15%',
            render: (_, record) => {
              const isEditing = editingModuleId === record.module.id;

              return (
                <Space>
                  {isEditing ? (
                    <>
                      <Button
                        icon={<CheckOutlined />}
                        type="primary"
                        size="small"
                        onClick={() =>
                          handleChangeRole(record.module.id, record.role, editedRole ?? record.role)
                        }
                      />
                      <Button
                        icon={<CloseOutlined />}
                        size="small"
                        onClick={() => {
                          setEditingModuleId(null);
                          setEditedRole(null);
                        }}
                      />
                    </>
                  ) : (
                    <Button
                      icon={<EditOutlined />}
                      type="link"
                      size="small"
                      onClick={() => {
                        setEditingModuleId(record.module.id);
                        setEditedRole(record.role);
                      }}
                    />
                  )}
                  <Popconfirm
                    title="Remove user from this module?"
                    onConfirm={() => handleRemove(record.module.id, record.role)}
                    okText="Yes"
                    cancelText="No"
                  >
                    <Button icon={<DeleteOutlined />} type="text" size="small" danger />
                  </Popconfirm>
                </Space>
              );
            },
          },
        ]}
      />
    </div>
  );
};

export default UserView;
