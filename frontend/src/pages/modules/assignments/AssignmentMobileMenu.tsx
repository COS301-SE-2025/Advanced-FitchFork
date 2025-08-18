import { useEffect } from 'react';
import { useUI } from '@/context/UIContext';
import { useAssignment } from '@/context/AssignmentContext';
import { Button, Space, Typography } from 'antd';
import {
  FileDoneOutlined,
  ProfileOutlined,
  FileOutlined,
  MessageOutlined,
  SettingOutlined,
  RightOutlined,
} from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useModule } from '@/context/ModuleContext';

const AssignmentMobileMenu = () => {
  const { isMobile } = useUI();
  const { setValue } = useViewSlot();
  const module = useModule();
  const { assignment } = useAssignment();
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
  }, [assignment.name]);

  const navigateTo = (path: string) =>
    navigate(`/modules/${module.id}/assignments/${assignment.id}/${path}`);

  return (
    <div className="bg-gray-50 dark:bg-gray-950 space-y-6">
      {/* Submissions Section */}
      <div>
        <Typography.Text className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 block">
          Submissions
        </Typography.Text>
        <Space.Compact direction="vertical" className="w-full">
          <Button
            type="default"
            block
            className="!h-14 px-4 flex items-center !justify-between text-base"
            onClick={() => navigateTo('submissions')}
          >
            <Typography.Text className="flex items-center gap-2 text-left">
              <FileDoneOutlined className="text-lg" />
              Submissions
            </Typography.Text>
            <RightOutlined />
          </Button>
        </Space.Compact>
      </div>

      {/* Tickets Section */}
      <div>
        <Typography.Text className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 block">
          Communication
        </Typography.Text>
        <Space.Compact direction="vertical" className="w-full">
          <Button
            type="default"
            block
            className="!h-14 px-4 flex items-center !justify-between text-base"
            onClick={() => navigateTo('tickets')}
          >
            <Typography.Text className="flex items-center gap-2 text-left">
              <MessageOutlined className="text-lg" />
              Tickets
            </Typography.Text>
            <RightOutlined />
          </Button>
        </Space.Compact>
      </div>

      {/* Tasks & Resources Section */}
      <div>
        <Typography.Text className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 block">
          Tasks & Resources
        </Typography.Text>
        <Space.Compact direction="vertical" className="w-full">
          {[
            { label: 'Tasks', path: 'tasks', icon: <ProfileOutlined className="text-lg" /> },
            { label: 'Files', path: 'files', icon: <FileOutlined className="text-lg" /> },
          ].map(({ label, path, icon }) => (
            <Button
              key={path}
              type="default"
              block
              className="!h-14 px-4 flex items-center !justify-between text-base"
              onClick={() => navigateTo(path)}
            >
              <Typography.Text className="flex items-center gap-2 text-left">
                {icon}
                {label}
              </Typography.Text>
              <RightOutlined />
            </Button>
          ))}
        </Space.Compact>
      </div>

      {/* Configuration Section */}
      <div>
        <Typography.Text className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 block">
          Configuration
        </Typography.Text>
        <Space.Compact direction="vertical" className="w-full">
          <Button
            type="default"
            block
            className="!h-14 px-4 flex items-center !justify-between text-base"
            onClick={() => navigateTo('config')}
          >
            <Typography.Text className="flex items-center gap-2 text-left">
              <SettingOutlined className="text-lg" />
              Config
            </Typography.Text>
            <RightOutlined />
          </Button>
        </Space.Compact>
      </div>
    </div>
  );
};

export default AssignmentMobileMenu;
