import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  Segmented,
  Table,
  Button,
  Input,
  Typography,
  Card,
  Tag,
  Grid,
  Tabs,
  Space,
  Empty,
} from 'antd';
import type { ColumnsType } from 'antd/es/table';
import type { Key } from 'react';
import { useParams } from 'react-router-dom';

import { MODULE_ROLES, type ModuleRole } from '@/types/modules';
import { useTableQuery } from '@/hooks/useTableQuery';
import PageHeader from '@/components/PageHeader';
import { message } from '@/utils/message';
import { useAuth } from '@/context/AuthContext';
import {
  getPersonnel,
  assignPersonnel,
  removePersonnel,
  getEligibleUsers,
} from '@/services/modules/personnel';
import ModuleRoleTag, { roleLabels } from '@/components/modules/ModuleRoleTag';
import { useViewSlot } from '@/context/ViewSlotContext';

interface PersonnelRow {
  key: string;
  userId: number;
  username: string;
  email: string;
  role?: ModuleRole;
}

interface PersonnelTableCardProps {
  title: string;
  description: string;
  queryState: ReturnType<typeof useTableQuery>;
  data: PersonnelRow[];
  columns: ColumnsType<PersonnelRow>;
  loading: boolean;
  selectedKeys: Key[];
  onSelectionChange: (keys: Key[]) => void;
  action?: {
    label: string;
    onClick: () => void | Promise<void>;
    disabled: boolean;
    loading: boolean;
    dataCy: string;
  };
  emptyText: string;
  searchPlaceholder: string;
  dataCyPrefix: string;
}

const PersonnelTableCard = ({
  title,
  description,
  queryState,
  data,
  columns,
  loading,
  selectedKeys,
  onSelectionChange,
  action,
  emptyText,
  searchPlaceholder,
  dataCyPrefix,
}: PersonnelTableCardProps) => {
  const { searchTerm, setSearchTerm, clearAll, pagination, setPagination } = queryState;

  const handleReset = () => {
    clearAll();
    setPagination({ current: 1 });
  };

  const rowSelection = {
    selectedRowKeys: selectedKeys,
    onChange: (keys: Key[]) => onSelectionChange(keys),
  };

  return (
    <Card className="h-full" bodyStyle={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
      <div className="flex flex-wrap items-center gap-2">
        <Typography.Text className="font-medium text-base text-gray-900 dark:text-gray-100">
          {title}
        </Typography.Text>
        <Tag color="blue">{pagination.total ?? 0} total</Tag>
        {selectedKeys.length > 0 && <Tag color="processing">{selectedKeys.length} selected</Tag>}
      </div>
      <Typography.Paragraph className="!mt-0 !mb-2 !text-sm !text-gray-600 dark:!text-gray-400">
        {description}
      </Typography.Paragraph>

      <Space direction="vertical" size="middle" className="w-full">
        <div className="flex flex-col-reverse sm:flex-row sm:items-center sm:justify-between gap-2">
          <div className="flex flex-col sm:flex-row sm:items-center gap-2 w-full sm:w-auto">
            <Input.Search
              allowClear
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              onSearch={() => setPagination({ current: 1 })}
              placeholder={searchPlaceholder}
              className="w-full sm:w-64"
              data-cy={`${dataCyPrefix}-search`}
            />
            <Button onClick={handleReset} data-cy={`${dataCyPrefix}-reset`}>
              Reset
            </Button>
          </div>

          {action && (
            <Button
              type="primary"
              disabled={action.disabled}
              loading={action.loading}
              onClick={action.onClick}
              className="w-full sm:w-auto"
              data-cy={action.dataCy}
            >
              {action.label}
            </Button>
          )}
        </div>

        <Table<PersonnelRow>
          rowSelection={rowSelection}
          columns={columns}
          dataSource={data}
          loading={loading}
          pagination={{
            ...pagination,
            showSizeChanger: true,
            onChange: (page, pageSize) => setPagination({ current: page, pageSize }),
          }}
          size="middle"
          rowKey="key"
          scroll={{ x: true }}
          locale={{
            emptyText: <Empty description={emptyText} />,
          }}
          onRow={(record) => ({
            onClick: () => {
              const exists = selectedKeys.includes(record.key);
              if (exists) {
                onSelectionChange(selectedKeys.filter((key) => key !== record.key));
              } else {
                onSelectionChange([...selectedKeys, record.key]);
              }
            },
          })}
          data-cy={`${dataCyPrefix}-table`}
        />
      </Space>
    </Card>
  );
};

const ModulePersonnel = () => {
  const { id } = useParams();
  const moduleId = Number(id);
  const auth = useAuth();
  const { setValue } = useViewSlot();
  const screens = Grid.useBreakpoint();

  const [targetRole, setTargetRole] = useState<ModuleRole>('student');
  const [eligibleUsers, setEligibleUsers] = useState<PersonnelRow[]>([]);
  const [assignedUsers, setAssignedUsers] = useState<PersonnelRow[]>([]);
  const [eligibleSelection, setEligibleSelection] = useState<Key[]>([]);
  const [assignedSelection, setAssignedSelection] = useState<Key[]>([]);
  const [eligibleLoading, setEligibleLoading] = useState(false);
  const [assignedLoading, setAssignedLoading] = useState(false);
  const [assigning, setAssigning] = useState(false);
  const [removing, setRemoving] = useState(false);

  const eligibleQuery = useTableQuery();
  const assignedQuery = useTableQuery();
  const setEligiblePagination = eligibleQuery.setPagination;
  const setAssignedPagination = assignedQuery.setPagination;

  const availableRoles = useMemo(
    () => MODULE_ROLES.filter((r) => auth.user?.admin || r !== 'lecturer'),
    [auth.user],
  );

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Personnel
      </Typography.Text>,
    );
  }, [setValue]);

  useEffect(() => {
    if (Number.isNaN(moduleId)) return;
    setEligiblePagination({ current: 1 });
    setAssignedPagination({ current: 1 });
  }, [moduleId, setAssignedPagination, setEligiblePagination]);

  const eligibleEmailFilter = eligibleQuery.filterState.email?.[0];
  const eligibleUsernameFilter = eligibleQuery.filterState.username?.[0];
  const assignedEmailFilter = assignedQuery.filterState.email?.[0];
  const assignedUsernameFilter = assignedQuery.filterState.username?.[0];

  const eligibleParams = useMemo(
    () => ({
      page: eligibleQuery.pagination.current,
      per_page: eligibleQuery.pagination.pageSize,
      query: eligibleQuery.searchTerm || undefined,
      email: eligibleEmailFilter,
      username: eligibleUsernameFilter,
    }),
    [
      eligibleQuery.pagination.current,
      eligibleQuery.pagination.pageSize,
      eligibleQuery.searchTerm,
      eligibleEmailFilter,
      eligibleUsernameFilter,
    ],
  );

  const assignedParams = useMemo(
    () => ({
      page: assignedQuery.pagination.current,
      per_page: assignedQuery.pagination.pageSize,
      query: assignedQuery.searchTerm || undefined,
      email: assignedEmailFilter,
      username: assignedUsernameFilter,
      role: targetRole,
    }),
    [
      assignedQuery.pagination.current,
      assignedQuery.pagination.pageSize,
      assignedQuery.searchTerm,
      assignedEmailFilter,
      assignedUsernameFilter,
      targetRole,
    ],
  );

  const fetchEligibleUsers = useCallback(async () => {
    if (Number.isNaN(moduleId)) return;
    setEligibleLoading(true);
    try {
      const res = await getEligibleUsers(moduleId, eligibleParams);
      if (!res.success) {
        message.error(res.message || 'Failed to load eligible users');
        setEligibleUsers([]);
        setEligiblePagination({ total: 0 });
        return;
      }

      const users = (res.data?.users ?? []).map((user) => ({
        key: String(user.id),
        userId: user.id,
        username: user.username,
        email: user.email,
      }));
      setEligibleUsers(users);
      setEligiblePagination({ total: res.data?.total ?? 0 });
      setEligibleSelection((prev) => prev.filter((key) => users.some((u) => u.key === key)));
    } catch (error) {
      message.error('Failed to load eligible users');
    } finally {
      setEligibleLoading(false);
    }
  }, [moduleId, eligibleParams, setEligiblePagination]);

  const fetchAssignedUsers = useCallback(async () => {
    if (Number.isNaN(moduleId)) return;
    setAssignedLoading(true);
    try {
      const res = await getPersonnel(moduleId, assignedParams);
      if (!res.success) {
        message.error(res.message || 'Failed to load assigned personnel');
        setAssignedUsers([]);
        setAssignedPagination({ total: 0 });
        return;
      }

      const users = (res.data?.users ?? []).map((user) => ({
        key: String(user.id),
        userId: user.id,
        username: user.username,
        email: user.email,
        role: targetRole,
      }));
      setAssignedUsers(users);
      setAssignedPagination({ total: res.data?.total ?? 0 });
      setAssignedSelection((prev) => prev.filter((key) => users.some((u) => u.key === key)));
    } catch (error) {
      message.error('Failed to load assigned personnel');
    } finally {
      setAssignedLoading(false);
    }
  }, [moduleId, assignedParams, setAssignedPagination, targetRole]);

  useEffect(() => {
    void fetchEligibleUsers();
  }, [fetchEligibleUsers]);

  useEffect(() => {
    void fetchAssignedUsers();
  }, [fetchAssignedUsers]);

  const handleAssignSelected = async () => {
    if (eligibleSelection.length === 0) return;
    setAssigning(true);
    try {
      const userIds = eligibleSelection.map((key) => Number(key));
      const res = await assignPersonnel(moduleId, { role: targetRole, user_ids: userIds });
      if (!res.success) {
        message.error(res.message || 'Failed to assign personnel');
        return;
      }
      message.success(
        `Assigned ${userIds.length} user${userIds.length === 1 ? '' : 's'} to ${roleLabels[targetRole]} role`,
      );
      setEligibleSelection([]);
      await Promise.all([fetchEligibleUsers(), fetchAssignedUsers()]);
    } catch (error) {
      message.error('Failed to assign personnel');
    } finally {
      setAssigning(false);
    }
  };

  const handleRemoveSelected = async () => {
    if (assignedSelection.length === 0) return;
    setRemoving(true);
    try {
      const userIds = assignedSelection.map((key) => Number(key));
      const res = await removePersonnel(moduleId, { role: targetRole, user_ids: userIds });
      if (!res.success) {
        message.error(res.message || 'Failed to unassign personnel');
        return;
      }
      message.success(
        `Removed ${userIds.length} user${userIds.length === 1 ? '' : 's'} from ${roleLabels[targetRole]} role`,
      );
      setAssignedSelection([]);
      await Promise.all([fetchEligibleUsers(), fetchAssignedUsers()]);
    } catch (error) {
      message.error('Failed to unassign personnel');
    } finally {
      setRemoving(false);
    }
  };

  const isDesktop = screens.lg ?? false;
  const isMobile = !(screens.md ?? false);

  const eligibleColumns: ColumnsType<PersonnelRow> = useMemo(() => {
    const columns: ColumnsType<PersonnelRow> = [
      {
        dataIndex: 'username',
        title: 'Username',
        render: (value: string) => <Typography.Text strong>{value}</Typography.Text>,
      },
    ];

    if (!isMobile) {
      columns.push({ dataIndex: 'email', title: 'Email' });
    }

    return columns;
  }, [isMobile]);

  const assignedColumns: ColumnsType<PersonnelRow> = useMemo(() => {
    const columns = [...eligibleColumns];

    if (!isMobile) {
      columns.push({
        dataIndex: 'role',
        title: 'Role',
        render: (role: ModuleRole | undefined) => (role ? <ModuleRoleTag role={role} /> : null),
      });
    }

    return columns;
  }, [eligibleColumns, isMobile]);

  const roleSelector = (
    <Segmented
      options={availableRoles.map((role) => ({
        label: roleLabels[role],
        value: role,
      }))}
      value={targetRole}
      onChange={(value) => {
        setTargetRole(value as ModuleRole);
        assignedQuery.setPagination({ current: 1 });
        assignedQuery.clearSearch();
        assignedQuery.clearFilters();
        setAssignedSelection([]);
      }}
      data-cy="personnel-role-selector"
      block
    />
  );

  const assignCard = (
    <PersonnelTableCard
      title="Eligible Users"
      description="Search for users who can be assigned to this module role."
      queryState={eligibleQuery}
      data={eligibleUsers}
      columns={eligibleColumns}
      loading={eligibleLoading}
      selectedKeys={eligibleSelection}
      onSelectionChange={setEligibleSelection}
      action={{
        label: `Assign to ${roleLabels[targetRole]}`,
        onClick: handleAssignSelected,
        disabled: eligibleSelection.length === 0,
        loading: assigning,
        dataCy: 'assign-personnel-action',
      }}
      emptyText="No eligible users found"
      searchPlaceholder="Search eligible users"
      dataCyPrefix="eligible-users"
    />
  );

  const assignedCard = (
    <PersonnelTableCard
      title={`${roleLabels[targetRole]} Personnel`}
      description="Manage who currently holds this role in the module."
      queryState={assignedQuery}
      data={assignedUsers}
      columns={assignedColumns}
      loading={assignedLoading}
      selectedKeys={assignedSelection}
      onSelectionChange={setAssignedSelection}
      action={{
        label: `Remove from ${roleLabels[targetRole]}`,
        onClick: handleRemoveSelected,
        disabled: assignedSelection.length === 0,
        loading: removing,
        dataCy: 'remove-personnel-action',
      }}
      emptyText="No users assigned yet"
      searchPlaceholder="Search assigned users"
      dataCyPrefix="assigned-users"
    />
  );

  if (Number.isNaN(moduleId)) {
    return (
      <div className="bg-white dark:bg-gray-950 p-4 sm:p-6 h-full overflow-y-auto">
        <PageHeader
          title="Module Personnel"
          description="Assign eligible users to a specific role in this module."
        />
        <Typography.Text type="danger">Invalid module identifier.</Typography.Text>
      </div>
    );
  }

  return (
    <div className="bg-gray-50 dark:bg-gray-950 p-4 sm:p-6 h-full overflow-y-auto">
      <PageHeader
        title="Module Personnel"
        description="Assign eligible users to specific roles and keep track of who has access."
      />

      <div className="mt-4">{roleSelector}</div>

      <div className="mt-6">
        {isDesktop ? (
          <div className="grid lg:grid-cols-2 gap-4 xl:gap-6">
            {assignCard}
            {assignedCard}
          </div>
        ) : (
          <Tabs
            items={[
              { key: 'assign', label: `Assign ${roleLabels[targetRole]}`, children: assignCard },
              { key: 'manage', label: `Manage ${roleLabels[targetRole]}`, children: assignedCard },
            ]}
          />
        )}
      </div>
    </div>
  );
};

export default ModulePersonnel;
