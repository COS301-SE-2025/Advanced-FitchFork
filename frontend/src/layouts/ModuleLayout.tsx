import { useEffect, useState } from 'react';
import { useParams, useNavigate, useLocation, Outlet } from 'react-router-dom';
import { Layout, Menu, Typography, Spin, Tabs } from 'antd';
import {
  HomeOutlined,
  FileTextOutlined,
  BarChartOutlined,
  BookOutlined,
  UserOutlined,
} from '@ant-design/icons';
import { useMediaQuery } from 'react-responsive';
import { useAuth } from '@/context/AuthContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { ModuleProvider } from '@/context/ModuleContext';
import { getModuleDetails } from '@/services/modules';
import type { Module } from '@/types/modules';
import type { User } from '@/types/users';

const { Sider, Content } = Layout;
const { Title } = Typography;

interface ModuleDetails extends Module {
  lecturers: User[];
  tutors: User[];
  students: User[];
}

const ModuleLayout = () => {
  const { id } = useParams();
  const navigate = useNavigate();
  const location = useLocation();
  const moduleId = Number(id);
  const { setBreadcrumbLabel } = useBreadcrumbContext();

  const { isAdmin } = useAuth();
  const showPersonnel = isAdmin;

  const [loading, setLoading] = useState(true);
  const [module, setModule] = useState<ModuleDetails | null>(null);

  const isMobile = useMediaQuery({ maxWidth: 768 });

  useEffect(() => {
    const load = async () => {
      const res = await getModuleDetails(moduleId);
      if (res.success && res.data) {
        setModule(res.data);
        setBreadcrumbLabel(`modules/${res.data.id}`, res.data.code);
      }
      setLoading(false);
    };

    if (!isNaN(moduleId)) load();
  }, [moduleId]);

  const moduleMenu = [
    { key: `/modules/${moduleId}`, icon: <HomeOutlined />, label: 'Overview' },
    { key: `/modules/${moduleId}/assignments`, icon: <FileTextOutlined />, label: 'Assignments' },
    { key: `/modules/${moduleId}/grades`, icon: <BarChartOutlined />, label: 'Grades' },
    { key: `/modules/${moduleId}/resources`, icon: <BookOutlined />, label: 'Resources' },
    ...(showPersonnel
      ? [{ key: `/modules/${moduleId}/personnel`, icon: <UserOutlined />, label: 'Personnel' }]
      : []),
  ];

  const handleNav = ({ key }: { key: string }) => {
    if (location.pathname !== key) navigate(key);
  };

  if (loading || !module) {
    return (
      <div className="p-8">
        <Spin tip="Loading module..." />
      </div>
    );
  }

  const currentKey =
    moduleMenu
      .map((item) => item.key)
      .filter((key) => location.pathname === key || location.pathname.startsWith(key + '/'))
      .sort((a, b) => b.length - a.length)[0] ?? '';

  return (
    <Layout className="!bg-white dark:!bg-gray-950 h-full">
      {isMobile ? (
        <div className="w-full px-4 pt-4 bg-white dark:bg-gray-950">
          <Tabs
            activeKey={location.pathname}
            onChange={(key) => navigate(key)}
            items={moduleMenu.map((item) => ({
              key: item.key,
              label: item.label,
              icon: item.icon,
            }))}
            tabBarGutter={16}
            animated={false}
            className="!mb-0"
          />
        </div>
      ) : (
        <Sider
          width={240}
          className="!bg-white dark:!bg-gray-950 border-r border-gray-200 dark:border-gray-800"
        >
          <div className="flex flex-row justify-start items-center gap-2 px-4 py-5 border-b border-gray-200 dark:border-gray-800">
            <Title level={5} className="!mb-0">
              {module.code} <span className="text-gray-400 dark:text-gray-500">{module.year}</span>
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

      <ModuleProvider value={{ module }}>
        <Layout className="!bg-transparent">
          <Content className="overflow-y-auto min-h-full">
            <Outlet />
          </Content>
        </Layout>
      </ModuleProvider>
    </Layout>
  );
};

export default ModuleLayout;
