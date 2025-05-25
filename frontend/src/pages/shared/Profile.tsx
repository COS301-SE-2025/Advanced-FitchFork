import AppLayout from '@/layouts/AppLayout';
import { Typography, Row, Col, Avatar, Descriptions, Tag, Divider, Card, List, Button } from 'antd';
import {
  MailOutlined,
  IdcardOutlined,
  SafetyCertificateOutlined,
  UserOutlined,
  CalendarOutlined,
  EditOutlined,
} from '@ant-design/icons';

const { Title, Text } = Typography;

const mockUser = {
  fullName: 'Jane Doe',
  studentNumber: 'u20231234',
  email: 'jane.doe@example.com',
  admin: true,
  joinedAt: '2024-02-15',
  phone: '+27 81 234 5678',
  roles: ['Lecturer', 'Tutor'],
  department: 'Computer Science',
  lastLogin: '2025-05-22 14:35',
};

const mockActivity = [
  { date: '2025-05-22', activity: 'Logged in from new device' },
  { date: '2025-05-20', activity: 'Assigned as Tutor to COS332' },
  { date: '2025-05-18', activity: 'Updated email address' },
];

export default function ProfilePage() {
  return (
    <AppLayout title="Profile" description="Manage your account, roles, and settings.">
      <div className=" w-full max-w-6xl">
        <Row gutter={24}>
          {/* Left Panel */}
          <Col xs={24} md={8}>
            {/* Profile Card */}
            <Card className="rounded-lg mb-6 border border-gray-300">
              <div className="flex flex-col gap-4">
                <Avatar size={80} icon={<UserOutlined />} />
                <div>
                  <Title level={4} className="!mb-0">
                    {mockUser.fullName}
                  </Title>
                  <Text type="secondary">{mockUser.email}</Text>
                </div>
                <div>
                  {mockUser.admin && (
                    <Tag icon={<SafetyCertificateOutlined />} color="blue">
                      Admin
                    </Tag>
                  )}
                  {mockUser.roles.map((role) => (
                    <Tag key={role} color="green">
                      {role}
                    </Tag>
                  ))}
                </div>
                <Button icon={<EditOutlined />} type="default">
                  Edit Profile
                </Button>
              </div>
            </Card>

            {/* Activity Card */}
            <div className="mt-6">
              <Card className="rounded-lg border border-gray-300" title="Recent Activity">
                <List
                  size="small"
                  dataSource={mockActivity}
                  renderItem={(item) => (
                    <List.Item>
                      <Text>
                        <CalendarOutlined className="mr-2" />
                        {item.date} — {item.activity}
                      </Text>
                    </List.Item>
                  )}
                />
              </Card>
            </div>
          </Col>

          {/* Right Panel */}
          <Col xs={24} md={16}>
            <Card className="rounded-lg border border-gray-300" title="Account Information">
              <Descriptions
                column={1}
                labelStyle={{ fontWeight: 500, minWidth: 150 }}
                layout="horizontal"
              >
                <Descriptions.Item label="Full Name">{mockUser.fullName}</Descriptions.Item>
                <Descriptions.Item label="Student Number">
                  <IdcardOutlined className="mr-1" />
                  {mockUser.studentNumber}
                </Descriptions.Item>
                <Descriptions.Item label="Email">
                  <MailOutlined className="mr-1" />
                  {mockUser.email}
                </Descriptions.Item>
                <Descriptions.Item label="Phone Number">{mockUser.phone}</Descriptions.Item>
                <Descriptions.Item label="Department">{mockUser.department}</Descriptions.Item>
                <Descriptions.Item label="Account Created">{mockUser.joinedAt}</Descriptions.Item>
                <Descriptions.Item label="Last Login">{mockUser.lastLogin}</Descriptions.Item>
              </Descriptions>
            </Card>
          </Col>
        </Row>

        <Divider className="my-8" />

        <Row>
          <Col span={24}>
            <Card
              className="rounded-lg border border-gray-300"
              title="Security Settings (Coming Soon)"
            >
              <Text type="secondary">
                You'll be able to manage your password, 2FA, and connected devices here.
              </Text>
            </Card>
          </Col>
        </Row>
      </div>
    </AppLayout>
  );
}
