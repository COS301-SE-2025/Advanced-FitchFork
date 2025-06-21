import { Card, Avatar, Tag, Typography, Tooltip } from 'antd';
import { BookOutlined, StarFilled, StarOutlined } from '@ant-design/icons';
import { useNavigate } from 'react-router-dom';
import type { UserModuleRole } from '@/types/modules';

const { Meta } = Card;
const { Paragraph } = Typography;

interface Props {
  module: UserModuleRole;
  isFavorite: boolean;
  onToggleFavorite: (moduleId: number) => void;
}

const roleColorMap: Record<UserModuleRole['role'], string> = {
  Student: 'green',
  Tutor: 'orange',
  Lecturer: 'purple',
};

const roleLabelMap: Record<UserModuleRole['role'], string> = {
  Student: 'Enrolled',
  Tutor: 'Tutoring',
  Lecturer: 'Lecturing',
};

const ModuleCard = ({ module, isFavorite, onToggleFavorite }: Props) => {
  const navigate = useNavigate();

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
      className="w-full cursor-pointer dark:bg-neutral-800 dark:border-neutral-700"
      cover={
        <div className="h-[140px] !flex items-center justify-center bg-gray-100 dark:bg-neutral-700 relative">
          <BookOutlined className="text-5xl !text-gray-400 dark:!text-neutral-400" />
          <Tooltip title={isFavorite ? 'Unfavorite' : 'Favorite'}>
            <div
              onClick={handleStarClick}
              className="absolute top-2 right-2 text-xl text-yellow-400"
            >
              {isFavorite ? <StarFilled /> : <StarOutlined />}
            </div>
          </Tooltip>
        </div>
      }
    >
      <Meta
        avatar={<Avatar icon={<BookOutlined />} style={{ backgroundColor: '#1890ff' }} />}
        title={
          <div className="flex justify-between items-center">
            <span className="text-black dark:text-white">{module.code}</span>
            <div className="flex gap-1">
              <Tag color={roleColorMap[module.role]}>{roleLabelMap[module.role]}</Tag>
              <Tag color="blue">{module.year}</Tag>
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
