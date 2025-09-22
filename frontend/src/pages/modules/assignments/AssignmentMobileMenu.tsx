import { useEffect } from 'react';
import { useUI } from '@/context/UIContext';
import { useAssignment } from '@/context/AssignmentContext';
import { Button, Space, Typography } from 'antd';
import {
  FileDoneOutlined,
  ProfileOutlined,
  MessageOutlined,
  SettingOutlined,
  ExperimentOutlined,
  BarChartOutlined,
  RightOutlined,
} from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useModule } from '@/context/ModuleContext';
import { useAuth } from '@/context/AuthContext';

const AssignmentMobileMenu = () => {
  const { isMobile } = useUI();
  const { setValue } = useViewSlot();
  const module = useModule();
  const { assignment, readiness } = useAssignment();
  const auth = useAuth();
  const navigate = useNavigate();

  if (!isMobile) return null;

  useEffect(() => {
    setValue(
      <Typography.Text
        className="text-base font-medium text-gray-900 dark:text-gray-100 truncate"
        title={assignment.name}
      >
        {assignment.name}
      </Typography.Text>,
    );
  }, [assignment.name, setValue]);

  const navigateTo = (path: string) => {
    const base = `/modules/${module.id}/assignments/${assignment.id}`;
    navigate(path ? `${base}/${path}` : base);
  };

  const isTeachingStaff =
    auth.isAdmin || auth.isLecturer(module.id) || auth.isAssistantLecturer(module.id);
  const isAssignmentReady = readiness?.is_ready ?? false;

  const sections = [
    {
      title: 'Submissions',
      items: [
        {
          key: 'submissions',
          label: 'Submissions',
          icon: <FileDoneOutlined className="text-lg" />,
          path: 'submissions',
          show: true,
          disabled: !isAssignmentReady,
        },
      ],
    },
    {
      title: 'Communication',
      items: [
        {
          key: 'tickets',
          label: 'Tickets',
          icon: <MessageOutlined className="text-lg" />,
          path: 'tickets',
          show: true,
          disabled: !isAssignmentReady,
        },
      ],
    },
    {
      title: 'Lecturer Tools',
      items: [
        {
          key: 'tasks',
          label: 'Tasks',
          icon: <ProfileOutlined className="text-lg" />,
          path: 'tasks',
          show: isTeachingStaff,
          disabled: false,
        },
        {
          key: 'grades',
          label: 'Grades',
          icon: <BarChartOutlined className="text-lg" />,
          path: 'grades',
          show: isTeachingStaff,
          disabled: !isAssignmentReady,
        },
        {
          key: 'plagiarism',
          label: 'Plagiarism',
          icon: <ExperimentOutlined className="text-lg" />,
          path: 'plagiarism',
          show: isTeachingStaff,
          disabled: !isAssignmentReady,
        },
        {
          key: 'config',
          label: 'Files & Config',
          icon: <SettingOutlined className="text-lg" />,
          path: 'config',
          show: isTeachingStaff,
          disabled: false,
        },
      ],
    },
  ];

  return (
    <div className="bg-gray-50 dark:bg-gray-950 space-y-6 !pb-4">
      {sections
        .map((section) => ({
          ...section,
          items: section.items.filter((item) => item.show),
        }))
        .filter((section) => section.items.length > 0)
        .map((section) => (
          <div key={section.title}>
            <Typography.Text className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 block">
              {section.title}
            </Typography.Text>
            <Space.Compact direction="vertical" className="w-full">
              {section.items.map((item) => (
                <Button
                  key={item.key}
                  type="default"
                  block
                  className={`!h-14 px-4 flex items-center !justify-between text-base ${item.disabled ? '!text-gray-400 dark:!text-gray-600' : ''}`}
                  disabled={item.disabled}
                  onClick={() => {
                    if (item.disabled) return;
                    navigateTo(item.path);
                  }}
                >
                  <Typography.Text
                    className={`flex items-center gap-2 text-left ${item.disabled ? '!text-gray-400 dark:!text-gray-600' : ''}`}
                  >
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

export default AssignmentMobileMenu;
