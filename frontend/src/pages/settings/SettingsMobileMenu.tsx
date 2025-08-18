import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Button, Space, Typography } from 'antd';
import {
  ThunderboltOutlined,
  CheckSquareOutlined,
  SkinOutlined,
  RightOutlined,
} from '@ant-design/icons';

import { useViewSlot } from '@/context/ViewSlotContext';

const SettingsMobileMenu = () => {
  const navigate = useNavigate();
  const { setValue } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Settings
      </Typography.Text>,
    );
  }, [setValue]);

  const groupedNavItems = [
    {
      title: 'Account',
      items: [
        {
          label: 'Account',
          path: 'account',
          icon: <ThunderboltOutlined className="text-lg" />,
        },
        {
          label: 'Security',
          path: 'security',
          icon: <CheckSquareOutlined className="text-lg" />,
        },
      ],
    },
    {
      title: 'Interface',
      items: [
        {
          label: 'Appearance',
          path: 'appearance',
          icon: <SkinOutlined className="text-lg" />,
        },
      ],
    },
  ];

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto space-y-6 p-4">
        {groupedNavItems.map((group) => (
          <div key={group.title}>
            <Typography.Text className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 block">
              {group.title}
            </Typography.Text>

            <Space.Compact direction="vertical" className="w-full">
              {group.items.map((item) => (
                <Button
                  key={item.label}
                  type="default"
                  block
                  className="!h-14 px-4 flex items-center !justify-between text-base"
                  onClick={() => navigate(item.path)}
                >
                  <Typography.Text className="flex items-center gap-2 text-left">
                    {item.icon}
                    {item.label}
                  </Typography.Text>
                  <RightOutlined />
                </Button>
              ))}
            </Space.Compact>
          </div>
        ))}
      </div>
    </div>
  );
};

export default SettingsMobileMenu;
