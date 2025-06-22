import { LockOutlined, DesktopOutlined } from '@ant-design/icons';
import { Button, Divider, Input, Switch, Tag } from 'antd';
import SettingsGroup from '@/components/SettingsGroup';
import PageHeader from '@/components/PageHeader';
import useNotImplemented from '@/hooks/useNotImplemented';

const Security = () => {
  const notImplemented = useNotImplemented();

  return (
    <div className="bg-gray-50 dark:bg-gray-950 w-full max-w-6xl p-4 sm:p-6 space-y-12">
      <PageHeader
        title="Security Settings"
        description="Manage your password, two-factor authentication, and sessions."
      />

      <SettingsGroup
        title="Change Password"
        description="Update your password to keep your account secure."
      >
        <div>
          <label className="block font-medium mb-1">Current Password</label>
          <Input.Password size="large" placeholder="••••••••" prefix={<LockOutlined />} />
        </div>

        <div>
          <label className="block font-medium mb-1">New Password</label>
          <Input.Password size="large" placeholder="••••••••" prefix={<LockOutlined />} />
        </div>

        <div>
          <label className="block font-medium mb-1">Confirm New Password</label>
          <Input.Password size="large" placeholder="••••••••" prefix={<LockOutlined />} />
        </div>

        <div className="flex justify-end">
          <Button type="primary" onClick={notImplemented}>
            Update Password
          </Button>
        </div>
      </SettingsGroup>

      <Divider />

      <SettingsGroup
        title="Two-Factor Authentication"
        description="Add an extra layer of security to your account."
      >
        <div className="flex items-center justify-between">
          <span className="font-medium">Enable Two-Factor Authentication</span>
          <Switch defaultChecked onChange={notImplemented} />
        </div>
      </SettingsGroup>

      <Divider />

      <SettingsGroup
        title="Active Sessions"
        description="Devices currently signed into your account."
      >
        <div className="space-y-3">
          {[
            {
              browser: 'Chrome',
              os: 'Windows',
              location: 'Pretoria',
              lastActive: '2 hours ago',
              current: true,
            },
            {
              browser: 'Safari',
              os: 'iPhone',
              location: 'Johannesburg',
              lastActive: 'Yesterday',
              current: false,
            },
          ].map((session, index) => (
            <div
              key={index}
              className="flex items-center justify-between p-3 border border-gray-200 dark:border-gray-800 rounded-md bg-white dark:bg-gray-900"
            >
              <div className="flex items-center gap-3 text-sm">
                <DesktopOutlined className="text-lg text-gray-500" />
                <div className="space-y-0.5">
                  <div className="font-medium">
                    {session.browser} on {session.os}
                  </div>
                  <div className="text-gray-500 dark:text-gray-400 text-xs">
                    {session.location} • Last active {session.lastActive}
                  </div>
                </div>
              </div>

              <div className="flex items-center gap-2">
                {session.current ? (
                  <Tag color="green" className="text-xs">
                    This device
                  </Tag>
                ) : (
                  <Button type="link" danger size="small" onClick={notImplemented}>
                    Logout
                  </Button>
                )}
              </div>
            </div>
          ))}
        </div>

        <div className="flex justify-end mt-4">
          <Button danger size="middle" onClick={notImplemented}>
            Log Out of All Sessions
          </Button>
        </div>
      </SettingsGroup>
    </div>
  );
};

export default Security;
