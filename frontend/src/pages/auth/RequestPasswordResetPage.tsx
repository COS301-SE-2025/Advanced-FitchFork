import { Form, Input, Button, Typography, Card, Alert } from 'antd';
import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import { MailOutlined } from '@ant-design/icons';
import Logo from '@/components/common/Logo';
import { requestPasswordReset } from '@/services/auth';

const { Title, Text } = Typography;

export default function RequestPasswordResetPage() {
  const [form] = Form.useForm();
  const [submitted, setSubmitted] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [cooldown, setCooldown] = useState(0);

  useEffect(() => {
    if (cooldown <= 0) return;
    const interval = setInterval(() => setCooldown((prev) => prev - 1), 1000);
    return () => clearInterval(interval);
  }, [cooldown]);

  const handleFinish = async (values: { email: string }) => {
    setError(null);
    setLoading(true);

    const res = await requestPasswordReset(values.email);

    setLoading(false);

    if (res.success) {
      setSubmitted(true);
      setCooldown(60);
    } else {
      setError(res.message);
    }
  };

  return (
    <Card
      title={
        <div className="flex my-2 items-center gap-2">
          <Logo size="md" showText={false} />
          <Title level={4} className="!m-0">
            Forgot your password?
          </Title>
        </div>
      }
      className="w-full max-w-md sm:max-w-lg !rounded-xl shadow-md hover:shadow-lg transition-shadow dark:bg-gray-900 dark:border-gray-800"
    >
      <div className="mb-6">
        <Text className="text-gray-600 block text-sm sm:text-base">
          Enter your email and we&apos;ll send you a password reset link. If your account exists,
          you should receive an email within a few minutes.
        </Text>
      </div>

      <Form
        form={form}
        layout="vertical"
        onFinish={handleFinish}
        onValuesChange={() => setError(null)}
        size="large"
      >
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

        <Form.Item
          label={<span className="text-sm sm:text-base">Email</span>}
          name="email"
          rules={[
            { required: true, message: 'Please enter your email address' },
            { type: 'email', message: 'Enter a valid email address' },
          ]}
        >
          <Input prefix={<MailOutlined />} placeholder="student@up.ac.za" allowClear />
        </Form.Item>

        <Form.Item className="mt-6">
          <Button type="primary" htmlType="submit" block loading={loading} disabled={cooldown > 0}>
            {submitted ? (cooldown > 0 ? `Resend (${cooldown}s)` : 'Resend') : 'Send Reset Link'}
          </Button>
        </Form.Item>

        {submitted && (
          <div className="text-center mt-4">
            <Text type="success">If the account exists, a reset link has been sent.</Text>
          </div>
        )}
      </Form>

      <div className="text-center mt-6">
        <Link to="/login" className="text-blue-600 hover:underline">
          Back to Login
        </Link>
      </div>
    </Card>
  );
}
