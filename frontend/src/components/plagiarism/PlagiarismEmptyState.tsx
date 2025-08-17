import { Button, Typography, Space } from 'antd';
import { InfoCircleOutlined, PlusOutlined, ReloadOutlined } from '@ant-design/icons';

type Props = {
  onCreate?: () => void;
  onRefresh?: () => void;
};

const { Title, Paragraph } = Typography;

const PlagiarismEmptyState = ({ onCreate, onRefresh }: Props) => {
  return (
    <div className="w-full">
      <div className="rounded-xl border-2 border-dashed bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800">
        <div className="mx-auto max-w-4xl p-8 sm:p-10 text-center">
          <Space direction="vertical" size="middle" className="w-full">
            <div className="inline-flex items-center gap-2 rounded-full border border-gray-200 dark:border-gray-800 px-3 py-1 text-xs font-medium text-gray-600 dark:text-gray-400">
              <InfoCircleOutlined />
              Empty plagiarism cases
            </div>

            <Title level={3} className="!m-0 !text-gray-900 dark:!text-gray-100">
              No plagiarism cases yet
            </Title>

            <Paragraph className="!m-0 !text-sm !text-gray-600 dark:!text-gray-400">
              Create your first case or run a MOSS check to get started.
            </Paragraph>

            <div className="flex flex-col sm:flex-row items-center justify-center gap-2 pt-2">
              {onCreate && (
                <Button
                  type="primary"
                  icon={<PlusOutlined />}
                  onClick={onCreate}
                  className="min-w-[200px]"
                  data-testid="empty-add"
                >
                  Add plagiarism case
                </Button>
              )}
              {onRefresh && (
                <Button icon={<ReloadOutlined />} onClick={onRefresh} data-testid="empty-refresh">
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

export default PlagiarismEmptyState;
