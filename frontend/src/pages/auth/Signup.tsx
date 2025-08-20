import { useNavigate } from 'react-router-dom';
import { Button, Card, Form, Input, Typography, Divider, Alert } from 'antd';
import { useState } from 'react';
import Logo from '@/components/common/Logo';
import { useAuth } from '@/context/AuthContext';

const { Title, Text, Link } = Typography;

export default function Signup() {
  const navigate = useNavigate();
  const { register } = useAuth();
  const [form] = Form.useForm();
  const [formError, setFormError] = useState<string | null>(null);

  const handleFinish = async (values: {
    username: string;
    email: string;
    password: string;
    confirmPassword: string;
  }) => {
    const res = await register(values.username, values.email, values.password);

    if (res.success) {
      navigate('/dashboard');
    } else {
      setFormError(res.message || 'Registration failed');
    }
  };

  return (
    <Card
      title={
        <div className="flex items-center my-2 gap-2 justify-start rounded-md">
          <Logo size="md" showText={false} />
          <Title level={4} className="!m-0">
            Signup
          </Title>
        </div>
      }
      className="w-full max-w-md sm:max-w-lg !rounded-xl shadow-xl"
    >
      {formError && (
        <Alert
          message={formError}
          type="error"
          showIcon
          closable
          onClose={() => setFormError(null)}
          className="!mb-4"
        />
      )}

      <Form
        layout="vertical"
        form={form}
        onFinish={handleFinish}
        onValuesChange={() => setFormError(null)}
        size="large"
      >
        <Form.Item
          label={<span className="text-sm sm:text-base">Username</span>}
          name="username"
          rules={[{ required: true, message: 'Please enter your username' }]}
        >
          <Input placeholder="u00000000" />
        </Form.Item>

        <Form.Item
          label={<span className="text-sm sm:text-base">Email</span>}
          name="email"
          rules={[
            { required: true, message: 'Please enter your email' },
            { type: 'email', message: 'Enter a valid email address' },
          ]}
        >
          <Input placeholder="student@up.ac.za" />
        </Form.Item>

        <Form.Item
          label={<span className="text-sm sm:text-base">Password</span>}
          name="password"
          rules={[
            { required: true, message: 'Please enter your password' },
            { min: 8, message: 'Password must be at least 8 characters long' },
            {
              pattern: /^(?=.*[A-Za-z])(?=.*\d).+$/,
              message: 'Password must include at least one letter and one number',
            },
          ]}
        >
          <Input.Password placeholder="••••••••" />
        </Form.Item>

        <Form.Item
          label={<span className="text-sm sm:text-base">Confirm Password</span>}
          name="confirmPassword"
          dependencies={['password']}
          rules={[
            { required: true, message: 'Please confirm your password' },
            ({ getFieldValue }) => ({
              validator(_, value) {
                if (!value || getFieldValue('password') === value) {
                  return Promise.resolve();
                }
                return Promise.reject(new Error('Passwords do not match'));
              },
            }),
          ]}
        >
          <Input.Password placeholder="••••••••" />
        </Form.Item>

        <Form.Item className="mt-6">
          <Button type="primary" htmlType="submit" block size="large">
            Create Account
          </Button>
        </Form.Item>
      </Form>

      <Divider plain className="mt-10">
        or
      </Divider>

      <Text className="block text-center text-sm text-gray-600">
        Already have an account?{' '}
        <Link href="/login" className="text-blue-600">
          Sign in
        </Link>
      </Text>
    </Card>
  );
}
