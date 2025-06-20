import { Typography, List, Tag, Button } from 'antd';
import { CalendarOutlined, EyeOutlined, PlusOutlined } from '@ant-design/icons';

const { Title, Text } = Typography;

const ModuleAssignments = () => {
  const assignments = [
    {
      title: 'Practical 1: Transformations',
      due: '2025-03-10',
      status: 'Submitted',
    },
    {
      title: 'Practical 2: Shading',
      due: '2025-04-14',
      status: 'Pending',
    },
    {
      title: 'Practical 3: Lighting',
      due: '2025-05-08',
      status: 'Not Started',
    },
  ];

  const statusColorMap: Record<string, string> = {
    Submitted: 'green',
    Pending: 'orange',
    'Not Started': 'red',
  };

  return (
    <div className="space-y-6 p-4 sm:p-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
        <Title level={3} className="!mb-0">
          Assignments
        </Title>
        <Button type="default" icon={<PlusOutlined />}>
          New Assignment
        </Button>
      </div>

      {/* Assignments List */}
      <List
        bordered
        dataSource={assignments}
        className="rounded-md"
        renderItem={(item) => (
          <List.Item className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2">
            <div>
              <Text strong>{item.title}</Text>
              <div className="text-gray-500 dark:text-gray-400">
                <CalendarOutlined className="mr-1" />
                Due {item.due}
              </div>
            </div>
            <div className="flex items-center gap-3">
              <Tag color={statusColorMap[item.status]}>{item.status}</Tag>
              <Button size="small" icon={<EyeOutlined />}>
                View
              </Button>
            </div>
          </List.Item>
        )}
      />
    </div>
  );
};

export default ModuleAssignments;
