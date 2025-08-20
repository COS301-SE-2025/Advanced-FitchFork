import { Button, Typography, Card } from 'antd';
import { Link } from 'react-router-dom';
import Logo from '@/components/common/Logo';

const { Title, Text } = Typography;

export default function PasswordResetSuccessPage() {
  return (
    <Card
      title={
        <div className="flex items-center gap-2 justify-start">
          <Logo size="md" showText={false} />
          <Title level={4} className="!m-0">
            Password Reset
          </Title>
        </div>
      }
      className="w-full max-w-md sm:max-w-lg rounded-2xl shadow-md hover:shadow-lg transition-shadow dark:bg-neutral-900 dark:border-neutral-800"
    >
      <div className="text-center mb-6">
        <Title level={2} className="!mb-2 text-2xl sm:text-3xl">
          Success
        </Title>
        <Text className="block text-sm sm:text-base text-gray-600">
          Your password has been reset. You can now log in with your new credentials.
        </Text>
      </div>

      <Link to="/login">
        <Button type="primary" size="large" block>
          Return to Login
        </Button>
      </Link>
    </Card>
  );
}
