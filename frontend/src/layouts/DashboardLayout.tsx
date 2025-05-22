import { useState } from 'react';
import { Breadcrumb, Layout, Menu } from 'antd';
import { HomeOutlined, SettingOutlined, LogoutOutlined, UserOutlined } from '@ant-design/icons';
import { useNavigate, useLocation } from 'react-router-dom';
import Logo from '@components/Logo';
import { useAuth } from '@context/AuthContext';

const { Header, Sider, Content } = Layout;

const topMenuItems = [
  {
    key: '/dashboard',
    icon: <HomeOutlined />,
    label: 'Home',
  },
  {
    key: '/users',
    icon: <UserOutlined />,
    label: 'Users',
  },
  {
    key: '/settings',
    icon: <SettingOutlined />,
    label: 'Settings',
  },
];

const logoutMenuItem = [
  {
    key: 'logout',
    icon: <LogoutOutlined />,
    label: 'Logout',
  },
];

interface DashboardLayoutProps {
  title: string;
  description?: string;
  children: React.ReactNode;
}

export default function DashboardLayout({ title, description, children }: DashboardLayoutProps) {
  const [collapsed, setCollapsed] = useState(false);
  const navigate = useNavigate();
  const location = useLocation();
  const { logout } = useAuth();

  return (
    <Layout className="h-screen overflow-hidden !bg-white">
      <Sider
        width={240}
        collapsedWidth={80}
        collapsible
        collapsed={collapsed}
        onCollapse={setCollapsed}
        className="!bg-white"
        theme="light"
      >
        <div className="h-full flex flex-col justify-between">
          {/* Top: Logo + Menu */}
          <div>
            <div className="px-4 py-4 mb-4 flex items-center justify-center">
              <Logo size="md" showText={!collapsed} />
            </div>

            <Menu
              mode="inline"
              theme="light"
              selectedKeys={[location.pathname]}
              items={topMenuItems}
              onClick={({ key }) => {
                if (key === 'logout') return;
                navigate(key);
              }}
              inlineCollapsed={collapsed}
              className="mt-2"
              style={{ border: 'none' }}
            />
          </div>

          {/* Bottom: Logout item */}
          <div className="border-t border-gray-200">
            <Menu
              mode="inline"
              theme="light"
              selectedKeys={[]}
              items={logoutMenuItem}
              onClick={({ key }) => {
                if (key === 'logout') {
                  logout();
                  navigate('/login');
                }
              }}
              inlineCollapsed={collapsed}
            />
          </div>
        </div>
      </Sider>

      <Layout className="flex flex-col !bg-white">
        <Header className="!bg-white px-6 flex items-center justify-between h-16 shrink-0">
          <Breadcrumb separator=">">
            {location.pathname
              .split('/')
              .filter((part) => part)
              .map((part, index, arr) => {
                const path = '/' + arr.slice(0, index + 1).join('/');
                const isLast = index === arr.length - 1;
                const label = part.charAt(0).toUpperCase() + part.slice(1);
                return (
                  <Breadcrumb.Item key={path}>
                    {!isLast ? (
                      <a onClick={() => navigate(path)} className="text-blue-600 hover:underline">
                        {label}
                      </a>
                    ) : (
                      label
                    )}
                  </Breadcrumb.Item>
                );
              })}
          </Breadcrumb>
        </Header>

        <Content className="flex-1 bg-gray-100 !rounded-tl-2xl p-6 overflow-hidden">
          <div className="bg-white rounded-xl shadow-sm h-full p-6 flex flex-col">
            {/* Title + Description (not scrollable) */}
            <div className="mb-4">
              <h1 className="text-2xl font-semibold text-gray-800">{title}</h1>
              {description && <p className="text-gray-600 mt-1">{description}</p>}
            </div>

            {/* Scrollable children */}
            <div className="flex-1 overflow-y-auto">{children}</div>
          </div>
        </Content>
      </Layout>
    </Layout>
  );
}
