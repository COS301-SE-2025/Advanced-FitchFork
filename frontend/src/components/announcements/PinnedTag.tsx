import { Tag } from 'antd';
import { PushpinFilled } from '@ant-design/icons';

interface Props {
  pinned?: boolean; // Defaults to true
}

const PinnedTag = ({ pinned = true }: Props) => {
  if (pinned) {
    return (
      <Tag color="gold" icon={<PushpinFilled />}>
        Pinned
      </Tag>
    );
  }

  return <Tag color="default">Not pinned</Tag>;
};

export default PinnedTag;
