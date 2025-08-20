// components/layout/NotificationDropdown.tsx

import { List } from 'antd';

type NotificationItem = {
  id: number;
  title: string;
  time: string;
};

type NotificationDropdownProps = {
  notifications: NotificationItem[];
};

const NotificationDropdown = ({ notifications }: NotificationDropdownProps) => {
  return (
    <div className="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-lg shadow-xl w-72 max-h-80 overflow-y-auto">
      <List
        dataSource={notifications}
        renderItem={(item) => (
          <List.Item className="!px-3 !py-2 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors duration-150">
            <List.Item.Meta
              title={
                <span className="text-sm font-medium text-gray-900 dark:text-gray-100">
                  {item.title}
                </span>
              }
              description={<span className="text-xs text-gray-500">{item.time}</span>}
            />
          </List.Item>
        )}
      />
    </div>
  );
};

export default NotificationDropdown;
