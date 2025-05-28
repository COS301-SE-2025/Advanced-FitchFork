import { Result, Button, Typography } from 'antd';
import { HomeOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import Logo from '@/components/Logo';

const { Title, Text, Paragraph } = Typography;

export default function UnderConstruction() {
  const navigate = useNavigate();

  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-gray-50 dark:bg-gray-950 px-4 py-12">
      <div className="mb-8">
        <Logo />
      </div>

      <Result
        status="info"
        title={
          <Title level={2} className="!text-gray-800 dark:!text-gray-100">
            Page Under Construction
          </Title>
        }
        subTitle={
          <Text className="!text-gray-600 dark:!text-gray-300">
            We're still working on this page. Please check back later.
          </Text>
        }
        extra={
          <Button type="primary" size="large" icon={<HomeOutlined />} onClick={() => navigate(-1)}>
            Go Back
          </Button>
        }
        className="text-center"
      />

      <Paragraph className="text-center text-sm text-gray-500 dark:text-gray-400 mt-4">
        If you believe this is an error, please contact the development team.
      </Paragraph>
    </div>
  );
}
