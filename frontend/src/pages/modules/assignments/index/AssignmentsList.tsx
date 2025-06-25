import { Table, Tag, Button, Dropdown } from 'antd';
import { CalendarOutlined, MoreOutlined, EyeOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import type { ColumnsType, TablePaginationConfig } from 'antd/es/table';
import type { SorterResult } from 'antd/es/table/interface';
import { useEffect, useState } from 'react';
import dayjs from 'dayjs';
import PageHeader from '@/components/PageHeader';
import { useModule } from '@/context/ModuleContext';
import { useNotifier } from '@/components/Notifier';
import { useTableQuery } from '@/hooks/useTableQuery';
import { listAssignments } from '@/services/modules/assignments';
import { type Assignment, type AssignmentType } from '@/types/modules/assignments';
import type { SortOption } from '@/types/common';

const statusColorMap: Record<string, string> = {
  Submitted: 'green',
  Pending: 'orange',
  'Not Started': 'red',
};

const AssignmentsList = () => {
  const navigate = useNavigate();
  const module = useModule();
  const { notifyError } = useNotifier();

  const { pagination, setPagination, filterState, setFilterState, sorterState, setSorterState } =
    useTableQuery();

  const [assignments, setAssignments] = useState<Assignment[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchAssignments = async () => {
    setLoading(true);
    const res = await listAssignments(module.id, {
      page: pagination.current,
      per_page: pagination.pageSize,
      assignment_type: filterState.assignment_type?.[0] as AssignmentType | undefined,
      sort: sorterState,
    });

    if (res.success) {
      setAssignments(res.data.assignments);
      setPagination({ total: res.data.total });
    } else {
      notifyError('Failed to load assignments', res.message);
    }

    setLoading(false);
  };

  useEffect(() => {
    fetchAssignments();
  }, [module.id, pagination.current, pagination.pageSize, filterState, sorterState]);

  const columns: ColumnsType<Assignment> = [
    {
      title: 'Name',
      dataIndex: 'name',
      key: 'name',
      sorter: true,
      render: (name, record) => (
        <Button
          type="link"
          className="p-0"
          onClick={(e) => {
            e.stopPropagation();
            navigate(`${record.id}/submissions`);
          }}
        >
          {name}
        </Button>
      ),
    },
    {
      title: 'Type',
      dataIndex: 'assignment_type',
      key: 'assignment_type',
      render: (type) => (
        <Tag color={type === 'Practical' ? 'blue' : 'green'}>
          {type.charAt(0).toUpperCase() + type.slice(1)}
        </Tag>
      ),
    },
    {
      title: 'Status',
      dataIndex: 'status',
      key: 'status',
      render: (status) => (
        <Tag color={statusColorMap[status || 'Not Started']}>{status || 'Not Started'}</Tag>
      ),
    },
    {
      title: 'Due Date',
      dataIndex: 'due_date',
      key: 'due_date',
      sorter: true,
      render: (date) => (
        <span>
          <CalendarOutlined className="mr-1" />
          {dayjs(date).format('YYYY-MM-DD')}
        </span>
      ),
    },
    {
      title: 'Actions',
      key: 'actions',
      align: 'right',
      render: (_, record) => ({
        children: (
          <Dropdown
            trigger={['click']}
            menu={{
              items: [
                {
                  key: 'view',
                  icon: <EyeOutlined />,
                  label: 'View Submissions',
                },
              ],
              onClick: ({ key, domEvent }) => {
                domEvent.stopPropagation();
                if (key === 'view') {
                  navigate(`${record.id}/submissions`);
                }
              },
            }}
          >
            <Button icon={<MoreOutlined />} onClick={(e) => e.stopPropagation()} />
          </Dropdown>
        ),
      }),
    },
  ];

  const handleTableChange = (
    paginationConfig: TablePaginationConfig,
    filters: Record<string, any>,
    sorter: SorterResult<Assignment> | SorterResult<Assignment>[],
  ) => {
    setPagination({
      current: paginationConfig.current || 1,
      pageSize: paginationConfig.pageSize || 10,
    });

    const sorters: SortOption[] = [];
    const arr = Array.isArray(sorter) ? sorter : [sorter];
    for (const s of arr) {
      if (s.field && s.order) {
        sorters.push({ field: String(s.field), order: s.order });
      }
    }

    setSorterState(sorters);
    setFilterState(filters as Record<string, string[]>);
  };

  return (
    <div className="max-w-4xl p-4 sm:p-6">
      <PageHeader title="Assignments" description="View and track all module assignments." />

      <Table<Assignment>
        rowKey="id"
        columns={columns}
        dataSource={assignments}
        loading={loading}
        pagination={{
          ...pagination,
          showQuickJumper: true,
          showSizeChanger: true,
        }}
        onChange={handleTableChange}
        className="border border-gray-200 dark:border-gray-800 rounded-lg overflow-hidden"
        onRow={(record) => ({
          onClick: () => navigate(`${record.id}/submissions`),
        })}
      />
    </div>
  );
};

export default AssignmentsList;
