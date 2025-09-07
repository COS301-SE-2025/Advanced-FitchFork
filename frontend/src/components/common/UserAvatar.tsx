import { Avatar } from 'antd';
import { useEffect, useState } from 'react';
import { API_BASE_URL } from '@/config/api';
import { useTheme } from '@/context/ThemeContext';
import type { User } from '@/types/users';
import { getAvatarColors } from '@/utils/color';

export interface UserAvatarProps {
  user: Pick<User, 'id' | 'username'>;
  size?: 'small' | 'default' | 'large' | number;
  className?: string;
}

function getScaledFontSize(size: UserAvatarProps['size']): string {
  if (typeof size === 'number') return `${Math.floor(size * 0.5)}px`;
  switch (size) {
    case 'small':
      return '14px';
    case 'large':
      return '22px';
    case 'default':
    default:
      return '18px';
  }
}

const UserAvatar = ({ user, size = 'large', className = '' }: UserAvatarProps) => {
  const { isDarkMode } = useTheme();
  const [avatarUrl, setAvatarUrl] = useState<string | null>(null);

  useEffect(() => {
    if (!user?.id) return;
    const url = `${API_BASE_URL}/users/${user.id}/avatar`;
    fetch(url, { method: 'HEAD' })
      .then((res) => setAvatarUrl(res.ok ? url : null))
      .catch(() => setAvatarUrl(null));
  }, [user.id]);

  const { background, text } = getAvatarColors(user.username, isDarkMode);
  const fallbackLetter = user.username.charAt(0).toUpperCase();

  const fallbackStyle = avatarUrl
    ? {}
    : {
        backgroundColor: background,
        color: text,
        fontSize: getScaledFontSize(size),
        fontWeight: 500,
      };

  return (
    <Avatar
      size={size}
      src={avatarUrl || undefined}
      className={className}
      style={fallbackStyle}
      alt={user.username}
    >
      {fallbackLetter}
    </Avatar>
  );
};

export default UserAvatar;
