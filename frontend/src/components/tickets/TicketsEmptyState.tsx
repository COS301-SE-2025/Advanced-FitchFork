import { Button, Typography, Space, Tag } from 'antd';
import { PlusOutlined, ReloadOutlined, InfoCircleOutlined } from '@ant-design/icons';

type Props = {
  assignmentName?: string;
  /** Are tickets currently enabled for this assignment? */
  ticketsEnabled?: boolean;
  /** Open the create ticket flow */
  onCreate?: () => void;
  /** Refresh the list */
  onRefresh?: () => void;
};

const { Title, Paragraph } = Typography;

const TicketsEmptyState = ({
  assignmentName = 'this assignment',
  ticketsEnabled = true,
  onCreate,
  onRefresh,
}: Props) => {
  return (
    <div className="w-full">
      <div className="rounded-xl border-2 border-dashed bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800">
        <div className="mx-auto max-w-4xl p-8 sm:p-10 text-center">
          <Space direction="vertical" size="middle" className="w-full">
            <div className="inline-flex items-center gap-2 rounded-full border border-gray-200 dark:border-gray-800 px-3 py-1 text-xs font-medium text-gray-600 dark:text-gray-400">
              <InfoCircleOutlined />
              No tickets yet
            </div>

            <Title level={3} className="!m-0 !text-gray-900 dark:!text-gray-100">
              Start a discussion for <span className="font-semibold">{assignmentName}</span>
            </Title>

            <Paragraph className="!m-0 !text-sm !text-gray-600 dark:!text-gray-400">
              Use tickets to ask questions, report issues, or request clarifications.
            </Paragraph>

            <div className="flex flex-col sm:flex-row items-center justify-center gap-2 pt-2">
              {ticketsEnabled ? (
                <>
                  <Button
                    type="primary"
                    icon={<PlusOutlined />}
                    onClick={onCreate}
                    disabled={!onCreate}
                    className="min-w-[200px]"
                  >
                    Create ticket
                  </Button>
                  {onRefresh && (
                    <Button icon={<ReloadOutlined />} onClick={onRefresh}>
                      Refresh
                    </Button>
                  )}
                </>
              ) : (
                <Tag color="red" className="!text-sm">
                  Tickets disabled — contact your lecturer
                </Tag>
              )}
            </div>
          </Space>
        </div>
      </div>
    </div>
  );
};

export default TicketsEmptyState;
