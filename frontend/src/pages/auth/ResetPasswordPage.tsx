import { Form, Input, Button, Typography, Card, Alert } from 'antd';
import { useEffect, useState } from 'react';
import { Link, useNavigate, useSearchParams } from 'react-router-dom';
import { LockOutlined, SafetyOutlined } from '@ant-design/icons';
import Logo from '@/components/common/Logo';
import { resetPassword } from '@/services/auth';

const { Title, Text } = Typography;

export default function ResetPasswordPage() {
  const [form] = Form.useForm();
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();

  const token = searchParams.get('token');

  useEffect(() => {
    if (!token) setError('Missing password reset token.');
  }, [token]);

  const handleFinish = async (values: { password: string; confirmPassword: string }) => {
    if (!token) {
      setError('Invalid or missing reset token.');
      return;
    }
    setLoading(true);
    setError(null);
    const res = await resetPassword(token, values.password);
    setLoading(false);
    if (res.success) {
      navigate('/password-reset-success');
    } else {
      setError(res.message);
    }
  };

  return (
    <Card
      title={
        <div className="flex items-center gap-2 my-2">
          <Logo size="md" showText={false} />
          <Title level={4} className="!m-0">
            Reset Password
          </Title>
        </div>
      }
      className="w-full max-w-md sm:max-w-xl md:max-w-2xl rounded-2xl shadow-md hover:shadow-lg transition-shadow dark:bg-neutral-900 dark:border-neutral-800"
    >
      <div className="mb-6 ">
        <Text className="text-gray-600 block text-sm sm:text-base">
          Set a new password for your account.
        </Text>
      </div>

      {!token && (
        <Alert message="Missing password reset token." type="warning" showIcon className="!mb-4" />
      )}

      {error && (
        <Alert
          message={error}
          type="error"
          showIcon
          closable
          onClose={() => setError(null)}
          className="mb-4"
        />
      )}

      <Form
        form={form}
        layout="vertical"
        onFinish={handleFinish}
        onValuesChange={() => setError(null)}
        validateTrigger="onSubmit"
        requiredMark={false}
        size="large"
      >
        <Form.Item
          label="New Password"
          name="password"
          rules={[
            { required: true, message: 'Please enter a new password' },
            { min: 8, message: 'Password must be at least 8 characters long' },
            {
              pattern: /^(?=.*[A-Za-z])(?=.*\d).+$/,
              message: 'Include at least one letter and one number',
            },
          ]}
          hasFeedback
        >
          <Input.Password
            prefix={<LockOutlined />}
            placeholder="Enter new password"
            autoComplete="new-password"
          />
        </Form.Item>

        <Form.Item
          label="Confirm Password"
          name="confirmPassword"
          dependencies={['password']}
          rules={[
            { required: true, message: 'Please confirm your password' },
            ({ getFieldValue }) => ({
              validator(_, value) {
                if (!value || getFieldValue('password') === value) return Promise.resolve();
                return Promise.reject(new Error('Passwords do not match'));
              },
            }),
          ]}
          hasFeedback
        >
          <Input.Password
            prefix={<SafetyOutlined />}
            placeholder="Confirm new password"
            autoComplete="new-password"
          />
        </Form.Item>

        <Form.Item className="mt-6">
          <Button type="primary" htmlType="submit" block loading={loading} disabled={!token}>
            Reset Password
          </Button>
        </Form.Item>
      </Form>

      <div className="text-center mt-6">
        <Link to="/login" className="text-blue-600 hover:underline">
          Back to Login
        </Link>
      </div>
    </Card>
  );
}
