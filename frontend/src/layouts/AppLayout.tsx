import { useEffect, useState } from 'react';
import {
  Avatar,
  Breadcrumb,
  Button,
  Dropdown,
  Drawer,
  Layout,
  Menu,
  Typography,
  type MenuProps,
} from 'antd';
import { useNavigate, useLocation, Outlet } from 'react-router-dom';
import {
  UserOutlined,
  LogoutOutlined,
  DoubleLeftOutlined,
  DoubleRightOutlined,
  MenuOutlined,
  MoonOutlined,
  SunOutlined,
} from '@ant-design/icons';

import Logo from '@/components/Logo';
import { useTopMenuItems, BOTTOM_MENU_ITEMS, type MenuItem } from '@/constants/sidebar';
import { useAuth } from '@/context/AuthContext';
import { useBreadcrumbs } from '@/hooks/useBreadcrumbs';
import { useMediaQuery } from 'react-responsive';
import { useTheme } from '@/context/ThemeContext';

const { Header, Sider, Content } = Layout;
const { Text } = Typography;

export default function AppLayout() {
  const [collapsed, setCollapsed] = useState(
    () => localStorage.getItem('sidebarCollapsed') === 'true',
  );
  const [mobileSidebarVisible, setMobileSidebarVisible] = useState(false);
  const isMobile = useMediaQuery({ maxWidth: 768 });
  const breadcrumbs = useBreadcrumbs();
  const navigate = useNavigate();
  const location = useLocation();
  const { logout, isAdmin, isUser, user, profilePictureUrl } = useAuth();
  const forceCollapsed = useMediaQuery({ maxWidth: 1024 });
  const { mode, setMode } = useTheme();
  const isDark = mode === 'dark';

  useEffect(() => {
    localStorage.setItem('sidebarCollapsed', collapsed.toString());
  }, [collapsed]);

  useEffect(() => {
    if (forceCollapsed) {
      setCollapsed(true);
    }
  }, [forceCollapsed]);

  const topMenuItems = useTopMenuItems();

  const filterMenuItems = (items: MenuItem[]) =>
    items
      .map((item) => {
        if (item.children) {
          const visibleChildren = item.children.filter(
            (child) => (!child.adminOnly || isAdmin) && (!child.userOnly || isUser),
          );
          return visibleChildren.length ? { ...item, children: visibleChildren } : null;
        }
        return (!item.adminOnly || isAdmin) && (!item.userOnly || isUser) ? item : null;
      })
      .filter((item): item is MenuItem => item !== null);

  const visibleMenuItems = filterMenuItems(topMenuItems);
  const visibleBottomItems = filterMenuItems(BOTTOM_MENU_ITEMS);

  const profileMenuItems: MenuProps['items'] = [
    {
      key: 'profile',
      icon: <UserOutlined />,
      label: 'Profile',
      onClick: () => navigate('/settings/account'),
    },
    {
      key: 'logout',
      icon: <LogoutOutlined />,
      label: 'Logout',
      onClick: () => {
        logout();
        navigate('/login');
      },
    },
  ];

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
            selectedKeys={[
              visibleMenuItems
                .map((item) => item.key)
                .filter(
                  (key) => location.pathname === key || location.pathname.startsWith(key + '/'),
                )
                .sort((a, b) => b.length - a.length)[0] ?? '',
            ]}
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
          selectedKeys={[
            visibleMenuItems
              .map((item) => item.key)
              .filter((key) => location.pathname === key || location.pathname.startsWith(key + '/'))
              .sort((a, b) => b.length - a.length)[0] ?? '',
          ]}
          onClick={({ key }) => {
            if (key === 'logout') {
              logout();
              navigate('/login');
            } else if (key === 'theme-toggle') {
              // Do nothing: Theme toggle handled inside its own onClick
              return;
            } else {
              navigate(key);
            }
            if (isMobile) setMobileSidebarVisible(false);
          }}
          inlineCollapsed={!isMobile && collapsed}
          className="!bg-transparent"
          style={{ border: 'none' }}
        >
          <Menu.Item
            key="theme-toggle"
            icon={isDark ? <MoonOutlined /> : <SunOutlined />}
            title={isDark ? 'Switch to Light Mode' : 'Switch to Dark Mode'}
            onClick={(e) => {
              e.domEvent.stopPropagation(); // Prevent sidebar collapse if inside collapsible menu
              setMode(isDark ? 'light' : 'dark');
            }}
          >
            {!collapsed && (isDark ? 'Dark Mode' : 'Light Mode')}
          </Menu.Item>

          {/* Then render all bottom items */}
          {visibleBottomItems.map((item) => {
            if (!item) return null;
            return (
              <Menu.Item key={item.key} icon={item.icon}>
                {item.label}
              </Menu.Item>
            );
          })}
        </Menu>

        {!isMobile && !forceCollapsed && (
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
          collapsible={!forceCollapsed}
          collapsed={collapsed}
          onCollapse={!forceCollapsed ? setCollapsed : undefined}
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
                <Dropdown
                  menu={{ items: profileMenuItems }}
                  trigger={['click']}
                  placement="bottomRight"
                >
                  <div className="cursor-pointer flex items-center gap-2">
                    <Avatar
                      size="large"
                      icon={<UserOutlined />}
                      src={profilePictureUrl || undefined}
                    />

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
              <Dropdown
                menu={{ items: profileMenuItems }}
                trigger={['click']}
                placement="bottomRight"
              >
                <div className="cursor-pointer flex items-center gap-2 flex-row-reverse">
                  <Avatar
                    size="large"
                    icon={<UserOutlined />}
                    src={profilePictureUrl || undefined}
                  />

                  <Text className="hidden sm:inline text-gray-700 dark:text-gray-200 font-medium">
                    {user?.student_number ?? 'User'}
                  </Text>
                </div>
              </Dropdown>
            )}
          </div>
        </Header>

        <Content className="flex-1 min-h-0 overflow-hidden bg-gray-50 dark:bg-gray-950">
          <div className="h-full shadow-sm flex flex-col min-h-0">
            <Outlet />
          </div>
        </Content>
      </Layout>
    </Layout>
  );
}
