import { Layout, Menu } from 'antd';
import { UserOutlined, LockOutlined, SettingOutlined } from '@ant-design/icons';
import { Outlet, useNavigate, useLocation } from 'react-router-dom';
import { MobilePageHeader } from '@/components/common';
import { useUI } from '@/context/UIContext';

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
    items: [{ key: 'appearance', label: 'Appearance', icon: <SettingOutlined /> }],
  },
];

const SettingsLayout = () => {
  const { isMobile } = useUI();
  const navigate = useNavigate();
  const location = useLocation();
  const pathSegments = location.pathname.split('/');
  const selectedKey = pathSegments[pathSegments.length - 1] ?? '';

  return (
    <Layout className="flex h-full !bg-transparent">
      <Sider
        width={240}
        className="!bg-white dark:!bg-gray-900 border-r border-gray-200 dark:border-gray-800 sticky top-0 h-screen hidden sm:block"
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

      <Layout className="flex-1 flex flex-col min-h-0 !bg-transparent">
        {isMobile && <MobilePageHeader />}
        <Content className="flex-1 min-h-0 flex flex-col !bg-transparent">
          <Outlet />
        </Content>
      </Layout>
    </Layout>
  );
};

export default SettingsLayout;
