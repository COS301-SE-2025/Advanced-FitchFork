import { Button, Typography, Space } from 'antd';
import { InfoCircleOutlined, PlusOutlined, ReloadOutlined } from '@ant-design/icons';

type Props = {
  isStaff: boolean;
  onCreate?: () => void;
  onRefresh?: () => void;
  loading?: boolean;
};

const { Title, Paragraph, Text } = Typography;

const AttendanceEmptyState = ({ isStaff, onCreate, onRefresh, loading }: Props) => {
  return (
    <div className="flex-1 flex items-center justify-center h-full rounded-xl border-2 border-dashed bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800">
      <div className="mx-auto max-w-4xl p-8 sm:p-10 text-center">
        <Space direction="vertical" size="middle" className="w-full">
          <div className="inline-flex items-center gap-2 rounded-full border border-gray-200 dark:border-gray-800 px-3 py-1 text-xs font-medium text-gray-600 dark:text-gray-400">
            <InfoCircleOutlined />
            Empty attendance sessions
          </div>

          <Title level={3} className="!m-0 !text-gray-900 dark:!text-gray-100">
            No attendance sessions yet
          </Title>

          <Paragraph className="!m-0 !text-sm !text-gray-600 dark:!text-gray-400">
            {isStaff ? (
              <>Create your first session to start recording attendance.</>
            ) : (
              <>Staff can create sessions from this page.</>
            )}
          </Paragraph>

          <div className="flex items-center justify-center gap-2 pt-2">
            {isStaff && onCreate && (
              <Button
                type="primary"
                icon={<PlusOutlined />}
                onClick={onCreate}
                loading={loading}
                data-testid="attendance-empty-create"
              >
                Create Session
              </Button>
            )}

            {onRefresh && (
              <Button
                icon={<ReloadOutlined />}
                onClick={onRefresh}
                data-testid="attendance-empty-refresh"
              >
                Refresh
              </Button>
            )}
          </div>

          {!isStaff && (
            <Text type="secondary" className="!text-xs">
              You’ll see sessions here once a staff member creates them.
            </Text>
          )}
        </Space>
      </div>
    </div>
  );
};

export default AttendanceEmptyState;
