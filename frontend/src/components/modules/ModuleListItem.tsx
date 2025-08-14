import { List, Avatar, Typography, Tooltip } from 'antd';
import { BookOutlined, StarFilled, StarOutlined } from '@ant-design/icons';
import { useAuth } from '@/context/AuthContext';
import type { Module } from '@/types/modules';
import React from 'react';
import { formatModuleCode } from '@/utils/modules';
import ModuleRoleTag from './ModuleRoleTag';
import ModuleYearTag from './ModuleYearTag';

const { Paragraph } = Typography;

interface Props {
  module: Module;
  isFavorite: boolean;
  onToggleFavorite: (moduleId: number) => void;
  showFavorite?: boolean;
  onClick?: (m: Module) => void;
  actions?: React.ReactNode[];
}

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
            <span className="text-black dark:text-white font-medium">
              {formatModuleCode(module.code)}
            </span>

            <div className="ml-auto flex items-center gap-1">
              {role && <ModuleRoleTag role={role} />}
              <ModuleYearTag year={module.year} />

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
