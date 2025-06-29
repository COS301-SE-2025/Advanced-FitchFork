// components/layout/SidebarContent.tsx

import { Button, Menu } from 'antd';
import {
  DoubleLeftOutlined,
  DoubleRightOutlined,
  MoonOutlined,
  SunOutlined,
} from '@ant-design/icons';
import Logo from '@/components/Logo';
import type { MenuItem } from '@/constants/sidebar';

type SidebarContentProps = {
  collapsed: boolean;
  isMobile: boolean;
  forceCollapsed: boolean;
  navigate: (path: string) => void;
  location: { pathname: string };
  visibleMenuItems: MenuItem[];
  visibleBottomItems: MenuItem[];
  isDark: boolean;
  setCollapsed: (val: boolean) => void;
  setMode: (val: 'light' | 'dark') => void;
  logout: () => void;
};

const SidebarContent = ({
  collapsed,
  isMobile,
  forceCollapsed,
  navigate,
  location,
  visibleMenuItems,
  visibleBottomItems,
  isDark,
  setCollapsed,
  setMode,
  logout,
}: SidebarContentProps) => {
  const selectedKeys = [
    visibleMenuItems
      .map((item) => item.key)
      .filter((key) => location.pathname === key || location.pathname.startsWith(key + '/'))
      .sort((a, b) => b.length - a.length)[0] ?? '',
  ];

  return (
    <div className="bg-white dark:bg-gray-950 h-full flex flex-col justify-between">
      <div>
        <div
          className="py-4 mb-4 flex items-center justify-center cursor-pointer"
          onClick={() => isMobile && setCollapsed(false)}
        >
          <Logo collapsed={collapsed && !isMobile} />
        </div>

        <div className="px-2">
          <Menu
            mode="inline"
            theme="light"
            selectedKeys={selectedKeys}
            items={visibleMenuItems}
            onClick={({ key }) => {
              if (key !== 'logout') {
                navigate(key);
              }
              if (isMobile) setCollapsed(false);
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
          selectedKeys={selectedKeys}
          onClick={({ key, domEvent }) => {
            if (key === 'logout') {
              logout();
              navigate('/login');
            } else if (key === 'theme-toggle') {
              domEvent.stopPropagation();
              setMode(isDark ? 'light' : 'dark');
            } else {
              navigate(key);
            }
            if (isMobile) setCollapsed(false);
          }}
          inlineCollapsed={!isMobile && collapsed}
          className="!bg-transparent"
          style={{ border: 'none' }}
        >
          <Menu.Item
            key="theme-toggle"
            icon={isDark ? <MoonOutlined /> : <SunOutlined />}
            title={isDark ? 'Switch to Light Mode' : 'Switch to Dark Mode'}
          >
            {!collapsed && (isDark ? 'Dark Mode' : 'Light Mode')}
          </Menu.Item>

          {visibleBottomItems.map((item) => (
            <Menu.Item key={item.key} icon={item.icon}>
              {item.label}
            </Menu.Item>
          ))}
        </Menu>

        {!isMobile && !forceCollapsed && (
          <div className="px-1 mt-3">
            <Button
              block
              type="default"
              onClick={() => setCollapsed(!collapsed)}
              icon={collapsed ? <DoubleRightOutlined /> : <DoubleLeftOutlined />}
            >
              {collapsed ? '' : 'Collapse'}
            </Button>
          </div>
        )}
      </div>
    </div>
  );
};

export default SidebarContent;
