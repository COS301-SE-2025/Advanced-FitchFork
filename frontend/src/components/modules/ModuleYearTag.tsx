import React from 'react';
import { Tag } from 'antd';

interface Props {
  year: number;
}

const ModuleYearTag: React.FC<Props> = ({ year }) => {
  // Deterministic hash → color
  const getColorForYear = (year: number) => {
    // Simple hash function
    const hash = Array.from(year.toString()).reduce((acc, char) => acc + char.charCodeAt(0), 0);
    // Ant Design preset colors
    const colors = [
      'magenta',
      'red',
      'volcano',
      'orange',
      'gold',
      'lime',
      'green',
      'cyan',
      'blue',
      'geekblue',
      'purple',
    ];
    return colors[hash % colors.length];
  };

  return <Tag color={getColorForYear(year)}>{year}</Tag>;
};

export default ModuleYearTag;
