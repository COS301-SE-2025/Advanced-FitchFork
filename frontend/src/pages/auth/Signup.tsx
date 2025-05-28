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
import { useState } from 'react';
import Logo from '@/components/Logo';
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
    if (values.password !== values.confirmPassword) {
      setFormError('Passwords do not match');
      return;
    }

    const res = await register({
      student_number: values.username,
      email: values.email,
      password: values.password,
    });

    if (res.success) {
      navigate('/home');
    } else {
      setFormError(res.message || 'Registration failed');
    }
  };

  return (
    <ConfigProvider theme={{ algorithm: antdTheme.defaultAlgorithm }}>
      <div className="flex flex-col lg:flex-row min-h-screen w-full bg-white text-gray-800 dark:bg-white dark:text-gray-800">
        {/* Left: Visual Panel */}
        <div className="hidden lg:flex w-1/2 relative items-center justify-center bg-gradient-to-br from-blue-600 to-indigo-700">
          <div className="absolute inset-0 bg-black bg-opacity-30" />
          <div className="relative z-10 px-10 py-12 text-center text-white max-w-xl">
            <Title level={2} className="!text-white !mb-6 !text-3xl leading-snug">
              Code Smarter. Grade Faster.
            </Title>
            <Text className="text-lg text-white opacity-90 leading-relaxed">
              FitchFork helps educators automate code assessments with reliability and clarity.
            </Text>
          </div>
        </div>

        {/* Right: Signup Section */}
        <div className="flex w-full lg:w-1/2 items-center justify-center px-4 sm:px-6 md:px-10 min-h-screen">
          <Card className="w-full max-w-2xl rounded-2xl shadow-2xl">
            <div className="flex justify-start mb-6">
              <Logo size="md" showText={false} variant="light" />
            </div>

            <div className="text-center mb-10">
              <Title level={2} className="!mb-2 text-2xl sm:text-3xl md:text-4xl">
                Create your account
              </Title>
              <Text
                type="secondary"
                className="block text-sm sm:text-base md:text-lg text-gray-600"
              >
                Sign up to join FitchFork
              </Text>
            </div>

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
                label={<span className="text-sm sm:text-base">Student Number</span>}
                name="username"
                rules={[{ required: true, message: 'Please enter your student number' }]}
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
                rules={[{ required: true, message: 'Please enter your password' }]}
                className="mt-4"
              >
                <Input.Password placeholder="••••••••" />
              </Form.Item>

              <Form.Item
                label={<span className="text-sm sm:text-base">Confirm Password</span>}
                name="confirmPassword"
                rules={[{ required: true, message: 'Please confirm your password' }]}
                className="mt-4"
              >
                <Input.Password placeholder="••••••••" />
              </Form.Item>

              <Form.Item className="mt-8">
                <Button type="primary" htmlType="submit" block size="large">
                  Create Account
                </Button>
              </Form.Item>
            </Form>

            <Divider plain className="!mt-10">
              or
            </Divider>

            <Text className="block text-center text-xs sm:text-sm md:text-base text-gray-600">
              Already have an account?{' '}
              <Link href="/login" className="text-blue-600">
                Sign in
              </Link>
            </Text>
          </Card>
        </div>
      </div>
    </ConfigProvider>
  );
}
