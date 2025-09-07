import { Tag } from 'antd';

interface Props {
  score: number; // 0–100
}

const ScoreTag = ({ score }: Props) => {
  const getColor = (mark: number): string => {
    if (mark >= 75) return 'green';
    if (mark >= 50) return 'orange';
    return 'red';
  };

  return <Tag color={getColor(score)}>{score}%</Tag>;
};

export default ScoreTag;
