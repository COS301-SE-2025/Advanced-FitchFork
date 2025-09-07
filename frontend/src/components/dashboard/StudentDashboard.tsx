import React from 'react';
import { Row, Col } from 'antd';
import { AssignmentsPanel } from '@/components/assignments';
import { SubmissionsPanel } from '@/components/submissions';
import { GradesPanel } from '@/components/grades';
import { TicketsPanel } from '@/components/tickets';
import { AnnouncementsPanel } from '@/components/announcements';
import { MyModules } from '@/components/modules';

/** Student dashboard layout: 3 columns on lg+, stacked on mobile. */
const StudentDashboard: React.FC = () => {
  return (
    <div className="h-full min-h-0">
      <Row gutter={[16, 16]} className="h-full">
        {/* LEFT: Assignments + Submissions */}
        <Col xs={24} lg={10} className="lg:h-full h-auto min-h-0 min-w-0">
          <div className="min-h-0 flex flex-col lg:h-full gap-4 h-auto">
            <div className="lg:flex-[1.2] min-h-0">
              <div className="min-h-0 lg:h-full h-auto">
                <AssignmentsPanel />
              </div>
            </div>
            <div className="flex-1 min-h-0 hidden lg:block">
              <SubmissionsPanel />
            </div>
          </div>
        </Col>

        {/* MIDDLE: MyModules + Grades + Tickets */}
        <Col xs={24} lg={7} className="lg:h-full h-auto min-h-0 min-w-0">
          <div className="min-h-0 flex flex-col lg:h-full gap-4 h-auto">
            <div className="lg:flex-[0.9] min-h-0 hidden lg:block">
              <div className="lg:h-full h-auto min-h-0">
                <MyModules />
              </div>
            </div>
            <div className="lg:flex-[0.8] min-h-0">
              <div className="lg:h-full h-auto min-h-0">
                <GradesPanel />
              </div>
            </div>
            <div className="lg:flex-[0.8] min-h-0">
              <div className="lg:h-full h-auto min-h-0">
                <TicketsPanel role="student" />
              </div>
            </div>
          </div>
        </Col>

        {/* RIGHT: Announcements */}
        <Col xs={24} lg={7} className="lg:h-full h-auto min-h-0 min-w-0">
          <div className="lg:h-full h-auto min-h-0">
            <AnnouncementsPanel />
          </div>
        </Col>
      </Row>
    </div>
  );
};

export default StudentDashboard;
