import { useEffect, useState } from 'react';
import { Avatar, Breadcrumb, Button, Dropdown, Layout, Menu, Switch, Typography } from 'antd';
import { useNavigate, useLocation } from 'react-router-dom';
import {
  UserOutlined,
  LogoutOutlined,
  DoubleLeftOutlined,
  DoubleRightOutlined,
  BulbFilled,
  BulbOutlined,
} from '@ant-design/icons';
import Logo from '@/components/Logo';
import { TOP_MENU_ITEMS, BOTTOM_MENU_ITEMS } from '@/constants/sidebar';
import { useAuth } from '@/context/AuthContext';
import { useTheme } from '@/context/ThemeContext';

const { Header, Sider, Content } = Layout;
const { Title, Paragraph, Text } = Typography;

interface AppLayoutProps {
  title: string;
  description?: string;
  children: React.ReactNode;
}

export default function AppLayout({ title, description, children }: AppLayoutProps) {
  const [collapsed, setCollapsed] = useState(() => {
    return localStorage.getItem('sidebarCollapsed') === 'true';
  });

  const navigate = useNavigate();
  const location = useLocation();
  const { logout, isAdmin, user } = useAuth();
  const { isDarkMode, toggleDarkMode } = useTheme();

  useEffect(() => {
    localStorage.setItem('sidebarCollapsed', collapsed.toString());
  }, [collapsed]);

  const isUserAdmin = isAdmin();
  const isUser = !isUserAdmin;

  const filterMenuItems = (items: typeof TOP_MENU_ITEMS) =>
    items
      .map((item) => {
        if (item.children) {
          const visibleChildren = item.children.filter(
            (child) => (!child.adminOnly || isUserAdmin) && (!child.userOnly || isUser),
          );
          if (visibleChildren.length > 0) {
            return { ...item, children: visibleChildren };
          }
          return null;
        }

        const canShow = (!item.adminOnly || isUserAdmin) && (!item.userOnly || isUser);
        return canShow ? item : null;
      })
      .filter(Boolean);

  const visibleMenuItems = filterMenuItems(TOP_MENU_ITEMS);
  const visibleBottomItems = filterMenuItems(BOTTOM_MENU_ITEMS);

  const profileMenu = (
    <Menu>
      <Menu.Item key="profile" icon={<UserOutlined />} onClick={() => navigate('/profile')}>
        Profile
      </Menu.Item>
      <Menu.Item key="theme-toggle">
        <div
          className="flex items-center justify-between gap-4 w-full"
          onClick={(e) => e.stopPropagation()}
        >
          <div className="flex items-center gap-2">
            {isDarkMode ? <BulbFilled className="text-yellow-400" /> : <BulbOutlined />}
            <span>{isDarkMode ? 'Dark Mode' : 'Light Mode'}</span>
          </div>
          <Switch checked={isDarkMode} onChange={toggleDarkMode} />
        </div>
      </Menu.Item>

      <Menu.Item
        key="logout"
        icon={<LogoutOutlined />}
        onClick={() => {
          logout();
          navigate('/login');
        }}
      >
        Logout
      </Menu.Item>
    </Menu>
  );

  return (
    <Layout className="h-screen overflow-hidden bg-white dark:bg-gray-950">
      <Sider
        width={240}
        collapsedWidth={80}
        collapsible
        collapsed={collapsed}
        onCollapse={setCollapsed}
        trigger={null}
        className="!bg-transparent !p-0 !m-0"
      >
        <div className="bg-white dark:bg-gray-950 h-full flex flex-col justify-between">
          <div>
            <div className="px-4 py-4 mb-4 flex items-center justify-center overflow-hidden">
              <Logo collapsed={collapsed} />
            </div>
            <Menu
              mode="inline"
              theme="light"
              selectedKeys={[location.pathname]}
              items={visibleMenuItems}
              onClick={({ key }) => {
                if (key === 'logout') return;
                navigate(key);
              }}
              inlineCollapsed={collapsed}
              className="!bg-transparent !p-0 mt-2"
              style={{ border: 'none' }}
            />
          </div>
          <div className="px-2 pb-4">
            <Menu
              mode="inline"
              theme="light"
              selectedKeys={[]}
              items={visibleBottomItems}
              onClick={({ key }) => {
                if (key === 'logout') {
                  logout();
                  navigate('/login');
                }
              }}
              className="!bg-transparent"
              style={{ border: 'none' }}
              inlineCollapsed={collapsed}
            />
            <div className="px-4 mt-3">
              <Button
                block
                type="default"
                onClick={() => setCollapsed((prev) => !prev)}
                icon={collapsed ? <DoubleRightOutlined /> : <DoubleLeftOutlined />}
                className="w-full"
              >
                {collapsed ? '' : 'Collapse'}
              </Button>
            </div>
          </div>
        </div>
      </Sider>

      <Layout className="flex flex-col w-full h-screen overflow-hidden !bg-white dark:!bg-gray-950">
        <Header
          className="!bg-transparent !px-0"
          style={{
            backgroundColor: 'transparent',
          }}
        >
          <div className="bg-white dark:bg-gray-950 px-6 flex items-center justify-between w-full h-full">
            <Breadcrumb separator=">">
              {location.pathname
                .split('/')
                .filter(Boolean)
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

            <Dropdown overlay={profileMenu} trigger={['click']} placement="bottomRight">
              <div className="cursor-pointer flex items-center gap-2">
                <Text className="hidden sm:inline text-gray-700 dark:text-gray-200 font-medium">
                  {user?.student_number ?? 'User'}
                </Text>
                <Avatar size="large" src="profile.jpeg" alt="User Avatar" />
              </div>
            </Dropdown>
          </div>
        </Header>

        <Content className="flex-1 min-h-0 overflow-hidden px-6 py-6 bg-gray-100 dark:bg-black rounded-tl-2xl">
          <div className="bg-white dark:bg-gray-900 h-full rounded-xl shadow-sm p-6 flex flex-col min-h-0">
            <div className="mb-4">
              <Title className="!text-gray-800 dark:!text-gray-100">{title}</Title>
              {description && (
                <Paragraph className="!text-gray-600 dark:!text-gray-300 mt-1">
                  {description}
                </Paragraph>
              )}
            </div>

            <div className="flex-1 min-h-0 overflow-y-auto">{children}</div>
          </div>
        </Content>
      </Layout>
    </Layout>
  );
}
