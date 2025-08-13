import { List, Avatar, Tag, Typography, Tooltip } from 'antd';
import { BookOutlined, StarFilled, StarOutlined } from '@ant-design/icons';
import { useAuth } from '@/context/AuthContext';
import type { Module } from '@/types/modules';
import React from 'react';

const { Paragraph } = Typography;

interface Props {
  module: Module;
  isFavorite: boolean;
  onToggleFavorite: (moduleId: number) => void;
  showFavorite?: boolean;
  onClick?: (m: Module) => void;
  actions?: React.ReactNode[]; // if you want per-entity actions rendered externally
}

const roleColorMap = {
  student: 'green',
  tutor: 'orange',
  lecturer: 'purple',
  assistant_lecturer: 'pink',
} as const;

const roleLabelMap = {
  student: 'Enrolled',
  tutor: 'Tutoring',
  lecturer: 'Lecturing',
  assistant_lecturer: 'Assistant',
} as const;

const ModuleListItem: React.FC<Props> = ({
  module,
  isFavorite,
  onToggleFavorite,
  showFavorite = true,
  onClick,
}) => {
  const { getModuleRole } = useAuth();
  const role = getModuleRole(module.id);

  const handleRowClick = () => onClick?.(module);
  const handleStarClick: React.MouseEventHandler<HTMLSpanElement> = (e) => {
    e.stopPropagation();
    onToggleFavorite(module.id);
  };

  return (
    <List.Item
      key={module.id}
      className="dark:bg-gray-900 hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer transition"
      onClick={handleRowClick}
      data-cy={`entity-${module.id}`}
    >
      <List.Item.Meta
        avatar={<Avatar icon={<BookOutlined />} style={{ backgroundColor: '#1890ff' }} />}
        title={
          <div className="flex items-center gap-2">
            <span className="text-black dark:text-white font-medium">{module.code}</span>

            <div className="ml-auto flex items-center gap-1">
              {role && <Tag color={roleColorMap[role]}>{roleLabelMap[role]}</Tag>}
              <Tag color="blue">{module.year}</Tag>

              {showFavorite && (
                <Tooltip title={isFavorite ? 'Unfavorite' : 'Favorite'}>
                  <span
                    onClick={handleStarClick}
                    className="ml-1 text-yellow-400 text-lg leading-none flex items-center"
                    role="button"
                    aria-label={isFavorite ? 'Unfavorite module' : 'Favorite module'}
                  >
                    {isFavorite ? <StarFilled /> : <StarOutlined />}
                  </span>
                </Tooltip>
              )}
            </div>
          </div>
        }
        description={
          <Paragraph ellipsis={{ rows: 2 }} className="!mb-0 text-gray-700 dark:text-neutral-300">
            {module.description || 'No description available.'}
          </Paragraph>
        }
      />
    </List.Item>
  );
};

export default ModuleListItem;
