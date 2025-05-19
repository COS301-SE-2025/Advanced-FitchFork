import { Menu, Button } from 'antd';
import { HomeOutlined, SettingOutlined, LogoutOutlined } from '@ant-design/icons';
import { useNavigate, useLocation } from 'react-router-dom';
import Logo from '@components/Logo';
import { useAuth } from '@context/AuthContext';

const menuItems = [
  {
    key: '/dashboard',
    icon: <HomeOutlined />,
    label: 'Home',
  },
  {
    key: '/settings',
    icon: <SettingOutlined />,
    label: 'Settings',
  },
];

export default function SidebarNav() {
  const navigate = useNavigate();
  const location = useLocation();
  const { logout } = useAuth();

  return (
    <div className="h-full flex flex-col justify-between">
      {/* Top: Logo + Menu */}
      <div>
        <div className="px-6 py-4 border-gray-200 mb-6">
          <Logo size="md" />
        </div>
        <Menu
          mode="inline"
          theme="light"
          selectedKeys={[location.pathname]}
          items={menuItems}
          onClick={({ key }) => navigate(key)}
          className="mt-2"
        />
      </div>

      {/* Bottom: Logout */}
      <div className="px-4 py-4 border-t border-gray-200">
        <Button
          type="text"
          icon={<LogoutOutlined />}
          onClick={() => {
            logout();
            navigate('/login');
          }}
          className="w-full text-left"
        >
          Logout
        </Button>
      </div>
    </div>
  );
}
