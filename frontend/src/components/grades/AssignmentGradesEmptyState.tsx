import { Button, Typography, Space } from 'antd';
import { InfoCircleOutlined, ReloadOutlined } from '@ant-design/icons';

type Props = {
  onRefresh?: () => void;
  loading?: boolean;
};

const { Title, Paragraph, Text } = Typography;

const AssignmentGradesEmptyState = ({ onRefresh, loading }: Props) => {
  return (
    <div className="flex-1 flex items-center justify-center h-full rounded-xl border-2 border-dashed bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800">
      <div className="mx-auto max-w-4xl p-8 sm:p-10 text-center">
        <Space direction="vertical" size="middle" className="w-full">
          <div className="inline-flex items-center gap-2 rounded-full border border-gray-200 dark:border-gray-800 px-3 py-1 text-xs font-medium text-gray-600 dark:text-gray-400">
            <InfoCircleOutlined />
            Empty grades
          </div>

          <Title level={3} className="!m-0 !text-gray-900 dark:!text-gray-100">
            No grades to display
          </Title>

          <Paragraph className="!m-0 !text-sm !text-gray-600 dark:!text-gray-400">
            Grades are derived from students&apos; submissions according to the assignment&apos;s
            grading policy (Last/Best). When new submissions are made, use Refresh to update this
            view.
          </Paragraph>

          <div className="flex flex-col sm:flex-row items-center justify-center gap-2 pt-2">
            {onRefresh && (
              <Button
                type="primary"
                icon={<ReloadOutlined />}
                onClick={onRefresh}
                loading={loading}
                data-testid="grades-empty-refresh"
              >
                Refresh
              </Button>
            )}
          </div>

          <Text type="secondary" className="!text-xs">
            Tip: ensure the grading policy is set correctly in Assignment Config (Last or Best).
          </Text>
        </Space>
      </div>
    </div>
  );
};

export default AssignmentGradesEmptyState;
