import React from 'react';
import { Row, Col } from 'antd';
import { AssignmentsPanel } from '@/components/assignments';
import { TicketsPanel } from '@/components/tickets';
import { MyModules } from '@/components/modules';
import { AnnouncementsPanel } from '@/components/announcements';

/** Tutor dashboard: 2×2 on xl, stacked on mobile; fills height on desktop. */
const TutorDashboard: React.FC = () => {
  return (
    <div className="h-full min-h-0">
      <Row gutter={[16, 16]} className="h-full">
        {/* LEFT column: Assignments (top), MyModules (bottom) */}
        <Col xs={24} xl={12} className="h-full min-h-0 min-w-0">
          <div className="flex flex-col h-full min-h-0 gap-4">
            <div className="flex-1 min-h-0 flex flex-col overflow-hidden">
              <AssignmentsPanel role="tutor" />
            </div>
            <div className="flex-1 min-h-0 flex flex-col overflow-hidden">
              <MyModules role="tutor" />
            </div>
          </div>
        </Col>

        {/* RIGHT column: Tickets (top), Announcements (bottom) */}
        <Col xs={24} xl={12} className="h-full min-h-0 min-w-0">
          <div className="flex flex-col h-full min-h-0 gap-4">
            <div className="flex-1 min-h-0 flex flex-col overflow-hidden">
              <TicketsPanel role="tutor" />
            </div>
            <div className="flex-1 min-h-0 flex flex-col overflow-hidden">
              <AnnouncementsPanel role="tutor" />
            </div>
          </div>
        </Col>
      </Row>
    </div>
  );
};

export default TutorDashboard;
