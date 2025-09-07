import { useEffect, useState } from 'react';
import { Drawer, Layout, type MenuProps } from 'antd';
import { UserOutlined, LogoutOutlined } from '@ant-design/icons';
import { useNavigate, useLocation, Outlet } from 'react-router-dom';
import { useMediaQuery } from 'react-responsive';

import { useTopMenuItems, BOTTOM_MENU_ITEMS, type MenuItem } from '@/constants/sidebar';
import { useAuth } from '@/context/AuthContext';
import { useTheme } from '@/context/ThemeContext';

import SidebarContent from '@/components/layout/SidebarContent';
import HeaderBar from '@/components/layout/HeaderBar';

const { Header, Sider, Content } = Layout;

const notifications = [
  { id: 1, title: 'New submission in COS344', time: '2m ago' },
  { id: 2, title: 'Assignment marked in COS332', time: '5m ago' },
  { id: 3, title: 'System updated successfully', time: '10m ago' },
];

const AppLayout = () => {
  const [collapsed, setCollapsed] = useState(
    () => localStorage.getItem('sidebarCollapsed') === 'true',
  );
  const [mobileSidebarVisible, setMobileSidebarVisible] = useState(false);
  const isMobile = useMediaQuery({ maxWidth: 768 });
  const forceCollapsed = useMediaQuery({ maxWidth: 1024 });

  const navigate = useNavigate();
  const location = useLocation();
  const { logout, isAdmin, isUser } = useAuth();
  const { mode, setMode } = useTheme();
  const isDark = mode === 'dark';

  useEffect(() => {
    localStorage.setItem('sidebarCollapsed', collapsed.toString());
  }, [collapsed]);

  useEffect(() => {
    if (forceCollapsed) setCollapsed(true);
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

  const sidebarProps = {
    collapsed,
    setCollapsed,
    isMobile,
    location,
    navigate,
    visibleMenuItems,
    visibleBottomItems,
    forceCollapsed,
    setMode,
    isDark,
    setMobileSidebarVisible,
    logout,
  };

  return (
    <Layout className="!h-dvh overflow-hidden !bg-gray-50 dark:!bg-gray-950">
      {isMobile ? (
        <Drawer
          placement="right"
          open={mobileSidebarVisible}
          onClose={() => setMobileSidebarVisible(false)}
          width={240}
          closable={false}
          className="!p-0"
          styles={{ body: { padding: 0 }, header: { display: 'none' } }}
        >
          <SidebarContent
            {...sidebarProps}
            collapsed={false}
            onMobileNavigate={() => setMobileSidebarVisible(false)}
          />
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
          <SidebarContent {...sidebarProps} />
        </Sider>
      )}

      <Layout className="!bg-transparent flex flex-col flex-1 min-h-0">
        <Header className="border-b !bg-white dark:!bg-gray-900 dark:!bg-gray border-gray-200 dark:border-gray-800 !px-4 sm:px-6">
          <HeaderBar
            notifications={notifications}
            profileMenuItems={profileMenuItems}
            onMenuClick={() => setMobileSidebarVisible(true)}
          />
        </Header>

        <Content className="flex-1 min-h-0 overflow-hidden !bg-transparent">
          <Outlet />
        </Content>
      </Layout>
    </Layout>
  );
};

export default AppLayout;
