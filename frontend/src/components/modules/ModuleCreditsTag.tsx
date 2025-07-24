// src/components/modules/ModuleCreditsTag.tsx

import { Tag } from 'antd';

interface Props {
  credits: number;
}

const ModuleCreditsTag = ({ credits }: Props) => {
  return (
    <Tag color="blue" className="font-medium">
      {credits} credits
    </Tag>
  );
};

export default ModuleCreditsTag;
