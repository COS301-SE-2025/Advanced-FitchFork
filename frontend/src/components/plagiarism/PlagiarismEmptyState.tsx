import { Button, Typography, Space, Dropdown } from 'antd';
import {
  InfoCircleOutlined,
  PlusOutlined,
  ReloadOutlined,
  MoreOutlined,
  ExperimentOutlined,
} from '@ant-design/icons';
import type { MenuProps } from 'antd';

type Props = {
  onCreate?: () => void;
  onRefresh?: () => void;
  onGenerate?: () => void;
  loading?: boolean;
};

const { Title, Paragraph } = Typography;

const PlagiarismEmptyState = ({ onCreate, onRefresh, onGenerate, loading }: Props) => {
  // Only secondary actions go in the dropdown (primary Run MOSS is its own button)
  const secondaryMenuItems: MenuProps['items'] = [
    ...(onCreate
      ? [
          {
            key: 'add_case',
            icon: <PlusOutlined />,
            label: <span data-testid="control-action-add_case">Add Case</span>,
            onClick: () => onCreate?.(),
          },
        ]
      : []),
  ];

  return (
    <div className="flex-1 flex items-center justify-center h-full rounded-xl border-2 border-dashed bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800">
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
            {(onGenerate || onCreate) && (
              <Space.Compact>
                {/* Primary action (like ControlBar primary) */}
                <Button
                  type="primary"
                  icon={<ExperimentOutlined />}
                  onClick={() => onGenerate?.()}
                  loading={loading}
                  data-testid="control-action-run-moss"
                >
                  Run MOSS
                </Button>

                {/* Secondary actions dropdown (like ControlBar secondary) */}
                {secondaryMenuItems.length > 0 && (
                  <Dropdown
                    data-testid="control-action-dropdown"
                    menu={{ items: secondaryMenuItems }}
                    placement="bottomRight"
                  >
                    <Button type="primary" icon={<MoreOutlined />} aria-label="More actions" />
                  </Dropdown>
                )}
              </Space.Compact>
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
  );
};

export default PlagiarismEmptyState;
