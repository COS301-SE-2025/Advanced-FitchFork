import { Col, Dropdown, Row, Typography } from 'antd';
import {
  UploadOutlined,
  UserAddOutlined,
  FlagOutlined,
  HourglassOutlined,
  MoreOutlined,
} from '@ant-design/icons';
import useNotImplemented from '@/hooks/useNotImplemented';
import StatCard from '../StatCard';

const { Title } = Typography;

const UserManagementPanel = () => {
  const notImplemented = useNotImplemented();

  const items = [
    {
      key: 'upload',
      icon: <UploadOutlined />,
      label: 'Upload Bulk Users',
    },
    {
      key: 'view',
      label: 'View All',
    },
  ];

  const handleMenuClick = ({}: { key: string }) => {
    notImplemented(); // everything is not implemented for now
  };

  return (
    <div className="h-full bg-white dark:bg-gray-900 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
      <div className="flex items-center justify-between mb-4">
        <Title level={4}>Users</Title>
        <div>
          <Dropdown.Button
            type="primary"
            menu={{ items, onClick: handleMenuClick }}
            icon={<MoreOutlined />}
            onClick={notImplemented}
          >
            <UserAddOutlined /> Add User
          </Dropdown.Button>
        </div>
      </div>

      <Row gutter={[16, 16]}>
        <Col xs={24} sm={8}>
          <StatCard title="Total Users" value={128} prefix={<UserAddOutlined />} />
        </Col>
        <Col xs={24} sm={8}>
          <StatCard title="Flagged Users" value={5} prefix={<FlagOutlined />} />
        </Col>
        <Col xs={24} sm={8}>
          <StatCard title="Pending Approvals" value={2} prefix={<HourglassOutlined />} />
        </Col>
      </Row>
    </div>
  );
};

export default UserManagementPanel;
