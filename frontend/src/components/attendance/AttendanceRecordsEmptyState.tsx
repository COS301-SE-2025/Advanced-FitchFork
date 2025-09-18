// src/components/attendance/AttendanceRecordsEmptyState.tsx
import { Button, Typography, Space } from 'antd';
import {
  InfoCircleOutlined,
  EyeOutlined,
  UserAddOutlined,
  ReloadOutlined,
} from '@ant-design/icons';

type Props = {
  isStaff?: boolean;
  onOpenProjector?: () => void;
  onManualMark?: () => void;
  onRefresh?: () => void;
  loading?: boolean;
};

const { Title, Paragraph, Text } = Typography;

export default function AttendanceRecordsEmptyState({
  isStaff = false,
  onOpenProjector,
  onManualMark,
  onRefresh,
  loading = false,
}: Props) {
  return (
    <div className="flex-1 flex items-center justify-center h-full rounded-xl border-2 border-dashed bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800">
      <div className="mx-auto max-w-3xl p-8 sm:p-10 text-center">
        <Space direction="vertical" size="middle" className="w-full">
          <div className="inline-flex items-center gap-2 rounded-full border border-gray-200 dark:border-gray-800 px-3 py-1 text-xs font-medium text-gray-600 dark:text-gray-400">
            <InfoCircleOutlined />
            No attendance recorded
          </div>

          <Title level={3} className="!m-0 !text-gray-900 dark:!text-gray-100">
            Nothing here yet
          </Title>

          <Paragraph className="!m-0 !text-sm !text-gray-600 dark:!text-gray-400">
            When students mark attendance, their records will appear here in real time.
            {isStaff && (
              <>
                {' '}
                <Text>
                  You can also kick things off from the projector or mark a student manually.
                </Text>
              </>
            )}
          </Paragraph>

          <div className="flex flex-col sm:flex-row items-center justify-center gap-3">
            {isStaff && onOpenProjector && (
              <Button
                type="primary"
                icon={<EyeOutlined />}
                onClick={onOpenProjector}
                className="min-w-[200px]"
                loading={loading}
              >
                Open Projector
              </Button>
            )}
            {isStaff && onManualMark && (
              <Button
                icon={<UserAddOutlined />}
                onClick={onManualMark}
                className="min-w-[200px]"
                loading={loading}
              >
                Mark by username
              </Button>
            )}
            {onRefresh && (
              <Button
                icon={<ReloadOutlined />}
                onClick={onRefresh}
                className="min-w-[200px]"
                loading={loading}
              >
                Refresh
              </Button>
            )}
          </div>
        </Space>
      </div>
    </div>
  );
}
