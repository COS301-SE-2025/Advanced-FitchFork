import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Button, Card, Form, Input, Typography, Divider, Alert } from 'antd';
import Logo from '@/components/Logo';
import { useAuth } from '@/context/AuthContext';

const { Title, Text, Link } = Typography;

export default function Login() {
  const { login } = useAuth();
  const navigate = useNavigate();
  const [form] = Form.useForm();
  const [formError, setFormError] = useState<string | null>(null);

  const handleFinish = async (values: { username: string; password: string }) => {
    setFormError(null);
    const res = await login(values.username, values.password);
    if (res.success) {
      navigate('/dashboard');
    } else {
      setFormError(res.message);
    }
  };

  return (
    <Card className="w-full max-w-md sm:max-w-xl rounded-2xl shadow-xl">
      <div className="flex justify-start mb-6">
        <Logo size="md" showText={false} shadow />
      </div>

      <div className="text-center mb-8">
        <Title level={2} className="!mb-2 text-2xl sm:text-3xl">
          Welcome back
        </Title>
        <Text className="block text-sm sm:text-base text-gray-600">
          Log in to access your dashboard
        </Text>
      </div>

      {formError && (
        <Alert
          message={formError}
          type="error"
          showIcon
          closable
          onClose={() => setFormError(null)}
          className="mb-4"
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
          <Input placeholder="u00000001" />
        </Form.Item>

        <Form.Item
          label={<span className="text-sm sm:text-base">Password</span>}
          name="password"
          rules={[{ required: true, message: 'Please enter your password' }]}
          className="mt-4"
        >
          <Input.Password placeholder="••••••••" />
        </Form.Item>

        <div className="text-right -mt-2 mb-4">
          <Link href="/forgot-password" className="text-sm text-blue-600">
            Forgot password?
          </Link>
        </div>

        <Form.Item className="mt-6">
          <Button type="primary" htmlType="submit" block size="large">
            Sign In
          </Button>
        </Form.Item>
      </Form>

      <Divider plain className="mt-8">
        or
      </Divider>

      <Text className="block text-center text-sm text-gray-600">
        Don&apos;t have an account?{' '}
        <Link href="/signup" className="text-blue-600">
          Sign up
        </Link>
      </Text>
    </Card>
  );
}
