import { Button, Typography, Space } from 'antd';
import { InfoCircleOutlined, PlusOutlined } from '@ant-design/icons';

type Props = {
  onCreate?: () => void;
  loading?: boolean;
  onGenerateFromMakefile?: () => void;
  canGenerateFromMakefile?: boolean;
  generatingFromMakefile?: boolean;
};

const { Title, Paragraph } = Typography;

const TasksEmptyState = ({
  onCreate,
  loading = false,
  onGenerateFromMakefile,
  canGenerateFromMakefile = false,
  generatingFromMakefile = false,
}: Props) => {
  const showGenerateButton = Boolean(onGenerateFromMakefile && canGenerateFromMakefile);

  return (
    <div className="flex-1 flex items-center justify-center h-full rounded-xl border-2 border-dashed bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800">
      <div className="mx-auto max-w-3xl p-8 sm:p-10 text-center">
        <Space direction="vertical" size="middle" className="w-full">
          <div className="inline-flex items-center gap-2 rounded-full border border-gray-200 dark:border-gray-800 px-3 py-1 text-xs font-medium text-gray-600 dark:text-gray-400">
            <InfoCircleOutlined />
            No tasks configured
          </div>

          <Title level={3} className="!m-0 !text-gray-900 dark:!text-gray-100">
            No tasks yet
          </Title>

          <Paragraph className="!m-0 !text-sm !text-gray-600 dark:!text-gray-400">
            Set up your first task to define how this assignment should be built, run, and assessed.
          </Paragraph>

          {(onCreate || showGenerateButton) && (
            <div className="flex flex-col sm:flex-row items-center justify-center gap-3">
              {onCreate && (
                <Button
                  type="primary"
                  icon={<PlusOutlined />}
                  onClick={onCreate}
                  className="min-w-[200px]"
                  loading={loading}
                >
                  Add task
                </Button>
              )}
              {showGenerateButton && (
                <Button
                  type="default"
                  onClick={onGenerateFromMakefile}
                  className="min-w-[200px]"
                  disabled={!canGenerateFromMakefile || generatingFromMakefile}
                  loading={generatingFromMakefile}
                >
                  Generate Tasks
                </Button>
              )}
            </div>
          )}
        </Space>
      </div>
    </div>
  );
};

export default TasksEmptyState;
