import { Result, Button } from 'antd';
import { Link } from 'react-router-dom';
import Logo from '@components/Logo';

export default function NotFound() {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-gray-50 px-4 py-12">
      <div className="mb-8">
        <Logo size="md" />
      </div>
      <Result
        status="404"
        title="Page Not Found"
        subTitle="The page you’re looking for doesn’t exist or has been moved."
        extra={
          <Link to="/dashboard">
            <Button type="primary" size="large">
              Back to Dashboard
            </Button>
          </Link>
        }
        className="text-center"
      />
    </div>
  );
}
