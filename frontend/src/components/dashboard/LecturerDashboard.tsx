import React from 'react';
import { Row, Col } from 'antd';

import { AssignmentsPanel } from '@/components/assignments';
import { PlagiarismCasesPanel } from '@/components/plagiarism';
import ReleaseChecklistPanel from '@/components/release/ReleaseChecklistPanel';
import { MyModules } from '@/components/modules';
import { TicketsPanel } from '@/components/tickets';

import type { ModuleRole } from '@/types/modules';

const ROLE_LECTURER: ModuleRole = 'lecturer';

const LecturerDashboard: React.FC = () => {
  return (
    <div className="h-full min-h-0">
      {/* Desktop can be fixed by parent; this row will then fill and scroll internally */}
      <Row gutter={[16, 16]} className="h-full">
        {/* LEFT column (xl=12): Assignments (top) + ReleaseChecklist (bottom) */}
        <Col xs={24} xl={12} className="h-full min-h-0 min-w-0">
          <div className="flex flex-col h-full min-h-0 gap-4">
            {/* Give weights if you want, e.g. flex-[1.2] / flex-[0.8] */}
            <div className="flex-1 min-h-0 flex flex-col">
              <AssignmentsPanel
                role={ROLE_LECTURER}
                viewLabels={{ due: 'Open', upcoming: 'Upcoming' }}
              />
            </div>
            <div className="flex-1 min-h-0 flex flex-col">
              <ReleaseChecklistPanel role={ROLE_LECTURER} status="setup" />
            </div>
          </div>
        </Col>

        {/* RIGHT column (xl=12): Plagiarism (top) + MyModules/Tickets (bottom split) */}
        <Col xs={24} xl={12} className="h-full min-h-0 min-w-0">
          <div className="flex flex-col h-full min-h-0 gap-4">
            <div className="flex-1 min-h-0 flex flex-col">
              <PlagiarismCasesPanel role={ROLE_LECTURER} />
            </div>

            <div className="flex-1 min-h-0 flex flex-col">
              {/* Bottom row split: fill remaining height */}
              <div className="flex min-h-0 h-full flex-col md:flex-row gap-4">
                <div className="flex-1 min-h-0 overflow-hidden flex flex-col">
                  <MyModules role={ROLE_LECTURER} />
                </div>
                <div className="flex-1 min-h-0 overflow-hidden flex flex-col">
                  <TicketsPanel role={ROLE_LECTURER} />
                </div>
              </div>
            </div>
          </div>
        </Col>
      </Row>
    </div>
  );
};

export default LecturerDashboard;
