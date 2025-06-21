import { Typography } from 'antd';

const { Title, Paragraph } = Typography;

interface PageHeaderProps {
  title: React.ReactNode;
  description?: React.ReactNode;
}

const PageHeader = ({ title, description }: PageHeaderProps) => {
  return (
    <div className="mb-6 sm:mb-8">
      <Title className="!text-lg sm:!text-2xl !text-gray-800 dark:!text-gray-100 !leading-tight !mb-1">
        {title}
      </Title>
      {description && (
        <Paragraph className="!text-sm sm:!text-base !text-gray-600 dark:!text-gray-300">
          {description}
        </Paragraph>
      )}
    </div>
  );
};

export default PageHeader;
