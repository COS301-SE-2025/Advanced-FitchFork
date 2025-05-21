import { useParams } from 'react-router-dom';
import { Form, Input, Select, Button } from 'antd';
import DashboardLayout from '@layouts/DashboardLayout';

const { Option } = Select;

export default function UserEdit() {
  const { id } = useParams<{ id: string }>();

  const [form] = Form.useForm();

  const handleFinish = (values: any) => {
    console.log('Updated user:', { id, ...values });
  };

  return (
    <DashboardLayout
      title={`Edit User ${id}}`}
      description="This page is used to edit a specific users details."
    >
      <Form
        form={form}
        layout="vertical"
        onFinish={handleFinish}
        initialValues={{
          name: 'Alice Johnson',
          email: 'alice@example.com',
          role: 'Admin',
        }}
      >
        <Form.Item name="name" label="Name" rules={[{ required: true }]}>
          <Input />
        </Form.Item>

        <Form.Item name="email" label="Email" rules={[{ required: true, type: 'email' }]}>
          <Input />
        </Form.Item>

        <Form.Item name="role" label="Role" rules={[{ required: true }]}>
          <Select>
            <Option value="Admin">Admin</Option>
            <Option value="User">User</Option>
            <Option value="Moderator">Moderator</Option>
          </Select>
        </Form.Item>

        <Form.Item>
          <Button type="primary" htmlType="submit">
            Save Changes
          </Button>
        </Form.Item>
      </Form>
    </DashboardLayout>
  );
}
