import { useEffect, useState } from 'react';
import { Typography, Segmented, Table, Transfer, Input, Button, Tag, Skeleton } from 'antd';
import type { Key } from 'react';
import type { TransferProps, TablePaginationConfig, TableProps } from 'antd';
import type { ModuleRole } from '@/types/modules';
import { ModulesService } from '@/services/modules';
import { useNotifier } from '@/components/Notifier';
import { useTableQuery } from '@/hooks/useTableQuery';

const { Title, Text } = Typography;
const ROLES: ModuleRole[] = ['Lecturer', 'Tutor', 'Student'];

interface Props {
  moduleId: number;
}

interface TableTransferItem {
  key: string;
  student_number: string;
  email: string;
  title: string;
  description: string;
  role?: ModuleRole;
}

type TableRowSelection<T> = TableProps<T>['rowSelection'];
type TransferItem = Required<TransferProps>['dataSource'][number];

export default function PersonnelSection({ moduleId }: Props) {
  const { notifyError, notifySuccess } = useNotifier();

  const [eligibleUsers, setEligibleUsers] = useState<TableTransferItem[]>([]);
  const [assignedUsers, setAssignedUsers] = useState<TableTransferItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedRole, setSelectedRole] = useState<ModuleRole>('Student');
  const [roleAssignments, setRoleAssignments] = useState<Record<ModuleRole, Key[]>>({
    Lecturer: [],
    Tutor: [],
    Student: [],
  });

  const available = useTableQuery();
  const assigned = useTableQuery();

  const fetchData = async () => {
    setLoading(true);
    try {
      const eligibleRes = await ModulesService.getEligibleUsersForRole(moduleId, selectedRole, {
        page: available.pagination.current,
        per_page: available.pagination.pageSize,
        query: available.searchTerm,
        email: available.filterState.email?.[0],
        student_number: available.filterState.student_number?.[0],
      });

      const assignedRes = await {
        Lecturer: ModulesService.getLecturers,
        Tutor: ModulesService.getTutors,
        Student: ModulesService.getStudents,
      }[selectedRole](moduleId, {
        page: assigned.pagination.current,
        per_page: assigned.pagination.pageSize,
        query: assigned.searchTerm,
        email: assigned.filterState.email?.[0],
        student_number: assigned.filterState.student_number?.[0],
      });

      if (eligibleRes.success) {
        setEligibleUsers(
          eligibleRes.data.users.map((u) => ({
            key: String(u.id),
            student_number: u.student_number,
            email: u.email,
            title: u.email,
            description: u.student_number,
          })),
        );
        available.setPagination({ total: eligibleRes.data.total });
      } else {
        notifyError('Failed to load eligible users', eligibleRes.message);
      }

      if (assignedRes.success) {
        setAssignedUsers(
          assignedRes.data.users.map((u) => ({
            key: String(u.id),
            student_number: u.student_number,
            email: u.email,
            title: u.email,
            description: u.student_number,
            role: selectedRole, // <- add this line
          })),
        );
        assigned.setPagination({ total: assignedRes.data.total });
        setRoleAssignments((prev) => ({
          ...prev,
          [selectedRole]: assignedRes.data.users.map((u) => String(u.id)),
        }));
      } else {
        notifyError('Failed to load assigned users', assignedRes.message);
      }
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, [
    moduleId,
    selectedRole,
    available.pagination.current,
    available.pagination.pageSize,
    available.searchTerm,
    available.filterState.email,
    available.filterState.student_number,
    assigned.pagination.current,
    assigned.pagination.pageSize,
    assigned.searchTerm,
    assigned.filterState.email,
    assigned.filterState.student_number,
  ]);

  const handleTransferChange = async (
    nextKeys: Key[],
    _direction: 'left' | 'right',
    _movedKeys: Key[],
  ) => {
    const user_ids = nextKeys.map(Number);
    const prevKeys = roleAssignments[selectedRole].map(Number);

    const toAdd = user_ids.filter((id) => !prevKeys.includes(id));
    const toRemove = prevKeys.filter((id) => !user_ids.includes(id));

    setRoleAssignments((prev) => ({ ...prev, [selectedRole]: nextKeys }));

    const assignFn = {
      Lecturer: ModulesService.assignLecturers,
      Tutor: ModulesService.assignTutors,
      Student: ModulesService.enrollStudents,
    }[selectedRole];

    const removeFn = {
      Lecturer: ModulesService.removeLecturers,
      Tutor: ModulesService.removeTutors,
      Student: ModulesService.removeStudents,
    }[selectedRole];

    let assignRes = { success: true, message: '' };
    let removeRes = { success: true, message: '' };

    if (toAdd.length) {
      assignRes = await assignFn(moduleId, { user_ids: toAdd });
    }

    if (toRemove.length) {
      removeRes = await removeFn(moduleId, { user_ids: toRemove });
    }

    if (assignRes.success && removeRes.success) {
      notifySuccess(`Updated ${selectedRole}s`, assignRes.message || removeRes.message);
      await fetchData();
    } else {
      const firstError = !assignRes.success ? assignRes : removeRes;
      notifyError(`Failed to update ${selectedRole}s`, firstError.message);
    }
  };

  const allUsers = [...eligibleUsers, ...assignedUsers].filter(
    (u, i, arr) => arr.findIndex((x) => x.key === u.key) === i,
  );

  const getColumns = (
    state: ReturnType<typeof useTableQuery>,
  ): TableProps<TableTransferItem>['columns'] => [
    {
      dataIndex: 'student_number',
      title: 'Student Number',
      filterDropdown: ({ setSelectedKeys, selectedKeys, confirm, clearFilters }) => (
        <div style={{ padding: 8 }}>
          <Input
            placeholder="Filter by student number"
            value={selectedKeys[0]}
            onChange={(e) => setSelectedKeys(e.target.value ? [e.target.value] : [])}
            onPressEnter={() => {
              confirm();
              state.setFilterState({
                ...state.filterState,
                student_number: [selectedKeys[0] as string],
              });
            }}
            style={{ width: 188, marginBottom: 8, display: 'block' }}
          />
          <div style={{ display: 'flex', justifyContent: 'space-between' }}>
            <a
              onClick={() => {
                confirm();
                state.setFilterState({
                  ...state.filterState,
                  student_number: [selectedKeys[0] as string],
                });
              }}
            >
              Apply
            </a>
            <a
              onClick={() => {
                clearFilters?.();
                state.setFilterState({ ...state.filterState, student_number: [] });
              }}
            >
              Reset
            </a>
          </div>
        </div>
      ),
    },
    {
      dataIndex: 'email',
      title: 'Email',
      filterDropdown: ({ setSelectedKeys, selectedKeys, confirm, clearFilters }) => (
        <div style={{ padding: 8 }}>
          <Input
            placeholder="Filter by email"
            value={selectedKeys[0]}
            onChange={(e) => setSelectedKeys(e.target.value ? [e.target.value] : [])}
            onPressEnter={() => {
              confirm();
              state.setFilterState({ ...state.filterState, email: [selectedKeys[0] as string] });
            }}
            style={{ width: 188, marginBottom: 8, display: 'block' }}
          />
          <div style={{ display: 'flex', justifyContent: 'space-between' }}>
            <a
              onClick={() => {
                confirm();
                state.setFilterState({ ...state.filterState, email: [selectedKeys[0] as string] });
              }}
            >
              Apply
            </a>
            <a
              onClick={() => {
                clearFilters?.();
                state.setFilterState({ ...state.filterState, email: [] });
              }}
            >
              Reset
            </a>
          </div>
        </div>
      ),
    },
    {
      dataIndex: 'role',
      title: 'Role',
      render: (_, record) => {
        if (!record.role) {
          return <Tag color="default">None</Tag>;
        }
        const color =
          record.role === 'Lecturer' ? 'volcano' : record.role === 'Tutor' ? 'geekblue' : 'green';
        return <Tag color={color}>{record.role}</Tag>;
      },
    },
  ];

  const renderTableTransfer = (direction: 'left' | 'right', props: any) => {
    const state = direction === 'left' ? available : assigned;
    const pagination: TablePaginationConfig = {
      current: state.pagination.current,
      pageSize: state.pagination.pageSize,
      total: state.pagination.total,
      pageSizeOptions: state.pagination.pageSizeOptions,
      showSizeChanger: true,
      onChange: (page, pageSize) => {
        state.setPagination({ current: page, pageSize });
      },
    };

    const rowSelection: TableRowSelection<TransferItem> = {
      getCheckboxProps: () => ({ disabled: props.disabled }),
      onChange: (selectedRowKeys) => {
        props.onItemSelectAll(selectedRowKeys, 'replace');
      },
      selectedRowKeys: props.selectedKeys,
    };

    return (
      <div className="space-y-2 p-2">
        <div className="flex gap-2">
          <Input.Search
            allowClear
            placeholder={`Search ${direction === 'left' ? 'available' : 'assigned'} users`}
            value={state.searchTerm}
            onChange={(e) => state.setSearchTerm(e.target.value)}
            onSearch={() => state.setPagination({ current: 1 })}
            style={{ width: '100%' }}
          />
          <Button
            onClick={() => {
              state.clearSearch();
              state.setPagination({ current: 1 });
            }}
          >
            Clear
          </Button>
        </div>
        {loading ? (
          <div className="space-y-2">
            {[...Array(5)].map((_, idx) => (
              <div key={idx} className="grid grid-cols-3 gap-4 w-full">
                <Skeleton.Input block active />
                <Skeleton.Input block active />
                <Skeleton.Input block active />
              </div>
            ))}
          </div>
        ) : (
          <Table
            rowSelection={rowSelection}
            columns={getColumns(state)}
            dataSource={props.filteredItems}
            pagination={pagination}
            size="small"
            rowKey="key"
            onRow={({ key }) => ({
              onClick: () => {
                if (!props.disabled) {
                  props.onItemSelect(key, !props.selectedKeys.includes(key));
                }
              },
            })}
            style={{ pointerEvents: props.disabled ? 'none' : undefined }}
          />
        )}
      </div>
    );
  };

  return (
    <div className="space-y-6">
      <div>
        <Title level={4}>Module Personnel</Title>
        <Text className="text-gray-500 dark:text-gray-400">
          Use the segmented selector to assign Lecturers, Tutors, or Students.
        </Text>
      </div>

      <div className="bg-white dark:bg-gray-900">
        <div className="flex flex-col gap-4">
          <Segmented
            options={ROLES}
            value={selectedRole}
            onChange={(val) => {
              setSelectedRole(val as ModuleRole);
              available.setPagination({ current: 1 });
              assigned.setPagination({ current: 1 });
              available.clearSearch();
              assigned.clearSearch();
              available.clearFilters();
              assigned.clearFilters();
            }}
            size="large"
            block
          />

          <Transfer
            dataSource={allUsers}
            targetKeys={roleAssignments[selectedRole]}
            onChange={handleTransferChange}
            showSearch={false}
            showSelectAll={false}
            rowKey={(record) => record.key}
            titles={['Available', 'Assigned']}
            filterOption={() => true}
            oneWay={false}
          >
            {(props) => renderTableTransfer(props.direction as 'left' | 'right', props)}
          </Transfer>
        </div>
      </div>
    </div>
  );
}
