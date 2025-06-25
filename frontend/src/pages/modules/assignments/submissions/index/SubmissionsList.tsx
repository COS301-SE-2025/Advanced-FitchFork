import { Table, Typography, Tag, Button, Dropdown } from 'antd';
import { MoreOutlined, EyeOutlined, DownloadOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import type { ColumnsType } from 'antd/es/table';
import dayjs from 'dayjs';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';

const { Title, Text } = Typography;

type SubmissionStatus = 'Pending' | 'Graded';

type StudentSubmission = {
  id: number;
  attempt: number;
  filename: string;
  path: string;
  created_at: string;
  updated_at: string;
  status: SubmissionStatus;
  mark?: number;
};

const mockSubmissions: StudentSubmission[] = [
  {
    id: 1,
    attempt: 1,
    filename: 'assignment1_draft.pdf',
    path: '/submissions/assignment1_draft.pdf',
    created_at: '2025-06-10T09:00:00Z',
    updated_at: '2025-06-10T09:00:00Z',
    status: 'Pending',
  },
  {
    id: 2,
    attempt: 2,
    filename: 'assignment1_final.pdf',
    path: '/submissions/assignment1_final.pdf',
    created_at: '2025-06-12T18:30:00Z',
    updated_at: '2025-06-12T18:45:00Z',
    status: 'Graded',
    mark: 82,
  },
  {
    id: 3,
    attempt: 3,
    filename: 'assignment1_revised.pdf',
    path: '/submissions/assignment1_revised.pdf',
    created_at: '2025-06-13T08:15:00Z',
    updated_at: '2025-06-13T08:16:00Z',
    status: 'Graded',
    mark: 49,
  },
  {
    id: 4,
    attempt: 4,
    filename: 'assignment1_best.pdf',
    path: '/submissions/assignment1_best.pdf',
    created_at: '2025-06-14T14:00:00Z',
    updated_at: '2025-06-14T14:05:00Z',
    status: 'Graded',
    mark: 95,
  },
  {
    id: 5,
    attempt: 5,
    filename: 'assignment1_pending_review.pdf',
    path: '/submissions/assignment1_pending_review.pdf',
    created_at: '2025-06-15T10:30:00Z',
    updated_at: '2025-06-15T10:31:00Z',
    status: 'Pending',
  },
];

const getMarkColor = (mark: number): string => {
  if (mark >= 75) return 'green';
  if (mark >= 50) return 'orange';
  return 'red';
};

const SubmissionsList = () => {
  const navigate = useNavigate();
  const module = useModule();
  const assignment = useAssignment();

  const columns: ColumnsType<StudentSubmission> = [
    {
      title: 'Attempt',
      dataIndex: 'attempt',
      key: 'attempt',
      render: (attempt) => <Tag color="blue">#{attempt}</Tag>,
    },
    {
      title: 'Filename',
      dataIndex: 'filename',
      key: 'filename',
    },
    {
      title: 'Submitted At',
      dataIndex: 'created_at',
      key: 'created_at',
      render: (value) => dayjs(value).format('YYYY-MM-DD HH:mm'),
    },
    {
      title: 'Status',
      dataIndex: 'status',
      key: 'status',
      render: (status) => <Tag color={status === 'Graded' ? 'green' : 'default'}>{status}</Tag>,
    },
    {
      title: 'Mark (%)',
      dataIndex: 'mark',
      key: 'mark',
      render: (_, record) => {
        if (record.status === 'Graded' && typeof record.mark === 'number') {
          const color = getMarkColor(record.mark);
          return <Tag color={color}>{record.mark}%</Tag>;
        } else {
          return <Tag color="default">Not marked</Tag>;
        }
      },
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
                  label: 'View',
                },
                {
                  key: 'download',
                  icon: <DownloadOutlined />,
                  label: 'Download',
                },
              ],
              onClick: ({ key, domEvent }) => {
                domEvent.stopPropagation();
                if (key === 'view') {
                  navigate(
                    `/modules/${module.id}/assignments/${assignment.id}/submissions/${record.id}`,
                  );
                } else if (key === 'download') {
                  window.open(record.path, '_blank');
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

  return (
    <div className="max-w-4xl">
      <div className="flex items-center justify-between mb-2">
        <div>
          <Title level={4} className="mb-0">
            Your Submissions
          </Title>
          <Text className="text-gray-500 dark:text-gray-400">
            Below are your attempts for this assignment.
          </Text>
        </div>
        <Button
          type="primary"
          onClick={() => navigate(`/modules/${module.id}/assignments/${assignment.id}/submit`)}
        >
          New Submission
        </Button>
      </div>

      <Table<StudentSubmission>
        columns={columns}
        dataSource={mockSubmissions}
        rowKey="id"
        pagination={{ pageSize: 5 }}
        className="border-1 border-gray-200 dark:border-gray-800 rounded-lg overflow-hidden"
        onRow={(record) => ({
          onClick: () =>
            navigate(`/modules/${module.id}/assignments/${assignment.id}/submissions/${record.id}`),
        })}
      />
    </div>
  );
};

export default SubmissionsList;
