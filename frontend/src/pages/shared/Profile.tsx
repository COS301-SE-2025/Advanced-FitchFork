import { useEffect, useState } from 'react';
import AppLayout from '@/layouts/AppLayout';
import {
  Row,
  Col,
  Card,
  Avatar,
  Tag,
  Button,
  List,
  Descriptions,
  Divider,
  Typography,
  Switch,
} from 'antd';
import {
  MailOutlined,
  IdcardOutlined,
  SafetyCertificateOutlined,
  UserOutlined,
  CalendarOutlined,
  EditOutlined,
  LockOutlined,
  MobileOutlined,
  KeyOutlined,
} from '@ant-design/icons';
import { AuthService } from '@/services/auth';
import type { MeResponse, UserModule } from '@/types/auth';

const { Title, Text } = Typography;

const mockModules: UserModule[] = [
  {
    id: 1,
    code: 'COS332',
    year: 2025,
    description: 'Networks and Security',
    role: 'Tutor',
    created_at: '',
    updated_at: '',
    credits: 0,
  },
  {
    id: 2,
    code: 'COS344',
    year: 2025,
    description: 'Computer Graphics',
    role: 'Student',
    created_at: '',
    updated_at: '',
    credits: 0,
  },
  {
    id: 3,
    code: 'COS301',
    year: 2024,
    description: 'Software Engineering',
    role: 'Lecturer',
    created_at: '',
    updated_at: '',
    credits: 0,
  },
];

export default function ProfilePage() {
  const [user, setUser] = useState<MeResponse | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    AuthService.me()
      .then((res) => {
        if (res.success && res.data) setUser(res.data);
      })
      .finally(() => setLoading(false));
  }, []);

  if (loading || !user) return null;

  const userRoles = Array.from(new Set(user.modules.map((m) => m.role)));
  const visibleModules = user.modules.length === 0 ? mockModules : user.modules;

  return (
    <AppLayout title="Profile" description="Manage your account, roles, and settings.">
      <div className="w-full max-w-6xl">
        <Row gutter={24}>
          <Col xs={24} md={8}>
            <Card className="rounded-lg mb-6 border border-gray-300">
              <div className="flex flex-col gap-4">
                <Avatar size={80} src="/profile.jpeg" icon={<UserOutlined />} />
                <div>
                  <Title level={4} className="!mb-0">
                    {user.student_number}
                  </Title>
                  <Text type="secondary">{user.email}</Text>
                </div>
                <div className="flex flex-wrap gap-2">
                  {user.admin && (
                    <Tag icon={<SafetyCertificateOutlined />} color="blue">
                      Admin
                    </Tag>
                  )}
                  {userRoles.includes('Lecturer') && <Tag color="volcano">Lecturer</Tag>}
                  {userRoles.includes('Tutor') && <Tag color="geekblue">Tutor</Tag>}
                  {userRoles.includes('Student') && <Tag color="green">Student</Tag>}
                </div>
                <Button icon={<EditOutlined />} type="default">
                  Edit Profile
                </Button>
              </div>
            </Card>

            <Card className="rounded-lg border border-gray-300 mt-6" title="Recent Activity">
              <List
                size="small"
                dataSource={[
                  { date: '2025-05-24', activity: 'Logged in' },
                  { date: '2025-05-22', activity: 'Viewed Module COS344' },
                ]}
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
          </Col>

          <Col xs={24} md={16}>
            <Card className="rounded-lg border border-gray-300" title="Account Information">
              <Descriptions
                column={1}
                labelStyle={{ fontWeight: 500, minWidth: 150 }}
                layout="horizontal"
              >
                <Descriptions.Item label="Student Number">
                  <IdcardOutlined className="mr-1" />
                  {user.student_number}
                </Descriptions.Item>
                <Descriptions.Item label="Email">
                  <MailOutlined className="mr-1" />
                  {user.email}
                </Descriptions.Item>
                <Descriptions.Item label="Admin">{user.admin ? 'Yes' : 'No'}</Descriptions.Item>
              </Descriptions>
            </Card>
          </Col>
        </Row>

        <Divider className="my-8" />

        <Row>
          <Col span={24}>
            <Card className="rounded-lg border border-gray-300" title="Security Settings">
              <List
                itemLayout="horizontal"
                dataSource={[
                  {
                    title: 'Password',
                    description: 'Last changed 3 months ago',
                    icon: <LockOutlined />,
                    action: <Button type="link">Change</Button>,
                  },
                  {
                    title: 'Two-Factor Authentication',
                    description: '2FA is currently disabled',
                    icon: <KeyOutlined />,
                    action: <Switch defaultChecked={false} />,
                  },
                  {
                    title: 'Logged-in Devices',
                    description: '3 active sessions',
                    icon: <MobileOutlined />,
                    action: <Button type="link">Manage</Button>,
                  },
                ]}
                renderItem={(item) => (
                  <List.Item actions={[item.action]}>
                    <List.Item.Meta
                      avatar={<Avatar icon={item.icon} />}
                      title={item.title}
                      description={item.description}
                    />
                  </List.Item>
                )}
              />
            </Card>
          </Col>
        </Row>

        <Divider className="my-8" />

        <Row>
          <Col span={24}>
            <Card className="rounded-lg border border-gray-300" title="Enrolled Modules">
              <List
                itemLayout="horizontal"
                dataSource={visibleModules}
                renderItem={(mod) => (
                  <List.Item>
                    <List.Item.Meta
                      title={`${mod.code} (${mod.year})`}
                      description={mod.description}
                    />
                    <Tag
                      color={
                        mod.role === 'Lecturer'
                          ? 'volcano'
                          : mod.role === 'Tutor'
                            ? 'geekblue'
                            : 'green'
                      }
                      style={{ fontWeight: 500 }}
                    >
                      {mod.role}
                    </Tag>
                  </List.Item>
                )}
              />
            </Card>
          </Col>
        </Row>
      </div>
    </AppLayout>
  );
}
