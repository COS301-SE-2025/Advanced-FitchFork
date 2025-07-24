import { Button, Typography, Card } from 'antd';
import { Link } from 'react-router-dom';
import Logo from '@/components/Logo';

const { Title, Text } = Typography;

export default function PasswordResetSuccessPage() {
  return (
    <Card className="w-full max-w-md sm:max-w-lg rounded-2xl shadow-xl bg-white p-6 sm:p-8 max-h-[90vh] overflow-auto">
      <div className="flex justify-center mb-6">
        <Logo size="md" showText={false} shadow />
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
  );
}
