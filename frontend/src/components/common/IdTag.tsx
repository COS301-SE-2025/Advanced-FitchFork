import { Tag } from 'antd';

type IdTagProps = {
  id: number | string;
  prefix?: string; // e.g. "#" or "ID-"
};

const IdTag: React.FC<IdTagProps> = ({ id, prefix = '#' }) => {
  const label = `${prefix}${id}`;
  return <Tag color="default">{label}</Tag>;
};

export default IdTag;
