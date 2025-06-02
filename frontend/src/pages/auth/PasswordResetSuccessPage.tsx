import { Button, Typography, Card, ConfigProvider, theme as antdTheme } from 'antd';
import { Link } from 'react-router-dom';
import Logo from '@/components/Logo';

const { Title, Text } = Typography;

export default function PasswordResetSuccessPage() {
  return (
    <ConfigProvider theme={{ algorithm: antdTheme.defaultAlgorithm }}>
      <div className="flex flex-col lg:flex-row min-h-screen w-full bg-white text-gray-800">
        {/* Left: Card Panel */}
        <div className="flex w-full lg:w-2/5 items-center justify-center px-4 sm:px-6 md:px-10 py-12">
          <Card className="w-full max-w-md sm:max-w-lg rounded-2xl shadow-none lg:shadow-2xl bg-white p-6 sm:p-8 max-h-[90vh] overflow-auto">
            <div className="flex justify-center mb-6">
              <Logo size="md" showText={false} variant="light" shadow={true} />
            </div>

            <div className="text-center">
              <Title level={2} className="text-2xl sm:text-3xl mb-4">
                Password Reset Successful
              </Title>
              <Text className="block mb-6 text-sm sm:text-base text-gray-700">
                Your password has been reset. You can now log in with your new credentials.
              </Text>
              <Link to="/login">
                <Button type="primary" size="large" block>
                  Return to Login
                </Button>
              </Link>
            </div>
          </Card>
        </div>

        {/* Right: Visual Side Panel */}
        <div className="hidden lg:flex w-3/5 relative items-center justify-center bg-gradient-to-br from-blue-600 to-indigo-700">
          <img
            src="/bg_sidepanel.png"
            alt="Side panel background"
            className="absolute inset-0 w-full h-full object-cover opacity-80"
          />
          <div className="absolute inset-0 bg-[#1677FF] bg-opacity-60 z-10" />
          <div className="absolute inset-0 bg-black bg-opacity-30 z-20" />
          <div className="relative z-30 px-10 py-12 text-center text-white max-w-xl">
            <Title level={2} className="!text-white !mb-6 !text-3xl leading-snug">
              Reset Complete
            </Title>
            <Text className="text-lg text-white opacity-90 leading-relaxed">
              Your credentials have been secured — welcome back to productivity.
            </Text>
          </div>
        </div>
      </div>
    </ConfigProvider>
  );
}
