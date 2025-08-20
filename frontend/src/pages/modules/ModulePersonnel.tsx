import { useEffect, useMemo, useState } from 'react';
import { Segmented, Table, Transfer, Button, Input, Typography } from 'antd';
import type { Key } from 'react';
import type { TablePaginationConfig, TableProps } from 'antd';
import { MODULE_ROLES, type ModuleRole } from '@/types/modules';
import { useTableQuery } from '@/hooks/useTableQuery';
import { useParams } from 'react-router-dom';
import PageHeader from '@/components/PageHeader';
import { message } from '@/utils/message';
import { useAuth } from '@/context/AuthContext';
import {
  getPersonnel,
  assignPersonnel,
  removePersonnel,
  getEligibleUsers,
} from '@/services/modules/personnel';
import type { TableRowSelection } from 'antd/es/table/interface';
import ModuleRoleTag, { roleLabels } from '@/components/modules/ModuleRoleTag';
import { useViewSlot } from '@/context/ViewSlotContext';

interface TableTransferItem {
  key: string;
  username: string;
  email: string;
  title: string;
  description: string;
  role?: ModuleRole;
}

const ModulePersonnel = () => {
  const { id } = useParams();
  const moduleId = Number(id);
  const auth = useAuth();
  const { setValue } = useViewSlot();

  const [targetRole, setTargetRole] = useState<ModuleRole>('tutor');
  const [eligibleUsers, setEligibleUsers] = useState<TableTransferItem[]>([]);
  const [assignedUsers, setAssignedUsers] = useState<TableTransferItem[]>([]);
  const [targetKeys, setTargetKeys] = useState<Key[]>([]);
  const [loading, setLoading] = useState(true);

  const sourceQuery = useTableQuery();
  const targetQuery = useTableQuery();

  const availableRoles = useMemo(() => {
    return MODULE_ROLES.filter((r) => auth.user?.admin || r !== 'lecturer');
  }, [auth.user]);

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Personnel
      </Typography.Text>,
    );
  }, []);

  const fetchEligibleUsers = async () => {
    const res = await getEligibleUsers(moduleId, {
      page: sourceQuery.pagination.current,
      per_page: sourceQuery.pagination.pageSize,
      query: sourceQuery.searchTerm,
      email: sourceQuery.filterState.email?.[0],
      username: sourceQuery.filterState.username?.[0],
    });
    if (res.success) {
      const users = res.data.users.map((u) => ({
        key: String(u.id),
        username: u.username,
        email: u.email,
        title: u.email,
        description: u.username,
      }));
      setEligibleUsers(users);
      sourceQuery.setPagination({ total: res.data.total });
    } else {
      message.error(res.message);
    }
  };

  const fetchAssignedUsers = async () => {
    const res = await getPersonnel(moduleId, {
      role: targetRole,
      page: targetQuery.pagination.current,
      per_page: targetQuery.pagination.pageSize,
      query: targetQuery.searchTerm,
      email: targetQuery.filterState.email?.[0],
      username: targetQuery.filterState.username?.[0],
    });
    if (res.success) {
      const users = res.data.users.map((u) => ({
        key: String(u.id),
        username: u.username,
        email: u.email,
        title: u.email,
        description: u.username,
        role: targetRole,
      }));
      setAssignedUsers(users);
      setTargetKeys(users.map((u) => u.key));
      targetQuery.setPagination({ total: res.data.total });
    } else {
      message.error(res.message);
    }
  };

  const fetchUsers = async () => {
    setLoading(true);
    await Promise.all([fetchEligibleUsers(), fetchAssignedUsers()]);
    setLoading(false);
  };

  useEffect(() => {
    fetchUsers();
  }, [
    targetRole,
    sourceQuery.pagination.current,
    sourceQuery.pagination.pageSize,
    sourceQuery.searchTerm,
    sourceQuery.filterState.email,
    sourceQuery.filterState.username,
    targetQuery.pagination.current,
    targetQuery.pagination.pageSize,
    targetQuery.searchTerm,
    targetQuery.filterState.email,
    targetQuery.filterState.username,
  ]);

  const handleTransferChange = async (nextKeys: Key[]) => {
    const prev = targetKeys.map(Number);
    const next = nextKeys.map(Number);

    const toAssign = next.filter((id) => !prev.includes(id));
    const toRemove = prev.filter((id) => !next.includes(id));

    if (toAssign.length > 0) {
      await assignPersonnel(moduleId, { role: targetRole, user_ids: toAssign });
      message.success(
        `Assigned ${toAssign.length} user${toAssign.length === 1 ? '' : 's'} to ${roleLabels[targetRole]} role`,
      );
    }

    if (toRemove.length > 0) {
      await removePersonnel(moduleId, { role: targetRole, user_ids: toRemove });
      message.success(
        `Unassigned ${toRemove.length} user${toRemove.length === 1 ? '' : 's'} from ${roleLabels[targetRole]} role`,
      );
    }

    await fetchUsers();
  };

  const getColumns = (
    _: ReturnType<typeof useTableQuery>,
    isTarget: boolean,
  ): TableProps<TableTransferItem>['columns'] => {
    const baseCols: TableProps<TableTransferItem>['columns'] = [
      {
        dataIndex: 'username',
        title: 'Username',
      },
      {
        dataIndex: 'email',
        title: 'Email',
      },
    ];

    if (isTarget) {
      baseCols.push({
        dataIndex: 'role',
        title: 'Role',
        render: (_, record) => (record.role ? <ModuleRoleTag role={record.role} /> : null),
      });
    }

    return baseCols;
  };

  const renderTable = (direction: 'left' | 'right', props: any) => {
    const state = direction === 'left' ? sourceQuery : targetQuery;
    const pagination: TablePaginationConfig = {
      current: state.pagination.current,
      pageSize: state.pagination.pageSize,
      total: state.pagination.total,
      showSizeChanger: true,
      onChange: (page, pageSize) => state.setPagination({ current: page, pageSize }),
    };

    const rowSelection: TableRowSelection<TableTransferItem> = {
      getCheckboxProps: () => ({ disabled: props.disabled }),
      onChange: (selectedRowKeys) => props.onItemSelectAll(selectedRowKeys, 'replace'),
      selectedRowKeys: props.selectedKeys,
    };

    return (
      <div className="space-y-2 p-2">
        <div className="flex gap-2 mb-2">
          <Input.Search
            allowClear
            placeholder={`Search ${direction === 'left' ? 'eligible' : 'assigned'} users`}
            value={state.searchTerm}
            onChange={(e) => state.setSearchTerm(e.target.value)}
            onSearch={() => state.setPagination({ current: 1 })}
            style={{ width: '100%' }}
            data-cy={direction === 'left' ? 'available-user-search' : 'assigned-user-search'}
          />
          <Button onClick={() => state.clearAll()}>Clear</Button>
        </div>

        <Table
          rowSelection={rowSelection}
          columns={getColumns(state, direction === 'right')}
          dataSource={props.filteredItems}
          pagination={pagination}
          loading={loading}
          size="small"
          rowKey="key"
          onRow={({ key }) => ({
            'data-cy': `${direction === 'left' ? 'available' : 'assigned'}-user-row-${key}`,
            onClick: () => {
              if (!props.disabled) {
                props.onItemSelect(key, !props.selectedKeys.includes(key));
              }
            },
          })}
          data-cy={direction === 'left' ? 'available-user-table' : 'assigned-user-table'}
        />
      </div>
    );
  };

  return (
    <div className="bg-white dark:bg-gray-950 p-4 sm:p-6 h-full overflow-y-auto">
      <PageHeader
        title="Module Personnel"
        description="Assign eligible users to a specific role in this module."
      />

      <div className="flex justify-end mb-4">
        <Segmented
          options={availableRoles.map((role) => ({
            label: roleLabels[role],
            value: role,
          }))}
          value={targetRole}
          onChange={(val) => setTargetRole(val as ModuleRole)}
          data-cy="personnel-role-selector"
        />
      </div>

      <Transfer
        dataSource={[...eligibleUsers, ...assignedUsers]}
        targetKeys={targetKeys}
        onChange={handleTransferChange}
        showSearch={false}
        showSelectAll={false}
        rowKey={(record) => record.key}
        titles={['Eligible Users', roleLabels[targetRole]]}
        filterOption={() => true}
      >
        {(props) => renderTable(props.direction as 'left' | 'right', props)}
      </Transfer>
    </div>
  );
};

export default ModulePersonnel;
