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

  const navigateTo = (path: string) =>
    navigate(`/modules/${module.id}/assignments/${assignment.id}/${path}`);

  const isLecturerLike =
    auth.isAdmin || auth.isLecturer(module.id) || auth.isAssistantLecturer(module.id);

  return (
    <div className="bg-gray-50 dark:bg-gray-950 space-y-6 !pb-4">
      {/* Submissions Section */}
      <div>
        <Typography.Text className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 block">
          Submissions
        </Typography.Text>
        <Space.Compact direction="vertical" className="w-full">
          <Button
            type="default"
            block
            disabled={!readiness?.is_ready}
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

      {/* Communication Section */}
      <div>
        <Typography.Text className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 block">
          Communication
        </Typography.Text>
        <Space.Compact direction="vertical" className="w-full">
          <Button
            type="default"
            block
            disabled={!readiness?.is_ready}
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

      {/* Lecturer/Admin/Assistant Lecturer Extras */}
      {isLecturerLike && (
        <div>
          <Typography.Text className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 block">
            Lecturer Tools
          </Typography.Text>
          <Space.Compact direction="vertical" className="w-full">
            <Button
              type="default"
              block
              disabled={!readiness?.config_present}
              className="!h-14 px-4 flex items-center !justify-between text-base"
              onClick={() => navigateTo('tasks')}
            >
              <Typography.Text className="flex items-center gap-2 text-left">
                <ProfileOutlined className="text-lg" />
                Tasks
              </Typography.Text>
              <RightOutlined />
            </Button>
            <Button
              type="default"
              block
              disabled={!readiness?.config_present}
              className="!h-14 px-4 flex items-center !justify-between text-base"
              onClick={() => navigateTo('plagiarism')}
            >
              <Typography.Text className="flex items-center gap-2 text-left">
                <ExperimentOutlined className="text-lg" />
                Plagiarism
              </Typography.Text>
              <RightOutlined />
            </Button>
            <Button
              type="default"
              block
              disabled={!readiness?.is_ready}
              className="!h-14 px-4 flex items-center !justify-between text-base"
              onClick={() => navigateTo('grades')}
            >
              <Typography.Text className="flex items-center gap-2 text-left">
                <BarChartOutlined className="text-lg" />
                Grades
              </Typography.Text>
              <RightOutlined />
            </Button>
          </Space.Compact>
        </div>
      )}

      {/* Configuration (only for lecturer, assistant lecturer, or admin) */}
      {isLecturerLike && (
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
                Files &amp; Config
              </Typography.Text>
              <RightOutlined />
            </Button>
          </Space.Compact>
        </div>
      )}
    </div>
  );
};

export default AssignmentMobileMenu;
