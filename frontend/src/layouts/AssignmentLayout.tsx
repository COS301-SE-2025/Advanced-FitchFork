import { useState } from 'react';
import { useNavigate, useLocation, Outlet } from 'react-router-dom';
import {
  Spin,
  Dropdown,
  Button,
  Alert,
  Modal,
  Upload,
  Checkbox,
  Tag,
  Typography,
  Segmented,
  Space,
} from 'antd';
import type { MenuProps } from 'antd';
import {
  CheckCircleOutlined,
  CloseCircleOutlined,
  DownloadOutlined,
  UploadOutlined,
} from '@ant-design/icons';

import { useModule } from '@/context/ModuleContext';
import { useAuth } from '@/context/AuthContext';
import { useAssignment } from '@/context/AssignmentContext';

import { message } from '@/utils/message';

import {
  closeAssignment,
  downloadAssignmentFile,
  openAssignment,
} from '@/services/modules/assignments';
import { generateMemoOutput } from '@/services/modules/assignments/memo-output';
import { generateMarkAllocator } from '@/services/modules/assignments/mark-allocator';
import { submitAssignment } from '@/services/modules/assignments/submissions/post';

import type { AssignmentReadiness } from '@/types/modules/assignments';
import AssignmentSetup from '@/pages/modules/assignments/steps/AssignmentSetup';
import AssignmentStatusTag from '@/components/assignments/AssignmentStatusTag';
import EventBus from '@/utils/EventBus';
import { useUI } from '@/context/UIContext';

const { Title, Paragraph } = Typography;

const AssignmentLayout = () => {
  const module = useModule();
  const { assignment, readiness, refreshAssignment } = useAssignment();
  const auth = useAuth();
  const { isMobile } = useUI();
  const navigate = useNavigate();
  const location = useLocation();

  const [modalOpen, setModalOpen] = useState(false);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [isPractice, setIsPractice] = useState(false);
  const [setupOpen, setSetupOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const [downloadingSpec, setDownloadingSpec] = useState(false);

  const basePath = `/modules/${module.id}/assignments/${assignment.id}`;
  const isLecturerOrAdmin = auth.isLecturer(module.id) || auth.isAdmin;
  const isStudentOrTutor = auth.isStudent(module.id) || auth.isTutor(module.id);
  const showTabs = !auth.isStudent(module.id);

  const isOnSubmissions =
    location.pathname.startsWith(`${basePath}/submissions`) || location.pathname === `${basePath}`;

  const showHeaderCard = !isMobile || (isMobile && isOnSubmissions);

  const segments = [
    {
      value: `${basePath}/submissions`,
      label: 'Submissions',
    },
    {
      value: `${basePath}/tickets`,
      label: 'Tickets',
      disabled: !readiness?.is_ready,
    },
    ...(auth.isLecturer(module.id) || auth.isAdmin
      ? [
          {
            value: `${basePath}/tasks`,
            label: 'Tasks',
            disabled: !readiness?.config_present,
          },
          {
            value: `${basePath}/files`,
            label: 'Files',
            disabled: !readiness?.config_present,
          },
          { value: `${basePath}/config`, label: 'Config' },
          {
            value: `${basePath}/stats`,
            label: 'Statistics',
            disabled: !readiness?.is_ready,
          },
        ]
      : []),
  ];

  const activeKey =
    segments.find(
      (seg) => location.pathname === seg.value || location.pathname.startsWith(seg.value + '/'),
    )?.value || `${basePath}/submissions`;

  const handleOpenAssignment = async () => {
    setLoading(true);
    const hide = message.loading('Opening assignment...');
    try {
      const res = await openAssignment(module.id, assignment.id);
      hide();
      if (res.success) {
        await refreshAssignment();
        message.success('Assignment opened successfully');
      } else {
        message.error(res.message || 'Failed to open assignment');
      }
    } catch {
      hide();
      message.error('Unexpected error while opening assignment');
    } finally {
      setLoading(false);
    }
  };

  const handleCloseAssignment = async () => {
    setLoading(true);
    const hide = message.loading('Closing assignment...');
    try {
      const res = await closeAssignment(module.id, assignment.id);
      hide();
      if (res.success) {
        await refreshAssignment();
        message.success('Assignment closed successfully');
      } else {
        message.error(res.message || 'Failed to close assignment');
      }
    } catch {
      hide();
      message.error('Unexpected error while closing assignment');
    } finally {
      setLoading(false);
    }
  };

  const handleGenerateMemoOutput = async () => {
    setLoading(true);
    const hide = message.loading('Generating memo ouptut...');
    try {
      const res = await generateMemoOutput(module.id, assignment.id);
      hide();
      if (res.success) {
        await refreshAssignment();
        message.success('Memo output generated');
      } else {
        message.error(res.message || 'Failed to generate memo output');
      }
    } catch {
      hide();
      message.error('An unexpected error occurred');
    } finally {
      setLoading(false);
    }
  };

  const handleGenerateMarkAllocator = async () => {
    setLoading(true);
    const hide = message.loading('Generating mark allocator...');
    try {
      const res = await generateMarkAllocator(module.id, assignment.id);
      hide(); // close the loading message
      if (res.success) {
        await refreshAssignment();
        message.success('Mark allocator generated');
      } else {
        message.error(res.message || 'Failed to generate mark allocator');
      }
    } catch {
      hide();
      message.error('Failed to generate mark allocator');
    } finally {
      setLoading(false);
    }
  };

  const handleSubmitAssignment = async () => {
    if (!selectedFile) {
      message.error('Please select a file to submit.');
      return;
    }
    setModalOpen(false);
    setLoading(true);
    const hide = message.loading('Submitting assignment...');
    try {
      await submitAssignment(module.id, assignment.id, selectedFile, isPractice);
      await refreshAssignment();
      message.success('Submission successful');
      EventBus.emit('submission:updated');
    } catch {
      message.error('Submission failed');
    } finally {
      hide();
      setLoading(false);
      setSelectedFile(null);
      setIsPractice(false);
    }
  };

  const handleDownloadSpec = async () => {
    const specFile = assignment.files?.find((f) => f.file_type === 'spec');

    if (!specFile) {
      message.error('No specification file found for this assignment.');
      return;
    }

    setDownloadingSpec(true);
    const hide = message.loading('Starting specification download...');

    try {
      await downloadAssignmentFile(module.id, assignment.id, specFile.id);
      // apiDownload should trigger the browser download; nothing else to do here
      message.success('Download started');
    } catch (e) {
      message.error('Failed to download specification');
    } finally {
      hide();
      setDownloadingSpec(false);
    }
  };

  const menuItems: MenuProps['items'] = [
    {
      key: 'memo',
      label: 'Generate Memo Output',
      onClick: handleGenerateMemoOutput,
      disabled: loading,
    },
    {
      key: 'mark',
      label: 'Generate Mark Allocator',
      onClick: handleGenerateMarkAllocator,
      disabled: loading,
    },
    {
      type: 'divider',
    },
    {
      key: 'open',
      label: 'Open Assignment',
      onClick: handleOpenAssignment,
      disabled: loading || !['ready', 'closed', 'archived'].includes(assignment?.status ?? ''),
    },
    {
      key: 'close',
      label: 'Close Assignment',
      onClick: handleCloseAssignment,
      disabled: loading || assignment?.status !== 'open',
    },
    {
      type: 'divider',
    },
    {
      key: 'archive',
      label: 'Archive Assignment',
      disabled: loading,
    },
    {
      key: 'delete',
      label: 'Delete Assignment',
      danger: true,
      disabled: loading,
    },
  ];

  if (!assignment) {
    return <Spin className="p-6" tip="Loading assignment..." />;
  }

  const isSetupIncomplete = !readiness?.is_ready;

  return (
    <div className="h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto p-4">
        <Space direction="vertical" size="middle" className="w-full">
          {showHeaderCard && (
            <div className="bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800 mb-0 p-4 rounded-md border ">
              <div className="flex flex-col sm:flex-row sm:items-start sm:justify-between gap-6">
                {/* Left section: Info */}
                <div className="flex-1 space-y-4">
                  <div className="flex flex-wrap items-center gap-3">
                    <Title
                      level={3}
                      className="!mb-0 !text-gray-900 dark:!text-gray-100 !leading-none !text-2xl"
                    >
                      {assignment.name}
                    </Title>

                    <div className="flex items-center">
                      <AssignmentStatusTag status={assignment.status} />
                    </div>

                    {auth.isStudent(module.id) && (
                      <Tag
                        color="green"
                        className="!text-xs !font-medium !h-6 !px-2 !flex items-center"
                      >
                        Best Mark: 85%
                      </Tag>
                    )}
                  </div>

                  {assignment.description?.length > 0 && (
                    <Paragraph className="!text-sm !text-gray-600 dark:!text-gray-400">
                      {assignment.description}
                    </Paragraph>
                  )}

                  {assignment.files?.some((f) => f.file_type === 'spec') && (
                    <Button
                      type="link"
                      onClick={handleDownloadSpec}
                      icon={<DownloadOutlined />}
                      size="small"
                      className="!p-0"
                      loading={downloadingSpec}
                      disabled={downloadingSpec}
                    >
                      Download Specification
                    </Button>
                  )}
                </div>

                {/* Right section: Actions */}

                <div className="flex flex-col items-start sm:items-end gap-2 w-full sm:w-auto">
                  {(!isSetupIncomplete || isStudentOrTutor) &&
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
                        rootClassName="!w-full [&>button:first-child]:w-full"
                      >
                        Submit Assignment
                      </Dropdown.Button>
                    ))}
                </div>
              </div>
              {showTabs && (
                <div className=" hidden md:block mt-4">
                  <Segmented
                    options={segments}
                    value={activeKey}
                    onChange={(key) => navigate(key as string)}
                    size="middle"
                    block
                    className="dark:!bg-gray-950"
                  />
                </div>
              )}
            </div>
          )}

          {assignment.due_date && new Date() > new Date(assignment.due_date) && (
            <Alert
              message="Past Due Date - Practice submissions only"
              description="Practice submissions won't be considered for your final mark."
              type="warning"
              showIcon
              className="!mb-4"
            />
          )}

          {isStudentOrTutor ? (
            <Outlet />
          ) : isSetupIncomplete && isLecturerOrAdmin ? (
            <div className="flex flex-col items-center justify-center text-center bg-white dark:bg-gray-950 border border-dashed border-gray-300 dark:border-gray-700 rounded-lg p-12 space-y-6">
              <h2 className="text-2xl font-semibold text-gray-900 dark:text-gray-100">
                Assignment setup incomplete
              </h2>
              <p className="text-gray-600 dark:text-gray-400 max-w-xl">
                This assignment is not yet ready for use. Please complete the setup process to
                configure the necessary files, tasks, and settings before students can submit or
                view it.
              </p>
              <div className="space-y-2 w-full max-w-2xl text-left">
                {[
                  { key: 'config_present', label: 'Configuration file' },
                  { key: 'main_present', label: 'Main file' },
                  { key: 'makefile_present', label: 'Makefile' },
                  { key: 'memo_present', label: 'Memo file' },
                  { key: 'tasks_present', label: 'Tasks' },
                  { key: 'memo_output_present', label: 'Memo Output' },
                  { key: 'mark_allocator_present', label: 'Mark Allocator' },
                ].map((item) => {
                  const complete = readiness?.[item.key as keyof AssignmentReadiness];
                  return (
                    <div
                      key={item.key}
                      className="flex items-center justify-between p-3 rounded border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900"
                    >
                      <span className="text-sm font-medium text-gray-800 dark:text-gray-200">
                        {item.label}
                      </span>

                      <span
                        className={`flex items-center gap-1 text-xs px-2 py-1 rounded ${
                          complete
                            ? 'bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300'
                            : 'bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300'
                        }`}
                      >
                        {complete ? <CheckCircleOutlined /> : <CloseCircleOutlined />}
                        {complete ? 'Complete' : 'Incomplete'}
                      </span>
                    </div>
                  );
                })}
              </div>

              <Button
                type="primary"
                size="large"
                onClick={() => setSetupOpen(true)}
                loading={loading}
              >
                Complete Setup
              </Button>
            </div>
          ) : (
            <Outlet />
          )}
        </Space>

        <AssignmentSetup
          open={setupOpen}
          onClose={() => setSetupOpen(false)}
          assignmentId={assignment.id}
          module={module}
          onStepComplete={refreshAssignment}
        />

        <Modal
          open={modalOpen}
          onCancel={() => setModalOpen(false)}
          footer={null}
          title={<Typography.Title level={4}>Submit Assignment</Typography.Title>}
          centered
        >
          <div>
            <Upload
              maxCount={1}
              beforeUpload={(file) => {
                setSelectedFile(file);
                return false; // prevent automatic upload
              }}
              accept=".zip,.tar,.gz,.tgz"
              disabled={loading}
            >
              <Button icon={<UploadOutlined />} disabled={loading}>
                Click to select file
              </Button>
            </Upload>

            <Checkbox
              checked={isPractice}
              onChange={(e) => setIsPractice(e.target.checked)}
              disabled={loading}
              className="!mt-4"
            >
              This is a practice submission
            </Checkbox>

            <div className="flex justify-end gap-2 pt-4">
              <Button onClick={() => setModalOpen(false)} data-cy="submit-modal-cancel">
                Cancel
              </Button>
              <Button
                type="primary"
                onClick={handleSubmitAssignment}
                loading={loading}
                disabled={!selectedFile}
                data-cy="submit-modal-submit"
              >
                Submit
              </Button>
            </div>
          </div>
        </Modal>
      </div>
    </div>
  );
};

export default AssignmentLayout;
