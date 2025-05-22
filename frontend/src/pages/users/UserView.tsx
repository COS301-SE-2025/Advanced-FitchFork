import { useParams } from 'react-router-dom';
import DashboardLayout from '@layouts/DashboardLayout';
import { Descriptions, Tag } from 'antd';

const mockUserData: Record<string, { name: string; email: string; role: string }> = {
  '1': { name: 'Alice Johnson', email: 'alice@example.com', role: 'Admin' },
  '2': { name: 'Bob Smith', email: 'bob@example.com', role: 'User' },
  '3': { name: 'Charlie Rose', email: 'charlie@example.com', role: 'Moderator' },
  '4': { name: 'Diana Prince', email: 'diana@example.com', role: 'Admin' },
  '5': { name: 'Ethan Hunt', email: 'ethan@example.com', role: 'User' },
  '6': { name: 'Fiona Glenanne', email: 'fiona@example.com', role: 'Moderator' },
  '7': { name: 'George Clooney', email: 'george@example.com', role: 'User' },
  '8': { name: 'Hannah Baker', email: 'hannah@example.com', role: 'Moderator' },
  '9': { name: 'Ian Fleming', email: 'ian@example.com', role: 'Admin' },
  '10': { name: 'Julia Child', email: 'julia@example.com', role: 'User' },
  '11': { name: 'Kevin Durant', email: 'kevin@example.com', role: 'Moderator' },
  '12': { name: 'Lara Croft', email: 'lara@example.com', role: 'Admin' },
};

export default function UserView() {
  const { id } = useParams<{ id: string }>();
  const user = mockUserData[id ?? ''];

  if (!user) {
    return (
      <DashboardLayout title="User Not Found">
        <p>No user with ID {id} found.</p>
      </DashboardLayout>
    );
  }

  return (
    <DashboardLayout title={`User ${id} View`} description="Specific users details.">
      <Descriptions bordered column={1}>
        <Descriptions.Item label="Name">{user.name}</Descriptions.Item>
        <Descriptions.Item label="Email">{user.email}</Descriptions.Item>
        <Descriptions.Item label="Role">
          <Tag
            color={
              user.role === 'Admin' ? 'volcano' : user.role === 'Moderator' ? 'geekblue' : 'green'
            }
          >
            {user.role}
          </Tag>
        </Descriptions.Item>
      </Descriptions>
    </DashboardLayout>
  );
}
