import { Row, Col, Typography, List, Button, Divider } from 'antd';
import { CalendarOutlined, FileTextOutlined } from '@ant-design/icons';

const { Title, Text, Paragraph } = Typography;

const ModuleOverview = () => {
  const moduleInfo = {
    code: 'COS344',
    name: 'Computer Graphics',
    year: 2025,
    description:
      'This module covers essential concepts in 2D and 3D graphics, including transformations, rendering, and shading using OpenGL and C++.',
    lecturers: ['Dr. Cobus Redelinghuys'],
    tutors: ['Jane Doe', 'John Smith'],
    assignments: [
      { title: 'Practical 1: Transformations', due: '2025-03-10' },
      { title: 'Practical 2: Shading', due: '2025-04-14' },
    ],
  };

  return (
    <div className="space-y-6 p-4 sm:p-6">
      {/* Module Header */}
      <div>
        <Title level={3} className="!mb-1">
          {moduleInfo.code} · {moduleInfo.name}
        </Title>
        <Text type="secondary">Academic Year: {moduleInfo.year}</Text>
        <Divider className="!my-4" />
        <Paragraph>{moduleInfo.description}</Paragraph>
      </div>

      {/* Two-Column Info */}
      <Row gutter={32}>
        <Col xs={24} md={12}>
          <Title level={5}>Lecturer</Title>
          <List
            bordered
            dataSource={moduleInfo.lecturers}
            renderItem={(name) => <List.Item>{name}</List.Item>}
            className="rounded-md"
          />
        </Col>
        <Col xs={24} md={12}>
          <Title level={5}>Tutors</Title>
          <List
            bordered
            dataSource={moduleInfo.tutors}
            renderItem={(name) => <List.Item>{name}</List.Item>}
            className="rounded-md"
          />
        </Col>
      </Row>

      {/* Assignments */}
      <div>
        <Title level={5}>Upcoming Assignments</Title>
        <List
          bordered
          dataSource={moduleInfo.assignments}
          renderItem={(item) => (
            <List.Item>
              <div className="flex flex-col">
                <Text strong>{item.title}</Text>
                <Text type="secondary">
                  <CalendarOutlined className="mr-1" />
                  Due {item.due}
                </Text>
              </div>
            </List.Item>
          )}
          className="rounded-md"
        />
      </div>

      {/* Navigation */}
      <div className="text-right pt-2">
        <Button type="default" icon={<FileTextOutlined />} size="middle">
          View All Assignments
        </Button>
      </div>
    </div>
  );
};

export default ModuleOverview;
