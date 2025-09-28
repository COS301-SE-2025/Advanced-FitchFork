import { List, Typography, Empty, Tag, Tooltip, message } from 'antd';
import { MailOutlined, TeamOutlined } from '@ant-design/icons';
import { useModule } from '@/context/ModuleContext';
import UserAvatar from '@/components/common/UserAvatar';
import { useState } from 'react';

const { Text, Title } = Typography;

const ModuleStaffPanel = () => {
  const module = useModule();
  const [copiedUserId, setCopiedUserId] = useState<number | null>(null);

  const handleCopyEmail = async (email: string, id: number) => {
    if (!email) {
      message.warning('No email available to copy');
      return;
    }
    try {
      await navigator.clipboard.writeText(email);
      setCopiedUserId(id);
      message.success('Email copied!');
      setTimeout(() => setCopiedUserId(null), 1500);
    } catch (err) {
      message.error('Failed to copy email');
    }
  };

  const allStaff = [
    ...module.lecturers.map((user) => ({ ...user, role: 'Lecturer' })),
    ...module.tutors.map((user) => ({ ...user, role: 'Tutor' })),
  ];

  return (
    <div className="h-full min-h-0 flex flex-col w-full bg-white dark:bg-gray-900 rounded-md border border-gray-200 dark:border-gray-800">
      <div className="px-3 py-2 border-b border-gray-200 dark:border-gray-800">
        <div className="flex items-center gap-2">
          <TeamOutlined className="text-gray-500" />
          <Title level={5} className="!mb-0">
            Module Staff
          </Title>
        </div>
      </div>

      {allStaff.length === 0 ? (
        <div className="flex-1 flex items-center justify-center px-3 py-6">
          <Empty
            image={Empty.PRESENTED_IMAGE_SIMPLE}
            description="No staff assigned to this module."
          />
        </div>
      ) : (
        <List
          className="flex-1 overflow-y-auto"
          dataSource={allStaff}
          renderItem={(user) => {
            const rawEmail = user.email;
            const emailLabel = rawEmail ?? '—';
            return (
              <List.Item className="!px-3">
                <div className="flex items-center justify-between w-full gap-3">
                  <div className="flex items-center gap-3 min-w-0">
                    <UserAvatar user={user} size="default" />
                    <div className="min-w-0">
                      <Text strong className="truncate block">
                        {user.username}
                      </Text>
                      <Text type="secondary" className="!text-[12px] truncate">
                        {emailLabel}
                      </Text>
                    </div>
                  </div>

                  <div className="flex items-center gap-2 text-gray-500 dark:text-gray-400 text-sm shrink-0">
                    <Tag color={user.role === 'Lecturer' ? 'blue' : 'purple'}>{user.role}</Tag>
                    <Tooltip title={copiedUserId === user.id ? 'Copied!' : emailLabel}>
                      <MailOutlined
                        onClick={() => handleCopyEmail(rawEmail ?? '', user.id)}
                        className="cursor-pointer !text-blue-500 text-lg"
                      />
                    </Tooltip>
                  </div>
                </div>
              </List.Item>
            );
          }}
        />
      )}
    </div>
  );
};

export default ModuleStaffPanel;
