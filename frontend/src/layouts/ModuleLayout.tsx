import { useNavigate, useLocation, Outlet } from 'react-router-dom';
import { Layout, Menu, Typography } from 'antd';
import {
  HomeOutlined,
  FileTextOutlined,
  BarChartOutlined,
  BookOutlined,
  UserOutlined,
  NotificationOutlined,
} from '@ant-design/icons';
import { useMediaQuery } from 'react-responsive';
import { useAuth } from '@/context/AuthContext';
import { useModule } from '@/context/ModuleContext';
import MobilePageHeader from '@/components/common/MobilePageHeader';
import { formatModuleCode } from '@/utils/modules';

const { Sider, Content } = Layout;
const { Title } = Typography;

const ModuleLayout = () => {
  const module = useModule();
  const { id: moduleId, code, year } = module;
  const navigate = useNavigate();
  const location = useLocation();
  const isMobile = useMediaQuery({ maxWidth: 768 });

  const auth = useAuth();
  const showPersonnel = auth.isAdmin || auth.isLecturer(Number(moduleId));

  const moduleMenu = [
    { key: `/modules/${moduleId}`, icon: <HomeOutlined />, label: 'Overview' },
    {
      key: `/modules/${moduleId}/announcements`,
      icon: <NotificationOutlined />,
      label: 'Announcements',
    },
    { key: `/modules/${moduleId}/assignments`, icon: <FileTextOutlined />, label: 'Assignments' },
    { key: `/modules/${moduleId}/grades`, icon: <BarChartOutlined />, label: 'Grades' },
    { key: `/modules/${moduleId}/resources`, icon: <BookOutlined />, label: 'Resources' },
    ...(showPersonnel
      ? [{ key: `/modules/${moduleId}/personnel`, icon: <UserOutlined />, label: 'Personnel' }]
      : []),
  ];

  const currentKey =
    moduleMenu
      .map((item) => item.key)
      .filter((key) => location.pathname === key || location.pathname.startsWith(`${key}/`))
      .sort((a, b) => b.length - a.length)[0] ?? '';

  const handleNav = ({ key }: { key: string }) => {
    if (location.pathname !== key) navigate(key);
  };

  return (
    <Layout className="flex h-full !bg-transparent">
      {!isMobile && (
        <Sider
          width={240}
          className="!bg-white dark:!bg-gray-900 border-r border-gray-200 dark:border-gray-800"
        >
          <div className="flex items-center gap-2 px-4 py-5 border-b border-gray-200 dark:border-gray-800">
            <Title level={5} className="!mb-0">
              {formatModuleCode(code) + ' '}
              <span className="text-gray-400 dark:text-gray-500">{year}</span>
            </Title>
          </div>
          <Menu
            mode="inline"
            selectedKeys={[currentKey]}
            items={moduleMenu}
            onClick={handleNav}
            className="!bg-transparent !border-none pt-4"
          />
        </Sider>
      )}

      <Layout className="flex-1 flex flex-col min-h-0 !bg-transparent">
        {isMobile && <MobilePageHeader />}

        <Content className="flex-1 min-h-0 flex flex-col !bg-transparent">
          <Outlet />
        </Content>
      </Layout>
    </Layout>
  );
};

export default ModuleLayout;
