import React, { type PropsWithChildren } from 'react';
import { Row, Col, Card, Statistic } from 'antd';
import { ClockCircleOutlined, CheckCircleOutlined, TeamOutlined } from '@ant-design/icons';
import SubmissionAnalyticsPanel from '../submissions/SubmissionAnalyticsPanel';

type PlaceholderProps = PropsWithChildren<{ title: string }>;

const PlaceholderPanel: React.FC<PlaceholderProps> = ({ title, children }) => (
  <Card
    className="h-full rounded-2xl flex flex-col"
    styles={{ body: { display: 'flex', flexDirection: 'column', minHeight: 0, padding: 12 } }}
    title={<span className="font-semibold">{title}</span>}
  >
    <div className="flex-1 min-h-0">
      <div className="h-full rounded-lg border border-dashed border-gray-300 dark:border-gray-700 p-4 text-center text-gray-500 overflow-auto">
        {children ?? 'Coming soon'}
      </div>
    </div>
  </Card>
);

/** Simple Statistic tile that expands to its grid cell */
const StatsTile: React.FC<{
  title: string;
  value: number;
  icon: React.ReactNode;
  suffix?: React.ReactNode;
}> = ({ title, value, icon, suffix }) => (
  <Card className="h-full rounded-2xl" styles={{ body: { height: '100%', padding: 16 } }}>
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

const LecturerDashboard: React.FC = () => {
  // placeholder numbers for now
  const upcomingAssignments: number = 6;
  const unmarkedSubmissions: number = 14;
  const modulesYouLecturer: number = 3;

  return (
    <div className="h-full min-h-0">
      <Row gutter={[16, 16]} className="h-full">
        {/* Top-left: Submission Analytics */}
        <Col xs={24} md={14} className="min-h-0">
          <div className="h-full min-h-0 flex flex-col">
            <div className="flex-1 min-h-0">
              <SubmissionAnalyticsPanel />
            </div>
          </div>
        </Col>
        {/* Top-right: Plagiarism Cases */}
        <Col xs={24} md={10} className="min-h-0">
          <div className="h-full min-h-0 flex flex-col">
            <div className="flex-1 min-h-0">
              <PlaceholderPanel title="Plagiarism Cases" />
            </div>
          </div>
        </Col>

        {/* Bottom-left: Upcoming Assignments + My Modules */}
        <Col xs={24} md={12} className="min-h-0">
          <div className="h-full min-h-0 flex flex-col gap-3">
            <div className="flex-1 min-h-0">
              <PlaceholderPanel title="Upcoming Assignments" />
            </div>
            <div className="flex-1 min-h-0">
              <PlaceholderPanel title="My Modules" />
            </div>
          </div>
        </Col>

        {/* Bottom-right: split into 2 columns → left stats, right quick actions + release checklist */}
        <Col xs={24} md={12} className="min-h-0">
          <div className="h-full min-h-0 grid grid-cols-1 md:grid-cols-2 gap-3">
            {/* Left column: vertical stats strip */}
            <div className="min-h-0">
              <div className="h-full grid grid-rows-3 gap-3">
                <StatsTile
                  title="Upcoming assignments"
                  value={upcomingAssignments}
                  icon={<ClockCircleOutlined />}
                  suffix={upcomingAssignments === 1 ? 'item' : 'items'}
                />
                <StatsTile
                  title="Unmarked submissions"
                  value={unmarkedSubmissions}
                  icon={<CheckCircleOutlined />}
                  suffix={unmarkedSubmissions === 1 ? 'submission' : 'submissions'}
                />
                <StatsTile
                  title="Modules you lecture"
                  value={modulesYouLecturer}
                  icon={<TeamOutlined />}
                  suffix={modulesYouLecturer === 1 ? 'module' : 'modules'}
                />
              </div>
            </div>

            {/* Right column: Quick Actions (top) + Release Checklist (bottom) */}
            <div className="min-h-0 flex flex-col gap-3">
              <div className="flex-1 min-h-0">
                <PlaceholderPanel title="Quick Actions" />
              </div>
              <div className="flex-1 min-h-0">
                <PlaceholderPanel title="Release Checklist" />
              </div>
            </div>
          </div>
        </Col>
      </Row>
    </div>
  );
};

export default LecturerDashboard;
