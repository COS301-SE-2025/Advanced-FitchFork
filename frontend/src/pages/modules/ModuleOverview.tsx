import { Row, Col, Space, Typography } from 'antd';
import { useModule } from '@/context/ModuleContext';
import { ModuleHeader, ModuleStaffPanel } from '@/components/modules';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useEffect } from 'react';
import { AnnouncementsPanel } from '@/components/announcements';
import { AssignmentsPanel } from '@/components/assignments';
import { GradesPanel } from '@/components/grades';

const ANNOUNCEMENT_PREVIEW_LIMIT = 5;
const ASSIGNMENT_PREVIEW_LIMIT = 6;
const GRADES_PREVIEW_LIMIT = 10;

const ModuleOverview = () => {
  const module = useModule();
  const { setValue } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Module Overview
      </Typography.Text>,
    );
  }, []);

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
                <AnnouncementsPanel
                  moduleId={module.id}
                  limit={ANNOUNCEMENT_PREVIEW_LIMIT}
                  minimal
                />
                <AssignmentsPanel
                  moduleId={module.id}
                  limit={ASSIGNMENT_PREVIEW_LIMIT}
                  minimal
                />
              </Space>
            </Col>

            {/* Right Column */}
            <Col xs={24} lg={8}>
              <Space direction="vertical" size="middle" className="w-full">
                <ModuleStaffPanel />
                <GradesPanel moduleId={module.id} limit={GRADES_PREVIEW_LIMIT} minimal />
              </Space>
            </Col>
          </Row>
        </Space>
      </div>
    </div>
  );
};

export default ModuleOverview;
