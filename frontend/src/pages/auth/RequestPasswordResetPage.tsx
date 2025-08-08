import { Form, Input, Button, Typography, Card, Alert } from 'antd';
import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
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
    <Card className="w-full max-w-md sm:max-w-xl md:max-w-2xl rounded-2xl shadow-xl max-h-[90vh] overflow-auto">
      <div className="flex justify-start mb-6">
        <Logo size="md" showText={false} shadow />
      </div>

      <div className="text-center mb-8">
        <Title level={2} className="!mb-2 text-2xl sm:text-3xl md:text-4xl">
          Forgot your password?
        </Title>
        <Text className="text-gray-600 block text-sm sm:text-base md:text-lg">
          Enter your email and we’ll send you a reset link.
        </Text>
      </div>

      <Form
        form={form}
        layout="vertical"
        onFinish={handleFinish}
        onValuesChange={() => setError(null)}
        className="mt-4"
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
          <Input placeholder="student@up.ac.za" />
        </Form.Item>

        <Form.Item className="mt-6">
          <Button type="primary" htmlType="submit" block loading={loading} disabled={cooldown > 0}>
            {submitted ? (cooldown > 0 ? `Resend (${cooldown}s)` : 'Resend') : 'Send Reset Link'}
          </Button>
        </Form.Item>

        {submitted && (
          <div className="text-center mt-4">
            <Text className="text-blue-600">
              If the account exists, a reset link has been sent.
            </Text>
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
