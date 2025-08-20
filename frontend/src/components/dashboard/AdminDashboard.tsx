import React, { type PropsWithChildren } from 'react';
import { Row, Col, Card, Statistic } from 'antd';
import { TeamOutlined, CloudServerOutlined, SafetyCertificateOutlined } from '@ant-design/icons';

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

const AdminDashboard: React.FC = () => {
  // placeholder numbers for now
  const activeUsers: number = 124;
  const runningServices: number = 18;
  const openIncidents: number = 2;

  return (
    <div className="h-full min-h-0">
      <Row gutter={[16, 16]} className="h-full">
        {/* Top-left: Platform Health */}
        <Col xs={24} md={12} className="min-h-0">
          <div className="h-full min-h-0 flex flex-col">
            <div className="flex-1 min-h-0">
              <PlaceholderPanel title="Platform Health" />
            </div>
          </div>
        </Col>

        {/* Top-right: Submission Statistics (renamed) */}
        <Col xs={24} md={12} className="min-h-0">
          <div className="h-full min-h-0 flex flex-col">
            <div className="flex-1 min-h-0">
              <PlaceholderPanel title="Submission Statistics" />
            </div>
          </div>
        </Col>

        {/* Bottom-left: split vertically → Audit Log (top) + Environments (bottom) */}
        <Col xs={24} md={12} className="min-h-0">
          <div className="h-full min-h-0 flex flex-col gap-3">
            <div className="flex-1 min-h-0">
              <PlaceholderPanel title="Audit Log" />
            </div>
            <div className="flex-1 min-h-0">
              <PlaceholderPanel title="Environments" />
            </div>
          </div>
        </Col>

        {/* Bottom-right: split into 2 columns → left stats, right quick actions + user management */}
        <Col xs={24} md={12} className="min-h-0">
          <div className="h-full min-h-0 grid grid-cols-1 md:grid-cols-2 gap-3">
            {/* Left column: vertical stats strip */}
            <div className="min-h-0">
              <div className="h-full grid grid-rows-3 gap-3">
                <StatsTile
                  title="Active users"
                  value={activeUsers}
                  icon={<TeamOutlined />}
                  suffix={activeUsers === 1 ? 'user' : 'users'}
                />
                <StatsTile
                  title="Running services"
                  value={runningServices}
                  icon={<CloudServerOutlined />}
                  suffix={runningServices === 1 ? 'service' : 'services'}
                />
                <StatsTile
                  title="Open incidents"
                  value={openIncidents}
                  icon={<SafetyCertificateOutlined />}
                  suffix={openIncidents === 1 ? 'incident' : 'incidents'}
                />
              </div>
            </div>

            {/* Right column: Quick Actions (top) + User Management (bottom) */}
            <div className="min-h-0 flex flex-col gap-3">
              <div className="flex-1 min-h-0">
                <PlaceholderPanel title="Quick Actions" />
              </div>
              <div className="flex-1 min-h-0">
                <PlaceholderPanel title="User Management" />
              </div>
            </div>
          </div>
        </Col>
      </Row>
    </div>
  );
};

export default AdminDashboard;
