import { Card } from 'antd';
import { UserOutlined, MailOutlined } from '@ant-design/icons';
import type { User } from '@/types/users';
import type { ReactNode } from 'react';
import UserAdminTag from './UserAdminTag';

const { Meta } = Card;

interface Props {
  user: User;
  actions?: ReactNode[];
}

const UserCard = ({ user, actions = [] }: Props) => {
  return (
    <Card
      hoverable
      actions={actions}
      className="rounded-xl border border-gray-200 dark:border-gray-800 transition-shadow duration-200 hover:shadow-md"
    >
      <Meta
        avatar={<UserOutlined style={{ fontSize: 24 }} />}
        title={
          <div className="flex justify-between items-center">
            <span className="font-semibold">{user.username}</span>
            <UserAdminTag admin={user.admin} />
          </div>
        }
        description={
          <div className="mt-1 text-sm text-gray-600 dark:text-gray-400">
            <MailOutlined className="mr-1" />
            {user.email}
          </div>
        }
      />
    </Card>
  );
};

export default UserCard;
