import { Typography } from 'antd';

interface ModuleCodeProps {
  code: string; // e.g., "COS326", "BIO1010"
}

const ModuleCode: React.FC<ModuleCodeProps> = ({ code }) => {
  // Match one or more letters, followed by one or more digits
  const match = code.match(/^([A-Za-z]+)(\d+)$/);

  const formatted = match ? `${match[1].toUpperCase()} ${match[2]}` : code; // Fallback if pattern doesn't match

  return <Typography.Text>{formatted}</Typography.Text>;
};

export default ModuleCode;
