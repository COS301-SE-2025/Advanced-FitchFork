import { Layout, Menu } from 'antd';
import {
  InfoCircleOutlined,
  FileTextOutlined,
  UploadOutlined,
  WarningOutlined,
  MailOutlined,
} from '@ant-design/icons';
import { Outlet, useNavigate, useLocation } from 'react-router-dom';

const { Sider, Content } = Layout;

const HELP_ITEMS = [
  { key: 'account', label: 'Account & Login', icon: <InfoCircleOutlined /> },
  { key: 'assignments', label: 'Assignments', icon: <FileTextOutlined /> },
  { key: 'submissions', label: 'Submissions', icon: <UploadOutlined /> },
  { key: 'troubleshooting', label: 'Troubleshooting', icon: <WarningOutlined /> },
  { key: 'contact', label: 'Contact', icon: <MailOutlined /> },
];

const HelpPageLayout = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const pathSegments = location.pathname.split('/');
  const selectedKey = pathSegments[pathSegments.length - 1] ?? '';

  return (
    <Layout className="min-h-screen !bg-white dark:!bg-gray-950">
      <Sider
        width={240}
        className="!bg-white dark:!bg-gray-950 border-r border-gray-200 dark:border-gray-800 sticky top-0 h-screen"
      >
        <div className="py-4 px-6 font-semibold text-gray-700 dark:text-gray-200 text-lg">
          Help Center
        </div>
        <Menu
          mode="inline"
          selectedKeys={[selectedKey]}
          onClick={({ key }) => navigate(`/help/${key}`)}
          className="!bg-transparent !p-0 custom-dark-menu"
          style={{ border: 'none' }}
          items={HELP_ITEMS}
        />
      </Sider>

      <Layout className="!bg-white dark:!bg-gray-950 flex-1 max-h-screen overflow-y-auto max-w-5xl">
        <Content className="p-4 sm:p-6 text-gray-800 dark:text-gray-100">
          <Outlet />
        </Content>
      </Layout>
    </Layout>
  );
};

export default HelpPageLayout;
