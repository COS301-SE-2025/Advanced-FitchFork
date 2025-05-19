import { Layout } from 'antd';
import { useAuth } from '@context/AuthContext';
import { useNavigate } from 'react-router-dom';
import SidebarNav from '@components/SidebarNav';

const { Header, Sider, Content } = Layout;

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  const { logout } = useAuth();
  const navigate = useNavigate();

  return (
    <Layout className="h-screen bg-gray-50">
      <Sider width={240} className="!bg-white" theme="light">
        <SidebarNav />
      </Sider>

      <Layout className="flex flex-col">
        <Header className="!bg-white px-6 flex items-center justify-between h-16">
          <div className="text-gray-800 font-semibold text-lg">Dashboard</div>
        </Header>

        <Content className="flex-1 p-6">
          <div className="bg-white rounded-xl shadow-sm p-6 h-full">{children}</div>
        </Content>
      </Layout>
    </Layout>
  );
}
