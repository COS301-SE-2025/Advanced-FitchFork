import StatCard from '@/components/StatCard';
import { Row, Col } from 'antd';
import SubmissionsPanel from '@/components/dashboard/SubmissionsPanel';
import UserManagementPanel from '@/components/dashboard/UserManagementPanel';
import QuickActionsPanel from '@/components/dashboard/QuickActionsPanel';
import SystemOverviewPanel from '@/components/dashboard/SystemOverviewPanel';
import ModuleAssignmentsPanel from '@/components/dashboard/ModuleAssignmentsPanel';
import {
  UserOutlined,
  AppstoreOutlined,
  FileTextOutlined,
  UploadOutlined,
} from '@ant-design/icons';

const Dashboard = () => {
  return (
    <div className="p-4">
      {/* Stat Cards */}
      <Row gutter={[16, 16]}>
        <Col xs={24} sm={12} md={12} lg={6}>
          <StatCard title="Total Users" value={1432} prefix={<UserOutlined />} />
        </Col>
        <Col xs={24} sm={12} md={12} lg={6}>
          <StatCard title="Total Modules" value={40} prefix={<AppstoreOutlined />} />
        </Col>
        <Col xs={24} sm={12} md={12} lg={6}>
          <StatCard title="Active Assignments" value={10} prefix={<FileTextOutlined />} />
        </Col>
        <Col xs={24} sm={12} md={12} lg={6}>
          <StatCard title="Submissions Today" value={100} prefix={<UploadOutlined />} />
        </Col>
      </Row>

      {/* Submissions + User Management + Quick Actions */}
      <Row gutter={[16, 16]} className="mt-4">
        <Col xs={24} md={12} lg={8}>
          <SubmissionsPanel />
        </Col>
        <Col xs={24} md={12} lg={8}>
          <UserManagementPanel />
        </Col>
        <Col xs={24} md={24} lg={8}>
          <QuickActionsPanel />
        </Col>
      </Row>

      {/* Module & System Insights */}
      <Row gutter={[16, 16]} className="mt-4">
        <Col xs={24} md={12}>
          <ModuleAssignmentsPanel />
        </Col>
        <Col xs={24} md={12}>
          <SystemOverviewPanel />
        </Col>
      </Row>
    </div>
  );
};

export default Dashboard;
