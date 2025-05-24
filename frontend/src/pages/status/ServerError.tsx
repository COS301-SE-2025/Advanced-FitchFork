import Logo from '@/components/Logo';
import { Result, Button, Typography } from 'antd';
import { Link } from 'react-router-dom';

const { Title, Text } = Typography;

export default function ServerError() {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-gray-50 dark:bg-gray-950 px-4 py-12">
      <div className="mb-8">
        <Logo />
      </div>
      <Result
        status="500"
        title={
          <Title level={2} className="!text-gray-800 dark:!text-gray-100">
            Server Error
          </Title>
        }
        subTitle={
          <Text className="!text-gray-600 dark:!text-gray-300">
            Oops! Something went wrong on our side.
          </Text>
        }
        extra={
          <Link to="/dashboard">
            <Button type="primary" size="large">
              Try Again
            </Button>
          </Link>
        }
        className="text-center"
      />
    </div>
  );
}
