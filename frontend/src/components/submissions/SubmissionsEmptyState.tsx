import { Button, Typography, Space, Tag } from 'antd';
import { CloudUploadOutlined, ReloadOutlined, InfoCircleOutlined } from '@ant-design/icons';
import { useAuth } from '@/context/AuthContext';
import { useModule } from '@/context/ModuleContext';

type Props = {
  assignmentName?: string;
  isAssignmentOpen?: boolean;
  onSubmit?: () => void;
  onRefresh?: () => void;
};

const SubmissionsEmptyState = ({
  assignmentName = 'this assignment',
  isAssignmentOpen = true,
  onSubmit,
  onRefresh,
}: Props) => {
  const module = useModule();
  const auth = useAuth();
  const isStudent = auth.isStudent(module.id);

  const titleText = isStudent
    ? isAssignmentOpen
      ? `No submission for ${assignmentName}`
      : `You didn't submit for ${assignmentName}`
    : isAssignmentOpen
      ? 'No submissions yet'
      : 'No submissions for this assignment';

  const descriptionText = isStudent
    ? isAssignmentOpen
      ? 'You have not submitted your work yet. Submit now to have it marked.'
      : 'You did not submit your work before the deadline.'
    : isAssignmentOpen
      ? 'No students have submitted work for this assignment yet.'
      : 'This assignment is closed; no more submissions can be made.';

  return (
    <div className="flex-1 flex items-center justify-center h-full rounded-xl border-2 border-dashed bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800">
      <div className="mx-auto max-w-4xl p-8 sm:p-10 text-center">
        <Space direction="vertical" size="middle" className="w-full">
          <div className="inline-flex items-center gap-2 rounded-full border border-gray-200 dark:border-gray-800 px-3 py-1 text-xs font-medium text-gray-600 dark:text-gray-400">
            <InfoCircleOutlined />
            {isStudent ? 'No submission yet' : 'No submissions yet'}
          </div>

          <Typography.Title level={3} className="!m-0 !text-gray-900 dark:!text-gray-100">
            {titleText}
          </Typography.Title>

          <Typography.Paragraph className="!m-0 !text-sm !text-gray-600 dark:!text-gray-400">
            {descriptionText}
          </Typography.Paragraph>

          <div className="flex flex-col sm:flex-row items-center justify-center gap-2 pt-2">
            {isAssignmentOpen ? (
              <>
                {onSubmit ? (
                  <Button
                    type="primary"
                    icon={<CloudUploadOutlined />}
                    onClick={onSubmit}
                    className="min-w-[200px]"
                  >
                    Submit now
                  </Button>
                ) : // No submit button for staff/tutors
                null}
                {onRefresh && (
                  <Button icon={<ReloadOutlined />} onClick={onRefresh}>
                    Refresh
                  </Button>
                )}
              </>
            ) : (
              <Tag color="red" className="!text-sm">
                Assignment closed — submissions disabled
              </Tag>
            )}
          </div>
        </Space>
      </div>
    </div>
  );
};

export default SubmissionsEmptyState;
