import { Result, Button } from 'antd';
import { Link } from 'react-router-dom';
import Logo from '@components/Logo';

export default function ServerError() {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-gray-50 px-4 py-12">
      <div className="mb-8">
        <Logo size="md" />
      </div>
      <Result
        status="500"
        title="Server Error"
        subTitle="Oops! Something went wrong on our side."
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
