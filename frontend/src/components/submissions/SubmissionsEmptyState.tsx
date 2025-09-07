import { Button, Typography, Space, Tag } from 'antd';
import { CloudUploadOutlined, ReloadOutlined, InfoCircleOutlined } from '@ant-design/icons';

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
  const titleText = isAssignmentOpen
    ? `No submission for ${assignmentName}`
    : `You didn't submit for ${assignmentName}`;

  const descriptionText = isAssignmentOpen
    ? 'You have not submitted your work yet. Submit now to have it marked.'
    : 'You did not submit your work before the deadline.';

  return (
    <div className="flex-1 flex items-center justify-center h-full rounded-xl border-2 border-dashed bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800">
      <div className="mx-auto max-w-4xl p-8 sm:p-10 text-center">
        <Space direction="vertical" size="middle" className="w-full">
          <div className="inline-flex items-center gap-2 rounded-full border border-gray-200 dark:border-gray-800 px-3 py-1 text-xs font-medium text-gray-600 dark:text-gray-400">
            <InfoCircleOutlined />
            No submission yet
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
                ) : (
                  <Button
                    type="primary"
                    icon={<CloudUploadOutlined />}
                    disabled
                    className="min-w-[200px]"
                  >
                    Submit now
                  </Button>
                )}
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
