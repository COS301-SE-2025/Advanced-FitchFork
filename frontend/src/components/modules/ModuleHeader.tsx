import { Typography, Tag, Space, Divider } from 'antd';
import type { Module } from '@/types/modules';

const { Title, Text, Paragraph } = Typography;

interface ModuleHeaderProps {
  module: Module;
}

const ModuleHeader = ({ module }: ModuleHeaderProps) => {
  return (
    <div className="bg-white dark:bg-gray-900 rounded-lg border border-gray-200 dark:border-gray-800 p-4 sm:p-6">
      {/* Top: Code + Name */}
      <Title level={3} className="!mb-1">
        {module.code} · {module.description}
      </Title>

      {/* Meta Info */}
      <Space size="middle" className="flex-wrap text-sm text-gray-600">
        <Text type="secondary">Academic Year: {module.year}</Text>
        <Tag color="blue">Semester 2</Tag>
        <Tag color="geekblue">{module.credits} Credits</Tag>
      </Space>

      <Divider className="!my-4" />

      {/* Extra Description */}
      <Paragraph className="!mb-0 text-gray-700">
        This module is worth {module.credits} credits and forms part of your curriculum for the
        selected academic year.
      </Paragraph>
    </div>
  );
};

export default ModuleHeader;
