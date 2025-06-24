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
import { useNavigate, useSearchParams } from 'react-router-dom';
import Logo from '@/components/Logo';
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
    if (!token) {
      setError('Missing password reset token.');
    }
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
    <ConfigProvider theme={{ algorithm: antdTheme.defaultAlgorithm }}>
      <div className="flex flex-col lg:flex-row min-h-screen w-full bg-white text-gray-800">
        {/* Left Panel */}
        <div className="flex w-full lg:w-2/5 items-center justify-center px-4 sm:px-6 md:px-10 py-10 min-h-screen">
          <Card className="w-full max-w-md sm:max-w-xl md:max-w-2xl rounded-2xl shadow-none lg:shadow-2xl max-h-[90vh] overflow-auto">
            <div className="flex justify-start mb-6">
              <Logo size="md" showText={false} variant="light" shadow={true} />
            </div>

            <div className="text-center mb-8">
              <Title level={2} className="!mb-2 text-2xl sm:text-3xl md:text-4xl">
                Set a New Password
              </Title>
              <Text className="text-gray-600 block text-sm sm:text-base md:text-lg">
                Enter and confirm your new password below.
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
                label={<span className="text-sm sm:text-base">New Password</span>}
                name="password"
                rules={[
                  { required: true, message: 'Please enter a new password' },
                  { min: 8, message: 'Password must be at least 8 characters long' },
                  {
                    pattern: /^(?=.*[A-Za-z])(?=.*\d).+$/,
                    message: 'Password must include at least one letter and one number',
                  },
                ]}
              >
                <Input.Password placeholder="Enter new password" />
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
                <Input.Password placeholder="Confirm new password" />
              </Form.Item>

              <Form.Item className="mt-6">
                <Button type="primary" htmlType="submit" block loading={loading} disabled={!token}>
                  Reset Password
                </Button>
              </Form.Item>
            </Form>

            <div className="text-center mt-6">
              <Button type="link" href="/login" className="text-blue-600">
                Back to Login
              </Button>
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
