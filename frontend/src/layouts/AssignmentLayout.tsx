import { useEffect, useState } from 'react';
import { useNavigate, useLocation, Outlet, useParams } from 'react-router-dom';
import { Spin, Dropdown, Segmented, Button, Alert, Modal, Upload, Checkbox } from 'antd';
import type { MenuProps } from 'antd';
import {
  CheckCircleOutlined,
  CloseCircleOutlined,
  DownloadOutlined,
  UploadOutlined,
} from '@ant-design/icons';

import { useModule } from '@/context/ModuleContext';
import { useAuth } from '@/context/AuthContext';
import { AssignmentProvider } from '@/context/AssignmentContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

import { message } from '@/utils/message';

import { getAssignmentDetails, getAssignmentReadiness } from '@/services/modules/assignments';
import { generateMemoOutput, getMemoOutput } from '@/services/modules/assignments/memo-output';
import {
  generateMarkAllocator,
  getMarkAllocator,
} from '@/services/modules/assignments/mark-allocator';
import { submitAssignment } from '@/services/modules/assignments/submissions/post';

import type { Assignment, AssignmentFile, AssignmentReadiness } from '@/types/modules/assignments';
import AssignmentSetup from '@/pages/modules/assignments/steps/AssignmentSetup';
import AssignmentStatusTag from '@/components/assignments/AssignmentStatusTag';
import type { MemoTaskOutput } from '@/types/modules/assignments/memo-output';
import type { MarkAllocatorItem } from '@/types/modules/assignments/mark-allocator';
import EventBus from '@/utils/EventBus';

interface AssignmentDetails extends Assignment {
  files: AssignmentFile[];
}

const AssignmentLayout = () => {
  const module = useModule();
  const { isStudent, isLecturer, isAdmin } = useAuth();
  const { assignment_id } = useParams();
  const navigate = useNavigate();
  const location = useLocation();
  const { setBreadcrumbLabel } = useBreadcrumbContext();

  const [assignment, setAssignment] = useState<AssignmentDetails | null>(null);
  const [memoOutput, setMemoOutput] = useState<MemoTaskOutput[]>([]);
  const [markAllocator, setMarkAllocator] = useState<MarkAllocatorItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [readiness, setReadiness] = useState<AssignmentReadiness | null>(null);
  const [setupOpen, setSetupOpen] = useState(false);

  const [modalOpen, setModalOpen] = useState(false);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [isPractice, setIsPractice] = useState(false);

  const assignmentIdNum = Number(assignment_id);
  const basePath = `/modules/${module.id}/assignments/${assignment_id}`;
  const showTabs = !isStudent(module.id) || isAdmin;

  const segments = [
    { value: `${basePath}/submissions`, label: 'Submissions' },
    ...(isLecturer(module.id) || isAdmin
      ? [
          {
            value: `${basePath}/files`,
            label: 'Files',
            disabled: !readiness?.config_present,
          },
          {
            value: `${basePath}/tasks`,
            label: 'Tasks',
            disabled: !readiness?.config_present,
          },
          {
            value: `${basePath}/memo-output`,
            label: 'Memo Output',
            disabled: !readiness?.config_present || !readiness?.tasks_present,
          },
          {
            value: `${basePath}/mark-allocator`,
            label: 'Mark Allocator',
            disabled: !readiness?.memo_output_present,
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

  const refreshAssignment = async () => {
    const [detailsRes, readinessRes, memoOutputRes, allocatorRes] = await Promise.all([
      getAssignmentDetails(module.id, assignmentIdNum),
      getAssignmentReadiness(module.id, assignmentIdNum),
      getMemoOutput(module.id, assignmentIdNum),
      getMarkAllocator(module.id, assignmentIdNum),
    ]);

    if (detailsRes.success && detailsRes.data) {
      setAssignment(detailsRes.data);
      setBreadcrumbLabel(
        `modules/${module.id}/assignments/${detailsRes.data.id}`,
        detailsRes.data.name,
      );
    }

    if (readinessRes.success) {
      setReadiness(readinessRes.data);
    }

    if (memoOutputRes.success && memoOutputRes.data) {
      setMemoOutput(memoOutputRes.data);
    }

    if (allocatorRes.success && allocatorRes.data) {
      setMarkAllocator(allocatorRes.data.tasks);
    }
  };

  useEffect(() => {
    if (!isNaN(assignmentIdNum)) {
      refreshAssignment().finally(() => setLoading(false));
    } else {
      setLoading(false);
    }
  }, [module.id, assignmentIdNum]);

  const handleGenerateMemoOutput = async () => {
    setLoading(true);
    const hide = message.loading('Generating memo ouptut...');
    try {
      const res = await generateMemoOutput(module.id, assignmentIdNum);
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
      const res = await generateMarkAllocator(module.id, assignmentIdNum);
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
      await submitAssignment(module.id, assignmentIdNum, selectedFile, isPractice);
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

  // TODO: Implement download of assignment specification file
  const handleDownloadSpec = () => {
    message.info('Download Specification: feature not yet implemented.');
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
    <div className="p-4 space-y-6">
      <div className="bg-white dark:bg-gray-950 rounded border border-gray-200 dark:border-gray-700 px-4 py-4 mb-4">
        <div className="flex justify-between items-start gap-3">
          <div>
            <div className="flex items-center gap-2 mb-2">
              <h1 className="text-xl font-semibold text-gray-900 dark:text-gray-100 !m-0">
                {assignment.name}
              </h1>
              <AssignmentStatusTag status={assignment.status} />
            </div>
            <p className="text-sm text-gray-500 dark:text-gray-400">
              Manage assignment #{assignment.id} in {module.code}
            </p>
            <Button
              type="default"
              onClick={handleDownloadSpec}
              icon={<DownloadOutlined />}
              size="small"
              className="mt-2"
            >
              Download Specification
            </Button>
          </div>

          <div>
            {isSetupIncomplete ? (
              <Button
                type="primary"
                onClick={() => setSetupOpen(true)}
                disabled={loading}
                loading={loading}
              >
                Complete Setup
              </Button>
            ) : (
              <Dropdown.Button
                menu={{ items: menuItems }}
                type="primary"
                disabled={loading}
                onClick={() => setModalOpen(true)}
                loading={loading}
              >
                Submit Assignment
              </Dropdown.Button>
            )}
          </div>
        </div>
      </div>

      {assignment.due_date && new Date() > new Date(assignment.due_date) && (
        <Alert
          message="Past Due Date - Practice submissions only"
          description="Practice submissions won't be considered for your final mark."
          type="warning"
          showIcon
          className="!mb-4"
        />
      )}

      {isSetupIncomplete ? (
        <div className="flex flex-col items-center justify-center text-center bg-white dark:bg-gray-950 border border-dashed border-gray-300 dark:border-gray-700 rounded-lg p-12 mt-8 space-y-6">
          <h2 className="text-2xl font-semibold text-gray-900 dark:text-gray-100">
            Assignment setup incomplete
          </h2>
          <p className="text-gray-600 dark:text-gray-400 max-w-xl">
            This assignment is not yet ready for use. Please complete the setup process to configure
            the necessary files, tasks, and settings before students can submit or view it.
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

          <Button type="primary" size="large" onClick={() => setSetupOpen(true)} loading={loading}>
            Complete Setup
          </Button>
        </div>
      ) : (
        <>
          {showTabs && (
            <div className="mb-4">
              <Segmented
                options={segments}
                value={activeKey}
                onChange={(key) => navigate(key as string)}
                size="middle"
                block
              />
            </div>
          )}

          <AssignmentProvider
            value={{ assignment, memoOutput, markAllocator, readiness, refreshAssignment }}
          >
            <Outlet />
          </AssignmentProvider>
        </>
      )}

      <AssignmentSetup
        open={setupOpen}
        onClose={() => setSetupOpen(false)}
        assignmentId={assignment.id}
        module={module}
        onStepComplete={refreshAssignment}
      />

      <Modal
        title="Submit Assignment"
        open={modalOpen}
        onCancel={() => setModalOpen(false)}
        onOk={handleSubmitAssignment}
        okButtonProps={{ loading }}
        okText="Submit"
      >
        <Upload
          maxCount={1}
          beforeUpload={(file) => {
            setSelectedFile(file);
            return false;
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
          style={{ marginTop: 16 }}
          disabled={loading}
        >
          This is a practice submission
        </Checkbox>
      </Modal>
    </div>
  );
};

export default AssignmentLayout;
