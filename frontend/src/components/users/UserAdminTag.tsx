import { Tag } from 'antd';

interface Props {
  admin: boolean;
}

const UserAdminTag = ({ admin }: Props) => {
  return admin ? <Tag color="green">Admin</Tag> : <Tag color="default">Regular</Tag>;
};

export default UserAdminTag;
