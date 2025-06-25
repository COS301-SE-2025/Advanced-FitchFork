import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import {
  Button,
  Card,
  Form,
  Input,
  Typography,
  Divider,
  ConfigProvider,
  theme as antdTheme,
  Alert,
} from 'antd';
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
      navigate('/home');
    } else {
      setFormError(res.message);
    }
  };

  return (
    <ConfigProvider theme={{ algorithm: antdTheme.defaultAlgorithm }}>
      <div className="flex flex-col lg:flex-row min-h-screen w-full bg-white text-gray-800">
        {/* Left: Form Panel */}
        <div className="flex w-full lg:w-2/5 items-center justify-center px-4 sm:px-6 md:px-10 py-12">
          <Card className="w-full max-w-md sm:max-w-xl rounded-2xl shadow-none lg:shadow-2xl">
            <div className="flex justify-start mb-6">
              <Logo size="md" showText={false} variant="light" shadow={true} />
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
        </div>

        {/* Right: Visual Panel */}
        <div className="hidden lg:flex w-3/5 relative items-center justify-center bg-gradient-to-br from-blue-600 to-indigo-700">
          <div className="absolute inset-0 bg-black bg-opacity-30" />
          <div className="relative z-10 px-6 py-10 text-center text-white max-w-xl">
            <Title level={2} className="!text-white !mb-4 text-2xl xl:!text-3xl leading-snug">
              Automate Code Evaluation with Precision
            </Title>
            <Text className="text-base xl:text-lg text-white opacity-90 leading-relaxed">
              FitchFork helps institutions and instructors streamline marking of programming
              assignments — accurately and at scale.
            </Text>
          </div>
        </div>
      </div>
    </ConfigProvider>
  );
}
