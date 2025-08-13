import { useAuth } from '@/context/AuthContext';
import { List, Typography, Empty, Tooltip } from 'antd';
import { AppstoreOutlined, InfoCircleOutlined } from '@ant-design/icons';
import dayjs from 'dayjs';
import ModuleCode from '@/components/modules/ModuleCode';
import ModuleYearTag from './ModuleYearTag';
import { useUI } from '@/context/UIContext';
import { useNavigate } from 'react-router-dom';
import { ModuleRoleTag } from '.';
import type { ModuleRole } from '@/types/modules';

const { Title, Text } = Typography;

interface Props {
  // modules: Module;
  scope?: ModuleRole;
}

const truncateTooltip = (text: string, maxLength = 200) =>
  text.length > maxLength ? `${text.slice(0, maxLength)}…` : text;

// Role → action label for the title
const ROLE_TITLE_LABEL: Record<ModuleRole, string> = {
  lecturer: 'Lecturing',
  assistant_lecturer: 'Assistant Lecturing',
  tutor: 'Tutoring',
  student: 'Studying',
};

const roleToTitleLabel = (role?: ModuleRole) =>
  role ? (ROLE_TITLE_LABEL[role] ?? role) : 'Current';

const MyModules: React.FC<Props> = ({ scope }) => {
  const { modules } = useAuth();
  const { isSm } = useUI();
  const navigate = useNavigate();

  const yearNow = dayjs().year();
  const filtered = modules
    .filter((m) => m.year === yearNow)
    .filter((m) => (scope ? m.role === scope : Boolean(m.role)));

  return (
    <div className="h-full min-h-0 flex flex-col w-full bg-white dark:bg-gray-900 rounded-md border border-gray-200 dark:border-gray-800">
      {/* Header */}
      <div className="px-3 py-2 border-b border-gray-200 dark:border-gray-800">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2">
            <AppstoreOutlined className="text-gray-500" />
            <Title level={isSm ? 5 : 5} className="!mb-0">
              {`My ${roleToTitleLabel(scope)} Modules`}
            </Title>
          </div>
          <Text type="secondary" className="!text-xs">
            {yearNow}
          </Text>
        </div>
      </div>

      {/* List */}
      <List
        className="flex-1 overflow-y-auto"
        locale={{
          emptyText: (
            <Empty
              image={Empty.PRESENTED_IMAGE_SIMPLE}
              description={
                scope ? `No ${ROLE_TITLE_LABEL[scope]} modules found` : 'No modules found'
              }
            />
          ),
        }}
        dataSource={filtered}
        renderItem={(mod) => (
          <List.Item
            key={mod.id}
            className="!px-3 cursor-pointer"
            onClick={() => navigate(`/modules/${mod.id}`)}
          >
            <div className="flex items-center justify-between gap-2 w-full">
              {/* Left: module code + info icon */}
              <div className="flex items-center gap-2 min-w-0">
                <ModuleCode code={mod.code} />
                {mod.description && (
                  <Tooltip title={truncateTooltip(mod.description, 200)} placement="top">
                    <InfoCircleOutlined className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300" />
                  </Tooltip>
                )}
              </div>

              {/* Right: year + role */}
              <div className="flex items-center gap-2 shrink-0">
                <ModuleYearTag year={mod.year} />
                {mod.role && <ModuleRoleTag role={mod.role} bordered asAction />}
              </div>
            </div>
          </List.Item>
        )}
      />
    </div>
  );
};

export default MyModules;
