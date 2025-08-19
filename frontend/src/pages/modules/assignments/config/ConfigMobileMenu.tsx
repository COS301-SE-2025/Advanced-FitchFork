import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { Button, Space, Typography } from 'antd';
import { ThunderboltOutlined, CheckSquareOutlined, RightOutlined } from '@ant-design/icons';

import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { useViewSlot } from '@/context/ViewSlotContext';

const ConfigMobileMenu = () => {
  const navigate = useNavigate();
  const module = useModule();
  const { assignment } = useAssignment();
  const { setValue } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Config
      </Typography.Text>,
    );
  }, []);

  const groupedNavItems = [
    {
      title: 'Configuration',
      items: [
        {
          label: 'Execution',
          path: 'execution',
          icon: <ThunderboltOutlined className="text-lg" />,
        },
        {
          label: 'Marking & Feedback',
          path: 'marking',
          icon: <CheckSquareOutlined className="text-lg" />,
        },
      ],
    },
  ];

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto space-y-6">
        {/* Lightweight header for context */}
        <div className="rounded-lg border border-gray-200 dark:border-gray-800 p-4 bg-white dark:bg-neutral-900">
          <Typography.Title level={5} className="!m-0">
            {assignment.name}
          </Typography.Title>
          <Typography.Text type="secondary">
            {module.code} • {module.year}
          </Typography.Text>
        </div>

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
                  // Relative navigation from /config
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

export default ConfigMobileMenu;
