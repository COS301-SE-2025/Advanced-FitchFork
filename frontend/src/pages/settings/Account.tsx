import { MailOutlined, UserOutlined, CalendarOutlined, PlusOutlined } from '@ant-design/icons';
import { useEffect, useMemo } from 'react';
import { Divider, Input, message, Upload } from 'antd';
import type { UploadProps } from 'antd';
import PageHeader from '@/components/PageHeader';
import SettingsGroup from '@/components/SettingsGroup';
import { useAuth } from '@/context/AuthContext';
import { uploadProfilePicture } from '@/services/auth';
import { API_BASE_URL } from '@/config/api';
import UserAvatar from '@/components/common/UserAvatar';

const Account = () => {
  const { user, setProfilePictureUrl } = useAuth();

  useEffect(() => {
    if (user?.id) {
      setProfilePictureUrl(`${API_BASE_URL}/auth/avatar/${user.id}?bust=${Date.now()}`);
    }
  }, [user?.id, setProfilePictureUrl]);

  const handleUpload: UploadProps['customRequest'] = async ({ file, onSuccess, onError }) => {
    const form = new FormData();
    form.append('file', file as File);

    try {
      const res = await uploadProfilePicture(form);
      if (res.success && user?.id) {
        const newUrl = `${API_BASE_URL}/auth/avatar/${user.id}?bust=${Date.now()}`;
        setProfilePictureUrl(newUrl);
        message.success('Profile picture updated!');
        onSuccess?.({}, new XMLHttpRequest());
      } else {
        const msg = (res as any)?.message || 'Upload failed';
        message.error(msg);
        onError?.(new Error(msg));
      }
    } catch (err: any) {
      message.error('Upload failed.');
      onError?.(err instanceof Error ? err : new Error('Unknown error'));
    }
  };

  const formattedCreatedAt = useMemo(() => {
    if (!user?.created_at) return '';
    try {
      return new Intl.DateTimeFormat('en-ZA', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      }).format(new Date(user.created_at));
    } catch {
      return '';
    }
  }, [user?.created_at]);

  return (
    <div className="bg-gray-50 dark:bg-gray-950 h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto p-4">
        <div className="space-y-12 max-w-4xl">
          <PageHeader
            title="Account Settings"
            description="Manage your profile picture and view account details."
          />

          {/* Profile Picture */}
          <SettingsGroup title="Profile Picture" description="Shown to other users.">
            <div className="relative w-[96px] h-[96px] group">
              <Upload
                name="avatar"
                listType="picture-circle"
                showUploadList={false}
                customRequest={handleUpload}
              >
                <div className="relative cursor-pointer w-[96px] h-[96px] rounded-full bg-gray-200 dark:bg-gray-800 flex items-center justify-center overflow-hidden group">
                  <div className="w-full h-full flex items-center justify-center">
                    <UserAvatar
                      user={{ id: user?.id ?? -1, username: user?.username ?? '' }}
                      size={96}
                      className="transition group-hover:opacity-20"
                    />
                  </div>
                  <div className="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition">
                    <PlusOutlined className="text-white text-xl" />
                  </div>
                </div>
              </Upload>
            </div>
          </SettingsGroup>

          <Divider />

          {/* Email (read-only) */}
          <SettingsGroup title="Email" description="Used for sign-in and account recovery.">
            <div>
              <label className="block font-medium mb-1">Email Address</label>
              <Input
                size="large"
                value={user?.email ?? ''}
                readOnly
                prefix={<MailOutlined />}
                className="bg-gray-100 dark:bg-gray-800"
              />
            </div>
          </SettingsGroup>

          <Divider />

          {/* Read-only account facts */}
          <SettingsGroup
            title="Account Details"
            description="Immutable identifiers and timestamps."
          >
            <div>
              <label className="block font-medium mb-1">Username</label>
              <Input
                size="large"
                value={user?.username ?? ''}
                readOnly
                prefix={<UserOutlined />}
                className="bg-gray-100 dark:bg-gray-800"
              />
            </div>

            <div>
              <label className="block font-medium mb-1 mt-4">Account Created</label>
              <Input
                size="large"
                value={formattedCreatedAt}
                readOnly
                prefix={<CalendarOutlined />}
                className="bg-gray-100 dark:bg-gray-800"
              />
            </div>
          </SettingsGroup>
        </div>
      </div>
    </div>
  );
};

export default Account;
