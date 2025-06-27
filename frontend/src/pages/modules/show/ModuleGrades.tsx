import { Table, Typography, Tag, Button } from 'antd';
import { FileTextOutlined } from '@ant-design/icons';

const { Title, Text } = Typography;

const ModuleGrades = () => {
  const columns = [
    {
      title: 'Student',
      dataIndex: 'student',
      key: 'student',
      render: (text: string) => <Text strong>{text}</Text>,
    },
    {
      title: 'Assignment',
      dataIndex: 'assignment',
      key: 'assignment',
    },
    {
      title: 'Grade',
      dataIndex: 'grade',
      key: 'grade',
      render: (grade: number) => (
        <Tag color={grade >= 75 ? 'green' : grade >= 50 ? 'orange' : 'red'}>{grade}%</Tag>
      ),
    },
    {
      title: '',
      key: 'actions',
      render: () => (
        <Button type="link" size="small" icon={<FileTextOutlined />}>
          View
        </Button>
      ),
    },
  ];

  const data = [
    {
      key: 1,
      student: 'John Doe',
      assignment: 'Practical 1',
      grade: 82,
    },
    {
      key: 2,
      student: 'Jane Smith',
      assignment: 'Practical 1',
      grade: 68,
    },
    {
      key: 3,
      student: 'Alex Kim',
      assignment: 'Practical 1',
      grade: 45,
    },
  ];

  return (
    <div className="space-y-6 p-4 sm:p-6">
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
        <Title level={3} className="!mb-0">
          Grades
        </Title>
        <Button type="default">Export CSV</Button>
      </div>

      <Table
        columns={columns}
        dataSource={data}
        pagination={{ pageSize: 5 }}
        className="bg-white dark:bg-neutral-900 rounded-md"
      />
    </div>
  );
};

export default ModuleGrades;
