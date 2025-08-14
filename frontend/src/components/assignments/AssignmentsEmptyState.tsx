import { Button, Typography, Space } from 'antd';
import { InfoCircleOutlined, PlusOutlined, ReloadOutlined } from '@ant-design/icons';

type Props = {
  /** e.g. "COS123" or full name; used in the heading */
  moduleLabel?: string;
  /** Can the viewer create assignments? (lecturer/admin) */
  canCreate?: boolean;
  /** Optional: open the "Create Assignment" flow */
  onCreate?: () => void;
  /** Optional: refresh the list */
  onRefresh?: () => void;
};

const { Title, Paragraph } = Typography;

const AssignmentsEmptyState = ({
  moduleLabel = 'this module',
  canCreate = false,
  onCreate,
  onRefresh,
}: Props) => {
  const title = canCreate ? 'No assignments yet' : 'No assignments available';
  const description = canCreate
    ? `Create your first assignment for ${moduleLabel}.`
    : `There aren&apos;t any assignments available for ${moduleLabel} right now.`;

  return (
    <div className="w-full">
      <div className="rounded-xl border-2 border-dashed bg-white dark:bg-gray-950 border-gray-300 dark:border-gray-700">
        <div className="mx-auto max-w-4xl p-8 sm:p-10 text-center">
          <Space direction="vertical" size="middle" className="w-full">
            <div className="inline-flex items-center gap-2 rounded-full border border-gray-200 dark:border-gray-800 px-3 py-1 text-xs font-medium text-gray-600 dark:text-gray-400">
              <InfoCircleOutlined />
              No assignments
            </div>

            <Title level={3} className="!m-0 !text-gray-900 dark:!text-gray-100">
              {title}
            </Title>

            <Paragraph className="!m-0 !text-sm !text-gray-600 dark:!text-gray-400">
              {description}
            </Paragraph>

            <div className="flex flex-col sm:flex-row items-center justify-center gap-2 pt-2">
              {canCreate && (
                <Button
                  type="primary"
                  icon={<PlusOutlined />}
                  onClick={onCreate}
                  className="min-w-[200px]"
                  disabled={!onCreate}
                >
                  Add assignment
                </Button>
              )}
              {onRefresh && (
                <Button icon={<ReloadOutlined />} onClick={onRefresh}>
                  Refresh
                </Button>
              )}
            </div>
          </Space>
        </div>
      </div>
    </div>
  );
};

export default AssignmentsEmptyState;
