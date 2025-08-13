import { Tag } from 'antd';
import { PushpinFilled } from '@ant-design/icons';

const PinnedTag = () => {
  return (
    <Tag color="gold" icon={<PushpinFilled />}>
      Pinned
    </Tag>
  );
};

export default PinnedTag;
