import { useState } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import {
  BulbFilled,
  BulbOutlined,
  CloseOutlined,
  LoginOutlined,
  MenuOutlined,
  TeamOutlined,
  UserAddOutlined,
} from '@ant-design/icons';
import { Button, Drawer, Space, Typography } from 'antd';

import { useTheme } from '@/context/ThemeContext';

const { Text } = Typography;

const MarketingHeader = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const { isDarkMode, setMode } = useTheme();

  const [isDrawerOpen, setIsDrawerOpen] = useState(false);

  const toggleTheme = () => setMode(isDarkMode ? 'light' : 'dark');

  const closeDrawer = () => setIsDrawerOpen(false);

  const handleNavigate = (path: string) => {
    if (location.pathname !== path) navigate(path);
    closeDrawer();
  };

  const isCurrentPath = (path: string) => location.pathname === path;

  const drawerButtonClass =
    'dark:!bg-gray-900 dark:!border-gray-700 dark:!text-gray-100 dark:hover:!bg-gray-800';

  return (
    <header className="sticky top-0 z-50 bg-white/80 backdrop-blur dark:bg-gray-950/80 w-full border-b border-gray-200 dark:border-gray-800">
      <div className="max-w-7xl mx-auto flex items-center justify-between py-5 px-4 sm:px-6 min-w-0">
        <button
          type="button"
          onClick={() => navigate('/')}
          className="flex items-center gap-2 sm:gap-3 min-w-0 text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-blue-500/60 rounded-md"
        >
          <img
            src={isDarkMode ? '/ff_logo_dark.svg' : '/ff_logo_light.svg'}
            alt="FitchFork logo"
            className="h-8 w-8 shrink-0"
          />
          <Text className="!m-0 text-lg sm:text-xl font-semibold text-gray-900 dark:text-white whitespace-nowrap truncate max-w-[48vw] sm:max-w-none">
            FitchFork
          </Text>
        </button>

        <div className="flex items-center gap-3">
          <Button type="primary" onClick={() => handleNavigate('/signup')}>
            Sign Up
          </Button>
          <Button
            type="text"
            icon={<MenuOutlined />}
            aria-label="Open navigation"
            onClick={() => setIsDrawerOpen(true)}
            className="text-gray-700 dark:text-gray-200"
          />
        </div>
      </div>

      <Drawer
        placement="right"
        open={isDrawerOpen}
        onClose={closeDrawer}
        width={312}
        closable={false}
        className="marketing-header-drawer !p-0"
        styles={{
          body: {
            padding: 0,
            background: 'transparent',
          },
        }}
      >
        <div className="bg-white dark:bg-gray-900 h-full flex flex-col">
          <div className="flex items-center justify-between px-6 py-5 border-b border-gray-200 dark:border-gray-800">
            <Text className="text-lg font-semibold text-gray-900 dark:text-gray-100">Menu</Text>
            <Button
              type="text"
              icon={<CloseOutlined />}
              onClick={closeDrawer}
              className="text-gray-500 hover:!text-gray-700 dark:text-gray-300 dark:hover:!text-white"
            />
          </div>

          <div className="px-6 py-5 border-b border-gray-200 dark:border-gray-800">
            <Space className="w-full justify-between" align="center">
              <Text className="text-base font-medium text-gray-800 dark:text-gray-200">
                Appearance
              </Text>
              <Button
                icon={isDarkMode ? <BulbFilled /> : <BulbOutlined />}
                onClick={toggleTheme}
                type="default"
                className={drawerButtonClass}
              >
                {isDarkMode ? 'Light mode' : 'Dark mode'}
              </Button>
            </Space>
          </div>

          <div className="flex-1 px-6 py-6 flex flex-col gap-3">
            <Button
              block
              icon={<TeamOutlined />}
              type={isCurrentPath('/team') ? 'primary' : 'default'}
              className={isCurrentPath('/team') ? undefined : drawerButtonClass}
              onClick={() => handleNavigate('/team')}
            >
              Team
            </Button>
            <Button
              block
              icon={<LoginOutlined />}
              type={isCurrentPath('/login') ? 'primary' : 'default'}
              className={isCurrentPath('/login') ? undefined : drawerButtonClass}
              onClick={() => handleNavigate('/login')}
            >
              Login
            </Button>
            <Button
              block
              icon={<UserAddOutlined />}
              type="primary"
              className="dark:!bg-blue-500 dark:hover:!bg-blue-400"
              onClick={() => handleNavigate('/signup')}
            >
              Sign Up
            </Button>
          </div>
        </div>
      </Drawer>
    </header>
  );
};

export default MarketingHeader;
