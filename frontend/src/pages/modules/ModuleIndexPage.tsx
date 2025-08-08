import { useEffect } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { useUI } from '@/context/UIContext';
import { useMobilePageHeader } from '@/context/MobilePageHeaderContext';
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

const ModuleIndexPage = () => {
  const { isMobile } = useUI();
  const navigate = useNavigate();
  const { id: moduleId } = useParams();
  const { setContent } = useMobilePageHeader();

  const { code, year } = useModule();
  const auth = useAuth();
  const showPersonnel = auth.isAdmin || auth.isLecturer(Number(moduleId));

  useEffect(() => {
    if (!isMobile) {
      navigate(`/modules/${moduleId}/overview`, { replace: true });
    }
  }, [isMobile, moduleId]);

  if (!isMobile) return null;

  useEffect(() => {
    setContent(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        {code} ({year})
      </Typography.Text>,
    );
  }, [code, year]);

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
    <div className="p-4 space-y-6">
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
                onClick={() => navigate(`/modules/${moduleId}/${item.path}`)}
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
  );
};

export default ModuleIndexPage;
