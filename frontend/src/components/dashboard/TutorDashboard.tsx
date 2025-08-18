import React from 'react';
import { Row, Col, Card, Statistic } from 'antd';
import { SubmissionsPanel } from '@/components/submissions';
import { TicketsPanel } from '@/components/tickets';
import { MyModules } from '@/components/modules';
import {
  MessageOutlined,
  ClockCircleOutlined,
  CheckCircleOutlined,
  TeamOutlined,
} from '@ant-design/icons';
import { GradesPanel } from '../grades';

/** Small tile using AntD Card + Statistic that stretches to fill its grid cell */
const StatsTile: React.FC<{
  title: string;
  value: number;
  icon: React.ReactNode;
  suffix?: React.ReactNode;
}> = ({ title, value, icon, suffix }) => (
  <Card
    className="h-full rounded-2xl !border-gray-200 dark:!border-gray-800"
    styles={{ body: { height: '100%', padding: 16 } }}
  >
    <div className="h-full flex items-start gap-3 min-w-0">
      <div className="shrink-0">{icon}</div>
      <div className="flex-1 min-w-0">
        <Statistic
          title={<span className="text-sm text-gray-500">{title}</span>}
          value={value}
          suffix={suffix}
        />
      </div>
    </div>
  </Card>
);

/** Tutor dashboard: 2×2 grid (md+), stacked on mobile. */
const TutorDashboard: React.FC = () => {
  const openTickets: number = 7;
  const activeAssignments: number = 4;
  const pendingReviews: number = 11;
  const tutorModuleCount: number = 3;

  return (
    <div className="h-full min-h-0">
      <Row gutter={[16, 16]} className="h-full">
        {/* Top-left */}
        <Col xs={24} md={12} className="min-h-0">
          <div className="h-full min-h-0 flex flex-col">
            <div className="flex-1 min-h-0">
              <SubmissionsPanel />
            </div>
          </div>
        </Col>

        {/* Top-right */}
        <Col xs={24} md={12} className="min-h-0">
          <div className="h-full min-h-0 flex flex-col">
            <div className="flex-1 min-h-0">
              <TicketsPanel />
            </div>
          </div>
        </Col>

        {/* Bottom-left — split horizontally: left Modules, right single-column Stats */}
        <Col xs={24} md={12} className="min-h-0">
          <div className="h-full min-h-0 grid grid-cols-1 md:grid-cols-3 gap-3">
            {/* Left (2/3): My Modules */}
            <div className="min-h-0 md:col-span-2">
              <div className="h-full min-h-0">
                <MyModules scope="tutor" />
              </div>
            </div>

            {/* Right (1/3): Stats stacked vertically */}
            <div className="min-h-0">
              <div className="h-full grid grid-rows-4 gap-3">
                <div className="min-h-0">
                  <StatsTile
                    title="Open tickets"
                    value={openTickets}
                    icon={<MessageOutlined />}
                    suffix={openTickets === 1 ? 'ticket' : 'tickets'}
                  />
                </div>
                <div className="min-h-0">
                  <StatsTile
                    title="Active assignments"
                    value={activeAssignments}
                    icon={<ClockCircleOutlined />}
                    suffix={activeAssignments === 1 ? 'assignment' : 'assignments'}
                  />
                </div>
                <div className="min-h-0">
                  <StatsTile
                    title="Pending reviews"
                    value={pendingReviews}
                    icon={<CheckCircleOutlined />}
                    suffix={pendingReviews === 1 ? 'submission' : 'submissions'}
                  />
                </div>
                <div className="min-h-0">
                  <StatsTile
                    title="Modules you tutor"
                    value={tutorModuleCount}
                    icon={<TeamOutlined />}
                    suffix={tutorModuleCount === 1 ? 'module' : 'modules'}
                  />
                </div>
              </div>
            </div>
          </div>
        </Col>

        {/* Bottom-right (unchanged) */}
        <Col xs={24} md={12} className="min-h-0">
          <div className="h-full min-h-0 flex flex-col">
            <div className="flex-1 min-h-0">
              <GradesPanel />
            </div>
          </div>
        </Col>
      </Row>
    </div>
  );
};

export default TutorDashboard;
