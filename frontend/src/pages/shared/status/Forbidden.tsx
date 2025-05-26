import Logo from '@/components/Logo';
import { Result, Button, Typography } from 'antd';
import { Link } from 'react-router-dom';

const { Title, Text } = Typography;

export default function Forbidden() {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-gray-50 dark:bg-gray-950 px-4 py-12">
      <div className="mb-8">
        <Logo />
      </div>

      <Result
        status="403"
        title={
          <Title level={2} className="!text-gray-800 dark:!text-gray-100">
            Forbidden
          </Title>
        }
        subTitle={
          <Text className="!text-gray-600 dark:!text-gray-300">
            You need to log in before accessing this page.
          </Text>
        }
        extra={
          <Link to="/login">
            <Button type="primary" size="large">
              Return to Login
            </Button>
          </Link>
        }
        className="text-center"
      />
    </div>
  );
}
