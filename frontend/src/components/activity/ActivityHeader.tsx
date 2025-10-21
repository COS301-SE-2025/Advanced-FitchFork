import { Button, Space, Typography } from 'antd';
import { HistoryOutlined, ReloadOutlined } from '@ant-design/icons';

interface Props {
    onRefresh: () => void;
    disabled?: boolean;
}

const ActivityHeader = ({ onRefresh, disabled = false }: Props) => (
    <div className="rounded-2xl border border-gray-200 bg-white px-4 py-5 shadow-none dark:border-gray-800 dark:bg-gray-900">
        <div className="flex flex-wrap items-center justify-between gap-3">
            <Space size={12} align="center">
                <span className="flex h-10 w-10 items-center justify-center rounded-full bg-blue-100 text-blue-600 dark:bg-blue-950/40 dark:text-blue-300">
                    <HistoryOutlined className="text-xl" />
                </span>
                <div>
                    <Typography.Title level={2} className="!mb-0 text-2xl">
                        My Activity
                    </Typography.Title>
                    <Typography.Text type="secondary">
                        Fast snapshots of what changed across your modules.
                    </Typography.Text>
                </div>
            </Space>

            <Button icon={<ReloadOutlined />} onClick={onRefresh} disabled={disabled}>
                Refresh
            </Button>
        </div>
    </div>
);

export default ActivityHeader;
