import { LockOutlined, DesktopOutlined } from '@ant-design/icons';
import { Button, Divider, Input, Switch, Tag } from 'antd';
import SettingsGroup from '@/components/SettingsGroup';
import PageHeader from '@/components/PageHeader';
import useNotImplemented from '@/hooks/useNotImplemented';
import { useState } from 'react';
import { message } from '@/utils/message';
import { changePassword } from '@/services/auth';

const Security = () => {
  const notImplemented = useNotImplemented();

  const [current, setCurrent] = useState('');
  const [next, setNext] = useState('');
  const [confirm, setConfirm] = useState('');
  const [loading, setLoading] = useState(false);

  const validate = () => {
    if (!current || !next || !confirm) {
      message.warning('Please fill in all password fields.');
      return false;
    }
    if (next !== confirm) {
      message.error('New password and confirmation do not match.');
      return false;
    }
    if (current === next) {
      message.error('New password must be different from the current password.');
      return false;
    }
    if (next.length < 8) {
      message.error('New password must be at least 8 characters.');
      return false;
    }
    return true;
  };

  const handleChangePassword = async () => {
    if (!validate()) return;
    setLoading(true);
    try {
      const res = await changePassword(current, next);

      if (res?.success === false) {
        message.error(res?.message || 'Failed to update password.');
        return;
      }

      message.success(res?.message || 'Password updated successfully.');
      setCurrent('');
      setNext('');
      setConfirm('');
    } catch (err: any) {
      const apiMsg =
        err?.response?.data?.message ||
        err?.message ||
        'Failed to update password. Please try again.';
      message.error(apiMsg);
    } finally {
      setLoading(false);
    }
  };

  const canSubmit =
    !loading &&
    current.length > 0 &&
    next.length >= 8 &&
    confirm.length > 0 &&
    next === confirm &&
    current !== next;

  return (
    <div className="bg-gray-50 dark:bg-gray-950 h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto p-4">
        <div className="space-y-12  max-w-6xl">
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
              <Input.Password
                size="large"
                placeholder="••••••••"
                prefix={<LockOutlined />}
                value={current}
                onChange={(e) => setCurrent(e.target.value)}
                onPressEnter={handleChangePassword}
                autoComplete="current-password"
              />
            </div>

            <div>
              <label className="block font-medium mb-1">New Password</label>
              <Input.Password
                size="large"
                placeholder="••••••••"
                prefix={<LockOutlined />}
                value={next}
                onChange={(e) => setNext(e.target.value)}
                onPressEnter={handleChangePassword}
                autoComplete="new-password"
              />
            </div>

            <div>
              <label className="block font-medium mb-1">Confirm New Password</label>
              <Input.Password
                size="large"
                placeholder="••••••••"
                prefix={<LockOutlined />}
                value={confirm}
                onChange={(e) => setConfirm(e.target.value)}
                onPressEnter={handleChangePassword}
                autoComplete="new-password"
              />
            </div>

            <div className="flex justify-end">
              <Button
                type="primary"
                onClick={handleChangePassword}
                loading={loading}
                disabled={!canSubmit}
              >
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
      </div>
    </div>
  );
};

export default Security;
