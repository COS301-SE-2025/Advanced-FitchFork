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
  LockOutlined,
} from '@ant-design/icons';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { useViewSlot } from '@/context/ViewSlotContext';
import { requiresMainForMode, requiresInterpreterForMode } from '@/policies/submission';
import type { SubmissionMode } from '@/types/modules/assignments/config';

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

  const mode = (config?.project?.submission_mode ?? 'manual') as SubmissionMode;
  const showAI = requiresInterpreterForMode(mode);
  const needsMain = requiresMainForMode(mode);

  const fileItems = useMemo(() => {
    const items = [];
    if (needsMain) {
      items.push({
        label: 'Main File',
        path: 'files/main',
        icon: <FileTextOutlined className="text-lg" />,
      });
    }
    items.push(
      { label: 'Makefile', path: 'files/makefile', icon: <FileAddOutlined className="text-lg" /> },
      { label: 'Memo File', path: 'files/memo', icon: <FileOutlined className="text-lg" /> },
      { label: 'Specification', path: 'files/spec', icon: <ProfileOutlined className="text-lg" /> },
    );
    return items;
  }, [needsMain]);

  const groups = useMemo(() => {
    const general = {
      title: 'General',
      items: [
        { label: 'Assignment', path: 'assignment', icon: <SettingOutlined className="text-lg" /> },
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
        { label: 'Security', path: 'security', icon: <LockOutlined className="text-lg" /> },
      ],
    };

    const ai = showAI
      ? [
          {
            title: 'AI',
            items: [
              {
                label: 'AI Settings',
                path: 'gatlam',
                icon: <SettingOutlined className="text-lg" />,
              },
              {
                label: 'Interpreter',
                path: 'interpreter',
                icon: <CodeOutlined className="text-lg" />,
              },
            ],
          },
        ]
      : [];

    const files = { title: 'Files', items: fileItems };

    return [general, ...ai, files];
  }, [fileItems, showAI]);

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto space-y-6">
        <div className="rounded-lg border border-gray-200 dark:border-gray-800 p-4 bg-white dark:bg-neutral-900">
          <Typography.Title level={5} className="!m-0">
            {assignment?.name ?? 'Assignment'}
          </Typography.Title>
          <Typography.Text type="secondary">
            {module.code} • {module.year}
          </Typography.Text>
        </div>

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
