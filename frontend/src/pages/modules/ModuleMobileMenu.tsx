import { useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useModule } from '@/context/ModuleContext';
import { useAuth } from '@/context/AuthContext';
import { Button, Space, Typography } from 'antd';
import {
  HomeOutlined,
  FileTextOutlined,
  BarChartOutlined,
  BookOutlined,
  UserOutlined,
  NotificationOutlined,
  RightOutlined,
} from '@ant-design/icons';
import { useViewSlot } from '@/context/ViewSlotContext';
import { ModuleHeader } from '@/components/modules';
import { formatModuleCode } from '@/utils/modules';

const ModuleMobileMenu = () => {
  const navigate = useNavigate();
  const module = useModule();
  const { setValue } = useViewSlot();
  const auth = useAuth();
  const showPersonnel = auth.isAdmin || auth.isLecturer(module.id);

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        {formatModuleCode(module.code)}
      </Typography.Text>,
    );
  }, [module]);

  const groupedNavItems = [
    {
      title: 'Module Navigation',
      items: [
        {
          label: 'Overview',
          path: 'overview',
          icon: <HomeOutlined className="text-lg" />,
        },
        {
          label: 'Announcements',
          path: 'announcements',
          icon: <NotificationOutlined className="text-lg" />,
        },
        {
          label: 'Assignments',
          path: 'assignments',
          icon: <FileTextOutlined className="text-lg" />,
        },
      ],
    },
    {
      title: 'Learning Resources',
      items: [
        {
          label: 'Grades',
          path: 'grades',
          icon: <BarChartOutlined className="text-lg" />,
        },
        {
          label: 'Resources',
          path: 'resources',
          icon: <BookOutlined className="text-lg" />,
        },
      ],
    },
    ...(showPersonnel
      ? [
          {
            title: 'Staff',
            items: [
              {
                label: 'Personnel',
                path: 'personnel',
                icon: <UserOutlined className="text-lg" />,
              },
            ],
          },
        ]
      : []),
  ];

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto p-4 space-y-6">
        <ModuleHeader module={module} />
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
                  onClick={() => navigate(`/modules/${module.id}/${item.path}`)}
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

export default ModuleMobileMenu;
