import { Button, Typography, Space } from 'antd';
import { UserOutlined, PlusOutlined, ReloadOutlined } from '@ant-design/icons';

type Props = {
  onCreate?: () => void;
  onRefresh?: () => void;
};

const { Title, Paragraph } = Typography;

const UsersEmptyState = ({ onCreate, onRefresh }: Props) => {
  return (
    <div className="flex-1 flex items-center justify-center h-full rounded-xl border-2 border-dashed bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800">
      <div className="mx-auto max-w-4xl p-8 sm:p-10 text-center">
        <Space direction="vertical" size="middle" className="w-full">
          <div className="inline-flex items-center gap-2 rounded-full border border-gray-200 dark:border-gray-800 px-3 py-1 text-xs font-medium text-gray-600 dark:text-gray-400">
            <UserOutlined />
            No users found
          </div>

          <Title level={3} className="!m-0 !text-gray-900 dark:!text-gray-100">
            No registered users
          </Title>

          <Paragraph className="!m-0 !text-sm !text-gray-600 dark:!text-gray-400">
            There are no users in the system yet. You can add a new user or refresh to check again.
          </Paragraph>

          <div className="flex flex-col sm:flex-row items-center justify-center gap-2 pt-2">
            {onCreate && (
              <Button
                type="primary"
                icon={<PlusOutlined />}
                onClick={onCreate}
                className="min-w-[200px]"
              >
                Add User
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
  );
};

export default UsersEmptyState;
