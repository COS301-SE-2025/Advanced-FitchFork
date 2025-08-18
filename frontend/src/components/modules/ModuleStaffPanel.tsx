import { Card, List, Typography, Empty, Tag, Tooltip, message } from 'antd';
import { MailOutlined } from '@ant-design/icons';
import { useModule } from '@/context/ModuleContext';
import { useUI } from '@/context/UIContext';
import UserAvatar from '@/components/common/UserAvatar';
import { useState } from 'react';

const { Text, Title } = Typography;

const ModuleStaffPanel = () => {
  const module = useModule();
  const { isSm } = useUI();
  const [copiedUserId, setCopiedUserId] = useState<number | null>(null);

  const handleCopyEmail = async (email: string, id: number) => {
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
    <Card
      title={
        <div className="flex items-center justify-between gap-2">
          <Title level={isSm ? 4 : 5} className="!mb-0">
            Module Staff
          </Title>
        </div>
      }
      className="!bg-white dark:!bg-gray-900 border-gray-200 dark:!border-gray-800 rounded-lg"
      styles={{
        body: { paddingTop: 12, paddingBottom: 12, paddingLeft: 0, paddingRight: 0 },
      }}
    >
      {allStaff.length === 0 ? (
        <Empty
          image={Empty.PRESENTED_IMAGE_SIMPLE}
          description="No staff assigned to this module."
        />
      ) : (
        <List
          dataSource={allStaff}
          renderItem={(user) => (
            <List.Item className="!px-6">
              <div className="flex items-center justify-between w-full">
                {/* Left: Avatar + Name */}
                <div className="flex items-center gap-3 min-w-0">
                  <UserAvatar user={user} size="default" />
                  <div className="min-w-0">
                    <Text strong className="truncate block">
                      {user.username}
                    </Text>
                  </div>
                </div>

                {/* Right: Role + Email icon */}
                <div className="flex items-center gap-2 text-gray-500 dark:text-gray-400 text-sm shrink-0">
                  <Tag color={user.role === 'Lecturer' ? 'blue' : 'purple'}>{user.role}</Tag>
                  <Tooltip title={copiedUserId === user.id ? 'Copied!' : user.email}>
                    <MailOutlined
                      onClick={() => handleCopyEmail(user.email, user.id)}
                      className="cursor-pointer !text-blue-500 text-lg"
                    />
                  </Tooltip>
                </div>
              </div>
            </List.Item>
          )}
        />
      )}
    </Card>
  );
};

export default ModuleStaffPanel;
