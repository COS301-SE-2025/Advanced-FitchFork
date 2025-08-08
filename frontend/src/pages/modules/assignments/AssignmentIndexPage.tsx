import { useEffect } from 'react';
import { useUI } from '@/context/UIContext';
import { useMobilePageHeader } from '@/context/MobilePageHeaderContext';
import { useAssignment } from '@/context/AssignmentContext';
import { Button, Space, Typography } from 'antd';
import {
  FileDoneOutlined,
  ProfileOutlined,
  FileOutlined,
  MessageOutlined,
  FileSearchOutlined,
  CalculatorOutlined,
  SettingOutlined,
  RightOutlined,
  DownloadOutlined,
} from '@ant-design/icons';
import { useNavigate, useParams } from 'react-router-dom';
import AssignmentStatusTag from '@/components/assignments/AssignmentStatusTag';

const AssignmentIndexPage = () => {
  const { isMobile } = useUI();
  const { setContent } = useMobilePageHeader();
  const { assignment } = useAssignment();
  const navigate = useNavigate();
  const { id: moduleId, assignment_id } = useParams();

  if (!isMobile) return null;

  useEffect(() => {
    if (!isMobile) {
      navigate(`/modules/${moduleId}/assignments/${assignment_id}/submissions`, { replace: true });
    }
  }, [isMobile, moduleId, assignment_id]);

  useEffect(() => {
    setContent(
      <Typography.Text
        className="text-base font-medium text-gray-900 dark:text-gray-100 truncate"
        title={assignment.name}
      >
        {assignment.name}
      </Typography.Text>,
    );
  }, [assignment.name]);

  const navigateTo = (path: string) =>
    navigate(`/modules/${moduleId}/assignments/${assignment_id}/${path}`);

  return (
    <div className="bg-gray-50 dark:bg-gray-950 p-4 space-y-6">
      <div className="bg-gray-50 dark:bg-gray-950 border-gray-200 dark:border-gray-800 p-4 mb-0">
        <div className="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-6">
          {/* Left section: Info */}
          <div className="flex-1 space-y-4">
            <div className="flex flex-wrap items-center gap-3">
              <Typography.Title
                level={3}
                className="!mb-0 !text-gray-900 dark:!text-gray-100 !leading-none !text-2xl"
              >
                {assignment.name}
              </Typography.Title>

              <div className="flex items-center">
                <AssignmentStatusTag status={assignment.status} />
              </div>

              {/* {auth.isStudent(module.id) && (
                <Tag color="green" className="!text-xs !font-medium !h-6 !px-2 !flex items-center">
                  Best Mark: 85%
                </Tag>
              )} */}
            </div>

            {assignment.description?.length > 0 && (
              <Typography.Paragraph className="!text-sm !text-gray-600 dark:!text-gray-400">
                {assignment.description}
              </Typography.Paragraph>
            )}

            <Button
              type="link"
              onClick={() => {}}
              icon={<DownloadOutlined />}
              size="small"
              className="!p-0"
            >
              Download Specification
            </Button>
          </div>

          {/* Right section: Actions */}
          <div className="flex flex-col items-start sm:items-end gap-2 w-full sm:w-auto">
            {/* {(!isSetupIncomplete || isStudentOrTutor) &&
              (isStudentOrTutor ? (
                <Button
                  type="primary"
                  onClick={() => setModalOpen(true)}
                  loading={loading}
                  className="w-full sm:w-auto"
                >
                  Submit Assignment
                </Button>
              ) : (
                <Dropdown.Button
                  menu={{ items: menuItems }}
                  type="primary"
                  disabled={loading}
                  onClick={() => setModalOpen(true)}
                  loading={loading}
                  className="w-full sm:w-auto"
                >
                  Submit Assignment
                </Dropdown.Button>
              ))} */}
          </div>
        </div>
      </div>

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

      {/* Tasks & Marking Section */}
      <div>
        <Typography.Text className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2 block">
          Tasks & Marking
        </Typography.Text>
        <Space.Compact direction="vertical" className="w-full">
          {[
            { label: 'Tasks', path: 'tasks', icon: <ProfileOutlined className="text-lg" /> },
            { label: 'Files', path: 'files', icon: <FileOutlined className="text-lg" /> },
            {
              label: 'Memo Output',
              path: 'memo-output',
              icon: <FileSearchOutlined className="text-lg" />,
            },
            {
              label: 'Mark Allocator',
              path: 'mark-allocator',
              icon: <CalculatorOutlined className="text-lg" />,
            },
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

export default AssignmentIndexPage;
