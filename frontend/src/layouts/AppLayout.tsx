import { useEffect, useState } from 'react';
import {
  Avatar,
  Breadcrumb,
  Button,
  Dropdown,
  Drawer,
  Layout,
  Menu,
  Switch,
  Typography,
} from 'antd';
import { useNavigate, useLocation } from 'react-router-dom';
import {
  UserOutlined,
  LogoutOutlined,
  DoubleLeftOutlined,
  DoubleRightOutlined,
  BulbFilled,
  BulbOutlined,
  MenuOutlined,
} from '@ant-design/icons';

import Logo from '@/components/Logo';
import { TOP_MENU_ITEMS, BOTTOM_MENU_ITEMS } from '@/constants/sidebar';
import { useAuth } from '@/context/AuthContext';
import { useTheme } from '@/context/ThemeContext';
import { useBreadcrumbs } from '@/hooks/useBreadcrumbs';
import { useMediaQuery } from 'react-responsive';

const { Header, Sider, Content } = Layout;
const { Title, Paragraph, Text } = Typography;

interface AppLayoutProps {
  title: React.ReactNode;
  description?: string;
  children: React.ReactNode;
}

export default function AppLayout({ title, description, children }: AppLayoutProps) {
  const [collapsed, setCollapsed] = useState(
    () => localStorage.getItem('sidebarCollapsed') === 'true',
  );
  const [mobileSidebarVisible, setMobileSidebarVisible] = useState(false);
  const isMobile = useMediaQuery({ maxWidth: 768 });

  const breadcrumbs = useBreadcrumbs();
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
          return visibleChildren.length ? { ...item, children: visibleChildren } : null;
        }
        return (!item.adminOnly || isUserAdmin) && (!item.userOnly || isUser) ? item : null;
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

  const renderSidebarContent = () => (
    <div className="bg-white dark:bg-gray-950 h-full flex flex-col justify-between">
      <div>
        <div
          className="py-4 mb-4 flex items-center justify-center cursor-pointer"
          onClick={() => isMobile && setMobileSidebarVisible(false)}
        >
          <Logo collapsed={collapsed && !isMobile} />
        </div>

        <div className="px-2">
          <Menu
            mode="inline"
            theme="light"
            selectedKeys={[location.pathname]}
            items={visibleMenuItems}
            onClick={({ key }) => {
              if (key === 'logout') return;
              navigate(key);
              if (isMobile) setMobileSidebarVisible(false);
            }}
            inlineCollapsed={!isMobile && collapsed}
            className="!bg-transparent !p-0 mt-2"
            style={{ border: 'none' }}
          />
        </div>
      </div>
      <div className="px-2 pb-4">
        <Menu
          mode="inline"
          theme="light"
          selectedKeys={[location.pathname]}
          items={visibleBottomItems}
          onClick={({ key }) => {
            if (key === 'logout') {
              logout();
              navigate('/login');
            } else {
              navigate(key);
            }
            if (isMobile) setMobileSidebarVisible(false);
          }}
          className="!bg-transparent"
          style={{ border: 'none' }}
        />
        {!isMobile && (
          <div className="px-1 mt-3">
            <Button
              block
              type="default"
              onClick={() => setCollapsed((prev) => !prev)}
              icon={collapsed ? <DoubleRightOutlined /> : <DoubleLeftOutlined />}
            >
              {collapsed ? '' : 'Collapse'}
            </Button>
          </div>
        )}
      </div>
    </div>
  );

  return (
    <Layout className="min-h-screen bg-gray-100 dark:bg-gray-950">
      {isMobile ? (
        <Drawer
          placement="right"
          open={mobileSidebarVisible}
          onClose={() => setMobileSidebarVisible(false)}
          width={240}
          closable={false}
          className="!p-0"
          styles={{
            body: { padding: 0, margin: 0 },
            header: { display: 'none' },
          }}
        >
          {renderSidebarContent()}
        </Drawer>
      ) : (
        <Sider
          width={240}
          collapsedWidth={80}
          collapsible
          collapsed={collapsed}
          onCollapse={setCollapsed}
          trigger={null}
          className="!bg-transparent border-r border-gray-200 dark:border-gray-800"
        >
          {renderSidebarContent()}
        </Sider>
      )}

      <Layout className="flex flex-col w-full h-screen !bg-white dark:!bg-gray-950">
        <Header className="!bg-transparent border-b border-gray-200 dark:border-gray-800 !px-4 sm:px-6">
          <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 sm:gap-0 w-full h-full">
            {/* Mobile: profile + hamburger in row */}
            {isMobile && (
              <div className="flex items-center justify-between w-full h-full">
                <Dropdown overlay={profileMenu} trigger={['click']} placement="bottomLeft">
                  <div className="cursor-pointer flex items-center gap-2">
                    <Avatar size="large" src="/profile.jpeg" alt="User Avatar" />
                    <Text className="text-gray-700 dark:text-gray-200 font-medium">
                      {user?.student_number ?? 'User'}
                    </Text>
                  </div>
                </Dropdown>

                <Button
                  type="text"
                  icon={<MenuOutlined />}
                  onClick={() => setMobileSidebarVisible(true)}
                  className="text-gray-700 dark:text-gray-200"
                />
              </div>
            )}

            {/* Breadcrumbs (desktop only) */}
            <Breadcrumb
              separator=">"
              className="hidden sm:flex flex-1"
              items={breadcrumbs.map(({ path, label, isLast }) => ({
                title: isLast ? (
                  label
                ) : (
                  <a onClick={() => navigate(path)} className="text-blue-600 hover:underline">
                    {label}
                  </a>
                ),
              }))}
            />

            {/* Right (desktop only): profile */}
            {!isMobile && (
              <Dropdown overlay={profileMenu} trigger={['click']} placement="bottomRight">
                <div className="cursor-pointer flex items-center gap-2">
                  <Avatar size="large" src="/profile.jpeg" alt="User Avatar" />
                  <Text className="hidden sm:inline text-gray-700 dark:text-gray-200 font-medium">
                    {user?.student_number ?? 'User'}
                  </Text>
                </div>
              </Dropdown>
            )}
          </div>
        </Header>

        <Content className="flex-1 min-h-0 overflow-hidden bg-white dark:bg-gray-950">
          <div className="h-full shadow-sm p-4 sm:p-6 flex flex-col min-h-0">
            <div className="mb-4">
              <Title className="!text-lg sm:!text-2xl !text-gray-800 dark:!text-gray-100 !leading-tight !mb-1">
                {title}
              </Title>
              {description && (
                <Paragraph className="!text-sm sm:!text-base !text-gray-600 dark:!text-gray-300">
                  {description}
                </Paragraph>
              )}
            </div>

            {/* Scrollable body that allows full-width children like tables */}
            <div className="flex-1 min-h-0 overflow-auto w-full">
              <div className="min-w-full">{children}</div>
            </div>
          </div>
        </Content>
      </Layout>
    </Layout>
  );
}
