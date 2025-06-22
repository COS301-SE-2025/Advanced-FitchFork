import { useNotifier } from '@/components/Notifier';
import {
  Card,
  Row,
  Col,
  Statistic,
  Tabs,
  Table,
  Button,
  Space,
  Tag,
  Typography,
  Divider,
  List,
} from 'antd';
import {
  BookOutlined,
  UserOutlined,
  TeamOutlined,
  FileTextOutlined,
  PlusOutlined,
  AlertOutlined,
  ClockCircleOutlined,
} from '@ant-design/icons';
import PageHeader from '@/components/PageHeader';

const { Title, Text } = Typography;
const { TabPane } = Tabs;

const dummySubmissions = [
  {
    key: '1',
    student: 'John Doe',
    module: 'COS333',
    assignment: 'Practical 4',
    status: 'Submitted',
  },
  {
    key: '2',
    student: 'Jane Smith',
    module: 'COS344',
    assignment: 'Assignment 2',
    status: 'Pending',
  },
];

const dummyLogs = [
  { key: '1', action: 'Created Module COS332', timestamp: '2025-05-27 10:00' },
  { key: '2', action: 'Edited Assignment 3', timestamp: '2025-05-27 09:45' },
];

const dummyDeadlines = [
  { key: '1', assignment: 'Assignment 2', module: 'COS344', due: '2025-06-01' },
  { key: '2', assignment: 'Practical 4', module: 'COS333', due: '2025-06-03' },
];

const submissionColumns = [
  { title: 'Student', dataIndex: 'student', key: 'student' },
  { title: 'Module', dataIndex: 'module', key: 'module' },
  { title: 'Assignment', dataIndex: 'assignment', key: 'assignment' },
  {
    title: 'Status',
    dataIndex: 'status',
    key: 'status',
    render: (status: string) => (
      <Tag color={status === 'Submitted' ? 'green' : 'orange'}>{status}</Tag>
    ),
  },
];

export default function Home() {
  const { notifyInfo, notifyError, notifySuccess } = useNotifier();

  return (
    <div className="p-4 sm:p-6">
      <PageHeader
        title="Admin Dashboard"
        description="Manage modules, users, assignments, and submissions efficiently."
      />
      <div className="w-full px-4 sm:px-6">
        {/* Header */}
        <Row justify="space-between" align="middle" className="mb-6">
          <Col>
            <Title level={3} className="!mb-0">
              Dashboard Overview
            </Title>
            <Text type="secondary">Welcome back, Admin</Text>
          </Col>
          <Col>
            <Space wrap>
              <Button onClick={() => notifyInfo('Info', 'This is an info message.')}>Info</Button>
              <Button onClick={() => notifyError('Error', 'An error occurred.')}>Error</Button>
              <Button onClick={() => notifySuccess('Success', 'Everything is working.')}>
                Success
              </Button>
            </Space>
          </Col>
        </Row>

        {/* Metrics */}
        <Row gutter={[16, 16]}>
          <Col xs={24} sm={12} md={6}>
            <Card>
              <Statistic title="Modules" value={12} prefix={<BookOutlined />} />
            </Card>
          </Col>
          <Col xs={24} sm={12} md={6}>
            <Card>
              <Statistic title="Users" value={58} prefix={<UserOutlined />} />
            </Card>
          </Col>
          <Col xs={24} sm={12} md={6}>
            <Card>
              <Statistic title="Students" value={134} prefix={<TeamOutlined />} />
            </Card>
          </Col>
          <Col xs={24} sm={12} md={6}>
            <Card>
              <Statistic title="Assignments" value={29} prefix={<FileTextOutlined />} />
            </Card>
          </Col>
        </Row>

        <Divider />

        {/* Tabs */}
        <Tabs defaultActiveKey="1" className="mt-4">
          <TabPane tab="Recent Submissions" key="1">
            <Row gutter={[16, 16]}>
              <Col span={24}>
                <Card>
                  <div className="overflow-x-auto">
                    <Table
                      columns={submissionColumns}
                      dataSource={dummySubmissions}
                      pagination={false}
                    />
                  </div>
                </Card>
              </Col>
            </Row>
          </TabPane>

          <TabPane tab="Module Management" key="2">
            <Row gutter={[16, 16]}>
              <Col span={24}>
                <Card title="Modules" extra={<Button icon={<PlusOutlined />}>Add Module</Button>}>
                  <p>Module management area coming soon.</p>
                </Card>
              </Col>
            </Row>
          </TabPane>

          <TabPane tab="User Management" key="3">
            <Row gutter={[16, 16]}>
              <Col span={24}>
                <Card title="Users" extra={<Button icon={<PlusOutlined />}>Add User</Button>}>
                  <p>User management tools will be integrated here.</p>
                </Card>
              </Col>
            </Row>
          </TabPane>
        </Tabs>

        <Divider />

        {/* Additional Info Panels */}
        <Row gutter={[16, 16]}>
          <Col xs={24} md={12}>
            <Card title="System Logs">
              <List
                itemLayout="horizontal"
                dataSource={dummyLogs}
                renderItem={(log) => (
                  <List.Item>
                    <List.Item.Meta
                      avatar={<AlertOutlined />}
                      title={log.action}
                      description={log.timestamp}
                    />
                  </List.Item>
                )}
              />
            </Card>
          </Col>
          <Col xs={24} md={12}>
            <Card title="Upcoming Deadlines">
              <List
                itemLayout="horizontal"
                dataSource={dummyDeadlines}
                renderItem={(item) => (
                  <List.Item>
                    <List.Item.Meta
                      avatar={<ClockCircleOutlined />}
                      title={`${item.assignment} — ${item.module}`}
                      description={`Due: ${item.due}`}
                    />
                  </List.Item>
                )}
              />
            </Card>
          </Col>
        </Row>
      </div>
    </div>
  );
}
