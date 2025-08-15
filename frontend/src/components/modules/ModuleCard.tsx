import { Card, Avatar, Typography, Tooltip } from 'antd';
import { BookOutlined, StarFilled, StarOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '@/context/AuthContext';
import type { Module } from '@/types/modules';
import { formatModuleCode } from '@/utils/modules';
import ModuleYearTag from './ModuleYearTag';
import ModuleRoleTag from './ModuleRoleTag';

const { Meta } = Card;
const { Paragraph } = Typography;

interface Props {
  module: Module;
  isFavorite: boolean;
  onToggleFavorite: (moduleId: number) => void;
  actions?: React.ReactNode[];
  showFavorite?: boolean;
}

const ModuleCard = ({
  module,
  isFavorite,
  onToggleFavorite,
  actions,
  showFavorite = true,
}: Props) => {
  const navigate = useNavigate();
  const { getModuleRole } = useAuth();
  const role = getModuleRole(module.id);

  const handleClick = () => {
    navigate(`/modules/${module.id}`);
  };

  const handleStarClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    onToggleFavorite(module.id);
  };

  return (
    <Card
      hoverable
      onClick={handleClick}
      className="w-full cursor-pointer !bg-white dark:!bg-gray-900 "
      cover={
        <div className="h-[140px] !flex items-center justify-center bg-gray-100 dark:bg-neutral-700 relative">
          <BookOutlined className="text-5xl !text-gray-400 dark:!text-neutral-400" />
          {showFavorite && (
            <Tooltip title={isFavorite ? 'Unfavorite' : 'Favorite'}>
              <div
                onClick={handleStarClick}
                className="absolute top-2 right-2 text-xl text-yellow-400"
              >
                {isFavorite ? <StarFilled /> : <StarOutlined />}
              </div>
            </Tooltip>
          )}
        </div>
      }
      actions={actions}
      data-testid="entity-card"
    >
      <Meta
        avatar={<Avatar icon={<BookOutlined />} style={{ backgroundColor: '#1890ff' }} />}
        title={
          <div className="flex justify-between items-center">
            <span className="text-black dark:text-white">{formatModuleCode(module.code)}</span>
            <div className="flex gap-1">
              {role && <ModuleRoleTag role={role} />}
              <ModuleYearTag year={module.year} />
            </div>
          </div>
        }
        description={
          <Paragraph ellipsis={{ rows: 2 }} className="mb-0 text-gray-700 dark:text-neutral-300">
            {module.description || 'No description available.'}
          </Paragraph>
        }
      />
    </Card>
  );
};

export default ModuleCard;
