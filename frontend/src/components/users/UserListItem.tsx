import { List, Avatar } from 'antd';
import { UserOutlined, MailOutlined } from '@ant-design/icons';
import type { User } from '@/types/users';
import React from 'react';
import UserAdminTag from './UserAdminTag';

type Props = {
  user: User;
  onClick?: (u: User) => void;
};

const UserListItem: React.FC<Props> = ({ user, onClick }) => {
  const handleClick = () => onClick?.(user);

  return (
    <List.Item
      key={user.id}
      className="dark:bg-gray-900 hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer transition"
      onClick={handleClick}
      data-cy={`entity-${user.id}`}
    >
      <List.Item.Meta
        avatar={<Avatar icon={<UserOutlined />} />}
        title={
          <div className="flex justify-between items-center">
            <span className="font-semibold text-black dark:text-white">{user.username}</span>
            <UserAdminTag admin={user.admin} />
          </div>
        }
        description={
          <div className="mt-0.5 text-sm text-gray-600 dark:text-gray-400 flex items-center">
            <MailOutlined className="mr-1" />
            <span className="truncate">{user.email}</span>
          </div>
        }
      />
    </List.Item>
  );
};

export default UserListItem;
