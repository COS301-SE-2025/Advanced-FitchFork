import { Tag } from 'antd';

export interface IgnoredTagProps {
  ignored: boolean;
}

/**
 * Renders a tag that says "Yes" if ignored, "No" otherwise.
 */
const IgnoredTag = ({ ignored }: IgnoredTagProps) => {
  return ignored ? <Tag color="default">Yes</Tag> : <Tag color="green">No</Tag>;
};

export default IgnoredTag;
