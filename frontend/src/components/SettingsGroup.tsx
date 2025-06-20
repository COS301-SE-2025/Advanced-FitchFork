import { Typography } from 'antd';

const { Title, Paragraph } = Typography;

const SettingsGroup = ({
  title,
  description,
  children,
  actions,
}: {
  title: string;
  description?: string;
  children: React.ReactNode;
  actions?: React.ReactNode;
}) => (
  <section className="flex flex-col sm:flex-row sm:items-start gap-6 sm:gap-12">
    <div className="sm:w-1/3">
      <Title level={5} className="!mb-1">
        {title}
      </Title>
      {description && (
        <Paragraph type="secondary" className="text-sm text-gray-600 dark:text-gray-400">
          {description}
        </Paragraph>
      )}
    </div>
    <div className="flex-1 space-y-4">
      {children}
      {actions && <div className="pt-2 flex justify-end">{actions}</div>}
    </div>
  </section>
);

export default SettingsGroup;
