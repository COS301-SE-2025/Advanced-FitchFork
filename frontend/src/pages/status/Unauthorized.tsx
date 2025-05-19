import { Result, Button } from 'antd';
import { Link } from 'react-router-dom';
import Logo from '@components/Logo';

export default function Unauthorized() {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-gray-50 px-4 py-12">
      {/* Centered Logo */}
      <div className="mb-8">
        <Logo size="md" />
      </div>

      {/* 403 Result */}
      <Result
        status="403"
        title="Unauthorized"
        subTitle="You don’t have permission to access this page."
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
