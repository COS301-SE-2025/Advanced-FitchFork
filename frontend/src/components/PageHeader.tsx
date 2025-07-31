import { Typography } from 'antd';

const { Title, Paragraph } = Typography;

interface PageHeaderProps {
  title: React.ReactNode;
  description?: React.ReactNode;
  extra?: React.ReactNode; // <-- New
}

const PageHeader = ({ title, description, extra }: PageHeaderProps) => {
  return (
    <div className="mb-6 sm:mb-8 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 sm:gap-6">
      <div>
        <Title className="!text-lg sm:!text-2xl !text-gray-800 dark:!text-gray-100 !leading-tight !mb-1">
          {title}
        </Title>
        {description && (
          <Paragraph className="!text-sm sm:!text-base !text-gray-600 dark:!text-gray-300 !mb-0">
            {description}
          </Paragraph>
        )}
      </div>
      {extra && <div className="shrink-0">{extra}</div>}
    </div>
  );
};

export default PageHeader;
