import {
  MailOutlined,
  PhoneOutlined,
  UserOutlined,
  CalendarOutlined,
  PlusOutlined,
} from '@ant-design/icons';
import { useEffect } from 'react';
import { Button, Divider, Input, message, Upload } from 'antd';
import type { UploadProps } from 'antd';
import PageHeader from '@/components/PageHeader';
import SettingsGroup from '@/components/SettingsGroup';
import { useAuth } from '@/context/AuthContext';
import useNotImplemented from '@/hooks/useNotImplemented';
import { uploadProfilePicture } from '@/services/auth';
import { API_BASE_URL } from '@/config/api';
import UserAvatar from '@/components/common/UserAvatar';

const Account = () => {
  const notImplemented = useNotImplemented();
  const { user, setProfilePictureUrl } = useAuth();

  useEffect(() => {
    if (user?.id) {
      setProfilePictureUrl(`${API_BASE_URL}/auth/avatar/${user.id}?bust=${Date.now()}`);
    }
  }, [user?.id]);

  const handleUpload: UploadProps['customRequest'] = async ({ file, onSuccess, onError }) => {
    const form = new FormData();
    form.append('file', file as File);

    try {
      const res = await uploadProfilePicture(form);
      if (res.success && user?.id) {
        const bust = Date.now();
        const newUrl = `${API_BASE_URL}/auth/avatar/${user.id}?bust=${bust}`;
        setProfilePictureUrl(newUrl);
        message.success('Profile picture updated!');
        onSuccess?.({}, new XMLHttpRequest());
      } else {
        message.error(res.message || 'Upload failed');
        onError?.(new Error(res.message || 'Upload failed'));
      }
    } catch (err: any) {
      message.error('Upload failed.');
      onError?.(err instanceof Error ? err : new Error('Unknown error'));
    }
  };

  const formattedCreatedAt = user?.created_at
    ? new Intl.DateTimeFormat('en-ZA', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
      }).format(new Date(user.created_at))
    : '';

  return (
    <div className="bg-gray-50 dark:bg-gray-950 h-full flex flex-col overflow-hidden">
      <div className="flex-1 overflow-y-auto p-4">
        <div className="space-y-12  max-w-6xl">
          <PageHeader
            title="Account Settings"
            description="Manage your profile, contact details, and account info."
          />

          <SettingsGroup
            title="Profile"
            description="Update your profile picture and full name that others will see."
          >
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
                      user={{ id: user?.id ?? -1, username: user?.username ?? 'User' }}
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

            <div>
              <label className="block font-medium mb-1">Full Name</label>
              <Input size="large" defaultValue={'Jane Doe'} prefix={<UserOutlined />} />
            </div>

            <div className="flex justify-end">
              <Button type="primary" onClick={notImplemented}>
                Save Name
              </Button>
            </div>
          </SettingsGroup>

          <Divider />

          <SettingsGroup
            title="Contact Information"
            description="Used for account recovery and communication."
          >
            <div>
              <label className="block font-medium mb-1">Email Address</label>
              <Input
                size="large"
                defaultValue={user?.email || 'jane.doe@example.com'}
                prefix={<MailOutlined />}
              />
            </div>

            <div>
              <label className="block font-medium mb-1">Phone Number</label>
              <Input size="large" defaultValue={'+27 61 123 4567'} prefix={<PhoneOutlined />} />
            </div>

            <div className="flex justify-end">
              <Button type="primary" onClick={notImplemented}>
                Save Contact Info
              </Button>
            </div>
          </SettingsGroup>

          <Divider />

          <SettingsGroup
            title="Account Details"
            description="Some information is unique to your account and cannot be changed."
          >
            <div>
              <label className="block font-medium mb-1">Username</label>
              <Input size="large" defaultValue={'janedoe123'} prefix={<UserOutlined />} />
            </div>

            <div className="flex justify-end">
              <Button type="primary" onClick={notImplemented}>
                Update Username
              </Button>
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
