import { Layout, Menu } from 'antd';
import { UserOutlined, LockOutlined, SettingOutlined, GlobalOutlined } from '@ant-design/icons';
import { Outlet, useNavigate, useLocation } from 'react-router-dom';

const { Sider, Content } = Layout;

const SETTINGS_GROUPS = [
  {
    title: 'Account',
    items: [
      { key: 'account', label: 'Account', icon: <UserOutlined /> },
      { key: 'security', label: 'Security', icon: <LockOutlined /> },
    ],
  },
  {
    title: 'Interface',
    items: [
      { key: 'appearance', label: 'Appearance', icon: <SettingOutlined /> },
      { key: 'language', label: 'Language', icon: <GlobalOutlined /> },
    ],
  },
];

const SettingsLayout = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const pathSegments = location.pathname.split('/');
  const selectedKey = pathSegments[pathSegments.length - 1] ?? '';

  return (
    <Layout className="min-h-screen !bg-transparent dark:!bg-gray-950 overflow-hidden">
      <Sider
        width={240}
        className="!bg-white dark:!bg-gray-950 border-r border-gray-200 dark:border-gray-800"
      >
        <div className="py-4 px-6 font-semibold text-gray-700 dark:text-gray-200 text-lg">
          Settings
        </div>
        {SETTINGS_GROUPS.map(({ title, items }) => (
          <div key={title} className="mb-4 px-2">
            <div className="text-gray-500 dark:text-gray-400 px-2 mb-2 text-sm font-small">
              {title}
            </div>
            <Menu
              mode="inline"
              theme="light"
              selectedKeys={[selectedKey]}
              onClick={({ key }) => navigate(`/settings/${key}`)}
              className="!bg-transparent !p-0"
              style={{ border: 'none' }}
              items={items}
            />
          </div>
        ))}
      </Sider>

      <Layout className="!bg-transparent flex-1 min-h-screen">
        <Content className="w-full h-full">
          <Outlet />
        </Content>
      </Layout>
    </Layout>
  );
};

export default SettingsLayout;
