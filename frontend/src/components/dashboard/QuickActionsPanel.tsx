import { Button, Typography } from 'antd';
import {
  PlusOutlined,
  TeamOutlined,
  InboxOutlined,
  ReloadOutlined,
  SettingOutlined,
} from '@ant-design/icons';

const { Title } = Typography;

const QuickActionsPanel = () => {
  return (
    <div className="h-full bg-white dark:bg-gray-950 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
      <Title level={4} className="mb-4">
        Quick Actions
      </Title>

      <div className="!space-y-2">
        <Button type="primary" block icon={<PlusOutlined />} className="text-left">
          Create New Module
        </Button>
        <Button block icon={<TeamOutlined />} className="text-left">
          Assign Tutors to Module
        </Button>
        <Button block icon={<InboxOutlined />} className="text-left">
          Archive Old Assignments
        </Button>

        <Button block icon={<ReloadOutlined />} className="text-left">
          Refresh Container Images
        </Button>
        <Button block icon={<SettingOutlined />} className="text-left">
          System Settings
        </Button>
      </div>
    </div>
  );
};

export default QuickActionsPanel;
