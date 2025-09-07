import { useEffect, useMemo } from 'react';
import { useNavigate } from 'react-router-dom';
import { Button, Space, Typography } from 'antd';
import {
  ThunderboltOutlined,
  CheckSquareOutlined,
  RightOutlined,
  FileTextOutlined,
  FileAddOutlined,
  FileOutlined,
  SettingOutlined,
  CodeOutlined,
  ProfileOutlined,
} from '@ant-design/icons';

import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { useViewSlot } from '@/context/ViewSlotContext';

type MobileItem = {
  label: string;
  path: string; // relative to /config route
  icon: React.ReactNode;
};

type MobileGroup = {
  title: string;
  items: MobileItem[];
};

const ConfigMobileMenu = () => {
  const { assignment, config } = useAssignment();
  const module = useModule();
  const navigate = useNavigate();
  const { setValue } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Config
      </Typography.Text>,
    );
  }, [setValue]);

  // Mode-aware visibility
  const submissionMode = config?.project?.submission_mode ?? 'manual';
  const isGatlam = submissionMode === 'gatlam';

  // Files group: omit "Main File" when in GATLAM mode (matches desktop)
  const fileItems: MobileItem[] = useMemo(() => {
    const base: MobileItem[] = [
      // main file (hidden for GATLAM)
      ...(isGatlam
        ? []
        : [
            {
              label: 'Main File',
              path: 'files/main',
              icon: <FileTextOutlined className="text-lg" />,
            },
          ]),
      {
        label: 'Makefile',
        path: 'files/makefile',
        icon: <FileAddOutlined className="text-lg" />,
      },
      {
        label: 'Memo File',
        path: 'files/memo',
        icon: <FileOutlined className="text-lg" />,
      },
      {
        label: 'Specification',
        path: 'files/spec',
        icon: <ProfileOutlined className="text-lg" />,
      },
    ];
    return base;
  }, [isGatlam]);

  const groups: MobileGroup[] = useMemo(() => {
    const general: MobileGroup = {
      title: 'General',
      items: [
        {
          label: 'Assignment',
          path: 'assignment',
          icon: <SettingOutlined className="text-lg" />,
        },
        {
          label: 'Execution Limits',
          path: 'execution',
          icon: <ThunderboltOutlined className="text-lg" />,
        },
        {
          label: 'Marking & Feedback',
          path: 'marking',
          icon: <CheckSquareOutlined className="text-lg" />,
        },
        {
          label: 'Output',
          path: 'output',
          icon: <CodeOutlined className="text-lg" />,
        },
      ],
    };

    const maybeGatlam: MobileGroup[] = isGatlam
      ? [
          {
            title: 'GATLAM',
            items: [
              { label: 'GATLAM', path: 'gatlam', icon: <SettingOutlined className="text-lg" /> },
              {
                label: 'Interpreter',
                path: 'interpreter',
                icon: <CodeOutlined className="text-lg" />,
              },
            ],
          },
        ]
      : [];

    const files: MobileGroup = {
      title: 'Files',
      items: fileItems,
    };

    return [general, ...maybeGatlam, files];
  }, [fileItems, isGatlam]);

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto space-y-6">
        {/* Header card */}
        <div className="rounded-lg border border-gray-200 dark:border-gray-800 p-4 bg-white dark:bg-neutral-900">
          <Typography.Title level={5} className="!m-0">
            {assignment?.name ?? 'Assignment'}
          </Typography.Title>
          <Typography.Text type="secondary">
            {module.code} • {module.year}
          </Typography.Text>
        </div>

        {/* Groups */}
        {groups.map((group) => (
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
                  onClick={() => navigate(item.path)} // relative to /config
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
