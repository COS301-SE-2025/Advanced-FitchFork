import { Row, Col, Typography, List, Button, Divider, Card, Empty, Tag } from 'antd';
import {
  FileTextOutlined,
  UserOutlined,
  IdcardOutlined,
  BarChartOutlined,
  BookOutlined,
  TeamOutlined,
  NotificationOutlined,
} from '@ant-design/icons';
import { useModule } from '@/context/ModuleContext';
import { useAuth } from '@/context/AuthContext';
import { useNavigate } from 'react-router-dom';

const { Title, Text, Paragraph } = Typography;

const ModuleOverview = () => {
  const module = useModule();
  const { isAdmin, isLecturer } = useAuth();
  const navigate = useNavigate();

  const allStaff = [
    ...module.lecturers.map((user) => ({ ...user, role: 'Lecturer' })),
    ...module.tutors.map((user) => ({ ...user, role: 'Tutor' })),
  ];

  const renderStaffList = (users: typeof allStaff) =>
    users.length === 0 ? (
      <Empty image={Empty.PRESENTED_IMAGE_SIMPLE} description="None assigned" />
    ) : (
      <List
        size="small"
        dataSource={users}
        renderItem={(user) => (
          <List.Item className="!px-0">
            <div className="flex items-center justify-between w-full">
              <div className="flex items-center gap-2">
                <UserOutlined />
                <span className="font-medium">{user.email}</span>
              </div>
              <div className="flex items-center gap-2 text-gray-500 dark:text-gray-400 text-sm">
                <Tag color={user.role === 'Lecturer' ? 'blue' : 'purple'}>{user.role}</Tag>
                <IdcardOutlined />
                <span>{user.username}</span>
              </div>
            </div>
          </List.Item>
        )}
      />
    );

  return (
    <div className="p-4 sm:p-6">
      {/* Module Header */}
      <div className="mb-6">
        <Title level={3} className="!mb-1">
          {module.code} · {module.description}
        </Title>
        <Text type="secondary">Academic Year: {module.year}</Text>
        <div className="mt-2 flex flex-wrap gap-2">
          <Tag color="blue">Semester 2</Tag>
        </div>
        <Divider className="!my-4" />
        <Paragraph>This module is worth {module.credits} credits.</Paragraph>
      </div>

      <Row gutter={[24, 24]}>
        {/* Left Column */}
        <Col xs={24} lg={16}>
          <Card
            title={<span>Upcoming Assignments</span>}
            extra={
              <Button
                onClick={() => navigate(`/modules/${module.id}/assignments`)}
                type="default"
                icon={<FileTextOutlined />}
              >
                View All
              </Button>
            }
            className="!mb-6"
          >
            <Empty description="No upcoming assignments." />
          </Card>

          <Card title="Recent Announcements" className="!mb-6">
            <Empty description="No announcements yet." />
          </Card>
        </Col>

        {/* Right Column */}
        <Col xs={24} lg={8}>
          <Card title="Module Staff" className="!mb-6">
            {renderStaffList(allStaff)}
          </Card>

          <Card title="Quick Access" className="!mb-6">
            <div className="flex flex-col gap-2">
              <Button
                block
                icon={<BarChartOutlined />}
                onClick={() => navigate(`/modules/${module.id}/grades`)}
              >
                Grades
              </Button>
              <Button
                block
                icon={<BookOutlined />}
                onClick={() => navigate(`/modules/${module.id}/resources`)}
              >
                Resources
              </Button>
              {(isAdmin || isLecturer(module.id)) && (
                <Button
                  block
                  icon={<TeamOutlined />}
                  onClick={() => navigate(`/modules/${module.id}/personnel`)}
                >
                  Personnel
                </Button>
              )}
              <Button block icon={<NotificationOutlined />} disabled>
                Manage Announcements
              </Button>
            </div>
          </Card>
        </Col>
      </Row>
    </div>
  );
};

export default ModuleOverview;
