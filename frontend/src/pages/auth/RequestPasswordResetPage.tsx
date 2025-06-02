import {
  Form,
  Input,
  Button,
  Typography,
  Card,
  Alert,
  ConfigProvider,
  theme as antdTheme,
} from 'antd';
import { useEffect, useState } from 'react';
import { Link } from 'react-router-dom';
import Logo from '@/components/Logo';
import { AuthService } from '@/services/auth';
// import { AuthService } from '@/services/auth'; // Uncomment when integrating real service

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

    const res = await AuthService.requestPasswordReset(values.email);

    setLoading(false);

    if (res.success) {
      setSubmitted(true);
      setCooldown(60);
    } else {
      setError(res.message);
    }
  };

  return (
    <ConfigProvider theme={{ algorithm: antdTheme.defaultAlgorithm }}>
      <div className="flex flex-col lg:flex-row min-h-screen w-full bg-white text-gray-800">
        {/* Left Panel: Form */}
        <div className="flex w-full lg:w-2/5 items-center justify-center px-4 sm:px-6 md:px-10 py-10 min-h-screen">
          <Card className="w-full max-w-md sm:max-w-xl md:max-w-2xl rounded-2xl shadow-none lg:shadow-2xl max-h-[90vh] overflow-auto">
            <div className="flex justify-start mb-6">
              <Logo size="md" showText={false} variant="light" shadow={true} />
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
                <Button
                  type="primary"
                  htmlType="submit"
                  block
                  loading={loading}
                  disabled={cooldown > 0}
                >
                  {submitted
                    ? cooldown > 0
                      ? `Resend (${cooldown}s)`
                      : 'Resend'
                    : 'Send Reset Link'}
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
        </div>

        {/* Right Panel: Visual */}
        <div className="hidden lg:flex w-3/5 relative items-center justify-center bg-gradient-to-br from-blue-600 to-indigo-700">
          <div className="absolute inset-0 bg-black bg-opacity-30" />
          <div className="relative z-10 px-10 py-12 text-center text-white max-w-xl">
            <Title level={2} className="!text-white !mb-6 !text-3xl leading-snug">
              Reset Securely, Return Swiftly
            </Title>
            <Text className="text-lg text-white opacity-90 leading-relaxed">
              Our password recovery system ensures privacy and speed — get back to what matters.
            </Text>
          </div>
        </div>
      </div>
    </ConfigProvider>
  );
}
