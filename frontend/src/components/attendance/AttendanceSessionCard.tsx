// src/components/modules/attendance/AttendanceSessionCard.tsx
import { Card, Avatar, Typography, Tag, Progress, Tooltip } from 'antd';
import {
  QrcodeOutlined,
  ClockCircleOutlined,
  PlayCircleFilled,
  PauseCircleOutlined,
} from '@ant-design/icons';
import dayjs from 'dayjs';
import type { AttendanceSession } from '@/types/modules/attendance';

const { Meta } = Card;
const { Text } = Typography;

type Props = {
  session: AttendanceSession;
  onClick?: (session: AttendanceSession) => void;
  actions?: React.ReactNode[];
  hoverable?: boolean;
};

export default function AttendanceSessionCard({
  session,
  onClick,
  actions,
  hoverable = true,
}: Props) {
  const total = session.student_count ?? 0;
  const attended = session.attended_count ?? 0;
  const pct = total > 0 ? Math.round((attended / total) * 100) : 0;

  const strokeColor = pct >= 75 ? '#52c41a' : pct >= 40 ? '#faad14' : '#ff4d4f';

  const tip = (
    <div className="text-xs">
      <div>
        <strong>Attended:</strong> {attended}
      </div>
      <div>
        <strong>Total students:</strong> {total}
      </div>
      <div>
        <strong>Percentage:</strong> {pct}%
      </div>
    </div>
  );

  const handleClick = () => onClick?.(session);

  return (
    <Card
      hoverable={hoverable}
      onClick={handleClick}
      className="w-full cursor-pointer !bg-white dark:!bg-gray-900"
      actions={actions}
      data-testid="attendance-session-card"
    >
      <Meta
        avatar={<Avatar icon={<QrcodeOutlined />} style={{ backgroundColor: '#1890ff' }} />}
        title={
          <div className="flex items-center justify-between gap-2">
            <span className="text-black dark:text-white truncate">{session.title}</span>
            {session.active ? (
              <Tag color="success" icon={<PlayCircleFilled />}>
                Active
              </Tag>
            ) : (
              <Tag icon={<PauseCircleOutlined />}>Inactive</Tag>
            )}
          </div>
        }
        description={
          <div className="flex flex-col gap-2">
            <Tooltip title={tip}>
              <Progress
                percent={pct}
                size="small"
                strokeColor={strokeColor}
                status={session.active ? 'active' : undefined}
                className="mb-0"
                aria-label={`Attendance ${pct}% (${attended}/${total})`}
              />
            </Tooltip>

            <div className="flex items-center justify-between text-xs text-gray-600 dark:text-gray-300">
              <span className="inline-flex items-center gap-1">
                <ClockCircleOutlined /> {session.rotation_seconds}s rotation
              </span>
              <Text type="secondary" className="!mb-0">
                {dayjs(session.created_at).format('YYYY-MM-DD HH:mm')}
              </Text>
            </div>
          </div>
        }
      />
    </Card>
  );
}
