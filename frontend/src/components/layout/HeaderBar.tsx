import { Badge, Button, Dropdown, Typography } from 'antd';
import { BellOutlined, MenuOutlined } from '@ant-design/icons';
import NotificationDropdown from './NotificationDropdown';
import { useAuth } from '@/context/AuthContext';
import { useMediaQuery } from 'react-responsive';
import UserAvatar from '../common/UserAvatar';
import BreadcrumbNav from '../common/BreadcrumbNav';

const { Text } = Typography;

type HeaderBarProps = {
  notifications: { id: number; title: string; time: string }[];
  profileMenuItems: any;
  onMenuClick: () => void;
};

const HeaderBar = ({ notifications, profileMenuItems, onMenuClick }: HeaderBarProps) => {
  const { user } = useAuth();
  const isMobile = useMediaQuery({ maxWidth: 768 });

  return (
    <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2 sm:gap-0 w-full h-full">
      {/* Mobile: profile + hamburger */}
      {isMobile && (
        <div className="flex items-center justify-between w-full h-full">
          <Dropdown menu={{ items: profileMenuItems }} trigger={['click']} placement="bottomRight">
            <div className="cursor-pointer flex items-center gap-2">
              <UserAvatar user={{ id: user?.id ?? -1, username: user?.username ?? 'User' }} />
              <Text className="text-gray-700 dark:text-gray-200 font-medium">
                {user?.username ?? 'User'}
              </Text>
            </div>
          </Dropdown>

          <Button
            type="text"
            icon={<MenuOutlined />}
            onClick={onMenuClick}
            className="text-gray-700 dark:text-gray-200"
          />
        </div>
      )}

      {/* Breadcrumbs (desktop only) */}
      {!isMobile && <BreadcrumbNav className="hidden sm:flex flex-1" />}

      {/* Desktop: notifications + profile */}
      {!isMobile && (
        <div className="flex items-center gap-4">
          <Dropdown
            trigger={['click']}
            placement="bottomRight"
            popupRender={() => <NotificationDropdown notifications={notifications} />}
          >
            <Badge count={notifications.length} size="small">
              <BellOutlined className="text-lg text-gray-700 dark:text-gray-200 cursor-pointer" />
            </Badge>
          </Dropdown>

          <Dropdown menu={{ items: profileMenuItems }} trigger={['click']} placement="bottomRight">
            <div className="cursor-pointer flex items-center gap-2 flex-row-reverse">
              <UserAvatar user={{ id: user?.id ?? -1, username: user?.username ?? 'User' }} />
              <Text className="hidden sm:inline text-gray-700 dark:text-gray-200 font-medium">
                {user?.username ?? 'User'}
              </Text>
            </div>
          </Dropdown>
        </div>
      )}
    </div>
  );
};

export default HeaderBar;
