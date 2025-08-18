import { Row, Col, Space, Typography } from 'antd';
import {
  BarChartOutlined,
  BookOutlined,
  TeamOutlined,
  NotificationOutlined,
} from '@ant-design/icons';
import { useModule } from '@/context/ModuleContext';
import { useAuth } from '@/context/AuthContext';
import { useNavigate } from 'react-router-dom';
import { QuickActions } from '@/components/common';
import { ModuleHeader, ModuleStaffPanel } from '@/components/modules';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useEffect } from 'react';
import { AnnouncementsPanel } from '@/components/announcements';
import { AssignmentsPanel } from '@/components/assignments';
import { GradesPanel } from '@/components/grades';

const ModuleOverview = () => {
  const module = useModule();
  const { setValue } = useViewSlot();
  const { isAdmin, isLecturer } = useAuth();
  const navigate = useNavigate();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Module Overview
      </Typography.Text>,
    );
  }, []);
  const quickActions = [
    {
      key: 'grades',
      label: 'Grades',
      icon: <BarChartOutlined />,
      onClick: () => navigate(`/modules/${module.id}/grades`),
    },
    {
      key: 'resources',
      label: 'Resources',
      icon: <BookOutlined />,
      onClick: () => navigate(`/modules/${module.id}/resources`),
    },
    ...(isAdmin || isLecturer(module.id)
      ? [
          {
            key: 'personnel',
            label: 'Personnel',
            icon: <TeamOutlined />,
            onClick: () => navigate(`/modules/${module.id}/personnel`),
          },
        ]
      : []),
    {
      key: 'announcements',
      label: 'Manage Announcements',
      icon: <NotificationOutlined />,
      onClick: () => console.log('Manage announcements'),
      disabled: true,
    },
  ];

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="p-4 flex-1 overflow-y-auto">
        <Space direction="vertical" size="middle" className="w-full">
          {/* Module Header */}
          <ModuleHeader module={module} />

          <Row gutter={[24, 24]}>
            {/* Left Column */}
            <Col xs={24} lg={16}>
              <Space direction="vertical" size="middle" className="w-full">
                <AnnouncementsPanel />
                <AssignmentsPanel />
                <GradesPanel />
              </Space>
            </Col>

            {/* Right Column */}
            <Col xs={24} lg={8}>
              <Space direction="vertical" size="middle" className="w-full">
                <ModuleStaffPanel />

                <div>
                  <QuickActions actions={quickActions} align="center" />
                </div>
              </Space>
            </Col>
          </Row>
        </Space>
      </div>
    </div>
  );
};

export default ModuleOverview;
