import { Tag, Spin, Empty, Button, Input, Select, Pagination, List } from 'antd';
import { CalendarOutlined, ReloadOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useEffect, useState } from 'react';
import { useModule } from '@/context/ModuleContext';
import { AssignmentsService } from '@/services/assignments';
import { useNotifier } from '@/components/Notifier';
import { useTableQuery } from '@/hooks/useTableQuery';
import dayjs from 'dayjs';
import type { Assignment, AssignmentType } from '@/types/assignments';
import PageHeader from '@/components/PageHeader';

const { Search } = Input;

const ASSIGNMENT_TYPES: AssignmentType[] = ['Assignment', 'Practical'];

const statusColorMap: Record<string, string> = {
  Submitted: 'green',
  Pending: 'orange',
  'Not Started': 'red',
};

const ModuleAssignmentsList = () => {
  const navigate = useNavigate();
  const module = useModule();
  const { notifyError } = useNotifier();

  const {
    searchTerm,
    setSearchTerm,
    pagination,
    setPagination,
    filterState,
    setFilterState,
    clearAll,
  } = useTableQuery();

  const [assignments, setAssignments] = useState<Assignment[]>([]);
  const [loading, setLoading] = useState(false);
  const [total, setTotal] = useState(0);

  const fetchAssignments = async () => {
    setLoading(true);
    const res = await AssignmentsService.listAssignments(module.id, {
      page: pagination.current,
      per_page: pagination.pageSize,
      query: searchTerm,
      assignment_type: filterState.assignment_type?.[0] as AssignmentType | undefined,
    });

    if (res.success) {
      setAssignments(res.data.assignments);
      setTotal(res.data.total);
    } else {
      notifyError('Failed to load assignments', res.message);
    }

    setLoading(false);
  };

  useEffect(() => {
    fetchAssignments();
  }, [module.id, searchTerm, pagination.current, pagination.pageSize, filterState]);

  return (
    <div className="p-4 sm:p-6">
      <div className="w-full max-w-5xl">
        <PageHeader title="Assignments" description="View and track all module assignments." />

        <div className="bg-white dark:bg-gray-900 border border-gray-300 dark:border-gray-600 rounded-lg mb-4 px-4 py-3">
          <div className="flex flex-wrap gap-2 items-center">
            <Search
              placeholder="Search assignments"
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              allowClear
              style={{ width: 240 }}
            />
            <Select
              placeholder="Filter by type"
              value={filterState.assignment_type?.[0]}
              allowClear
              onChange={(value) =>
                setFilterState({
                  ...filterState,
                  assignment_type: value ? [value] : [],
                })
              }
              style={{ width: 180 }}
              options={ASSIGNMENT_TYPES.map((t) => ({ label: t, value: t }))}
            />
            <Button icon={<ReloadOutlined />} onClick={clearAll}>
              Clear
            </Button>
          </div>
        </div>

        {loading ? (
          <div className="flex justify-center py-12">
            <Spin size="large" />
          </div>
        ) : assignments.length === 0 ? (
          <div className="bg-white dark:bg-gray-900 rounded-lg p-6">
            <Empty description="No assignments found." image={Empty.PRESENTED_IMAGE_SIMPLE}>
              <Button icon={<ReloadOutlined />} onClick={fetchAssignments}>
                Reload
              </Button>
            </Empty>
          </div>
        ) : (
          <div className="bg-white dark:bg-gray-900 rounded-lg">
            <List
              bordered
              itemLayout="vertical"
              dataSource={assignments}
              renderItem={(item) => (
                <div className="rounded-md overflow-hidden">
                  <List.Item
                    key={item.id}
                    onClick={() => navigate(`${item.id}/submissions`)}
                    className="cursor-pointer px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-800 transition"
                  >
                    <List.Item.Meta
                      title={
                        <div className="flex justify-between items-start">
                          <span className="font-medium">{item.name}</span>
                          <div className="flex gap-2">
                            <Tag color={statusColorMap[item.status || 'Not Started']}>
                              {item.status || 'Not Started'}
                            </Tag>
                            <Tag color={item.assignment_type === 'Practical' ? 'blue' : 'green'}>
                              {item.assignment_type.charAt(0).toUpperCase() +
                                item.assignment_type.slice(1)}
                            </Tag>
                          </div>
                        </div>
                      }
                      description={
                        <div className="flex items-center text-gray-500 dark:text-gray-400 text-sm mt-1">
                          <CalendarOutlined className="mr-1" />
                          Due {dayjs(item.due_date).format('MMM D, YYYY')}
                        </div>
                      }
                    />
                  </List.Item>
                </div>
              )}
            />
            {total > pagination.pageSize && (
              <div className="flex justify-end p-4">
                <Pagination
                  current={pagination.current}
                  pageSize={pagination.pageSize}
                  total={total}
                  showSizeChanger
                  showQuickJumper
                  onChange={(page, pageSize) => setPagination({ current: page, pageSize })}
                />
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
};

export default ModuleAssignmentsList;
