import { Typography } from 'antd';

const { Title, Paragraph } = Typography;

const StepFinal = () => {
  return (
    <div className="text-center space-y-4">
      <Title level={3}>Setup Complete</Title>
      <Paragraph>Your assignment has been successfully configured and is ready for use.</Paragraph>
    </div>
  );
};

export default StepFinal;
