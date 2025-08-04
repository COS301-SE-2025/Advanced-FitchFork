import {
  HomeOutlined,
  SettingOutlined,
  LogoutOutlined,
  UserOutlined,
  AppstoreOutlined,
  BarChartOutlined,
  CalendarOutlined,
  QuestionCircleOutlined,
  CommentOutlined,
} from '@ant-design/icons';
import React from 'react';
import { useAuth } from '@/context/AuthContext';

/**
 * Menu item structure.
 */
export interface MenuItem {
  key: string;
  label: string;
  icon?: React.ReactNode;
  adminOnly?: boolean;
  userOnly?: boolean;
  children?: MenuItem[];
}

/**
 * Dynamically generates the top sidebar items based on role and module presence.
 */
export const useTopMenuItems = (): MenuItem[] => {
  const { isAdmin, isUser } = useAuth();

  const items: MenuItem[] = [
    {
      key: '/dashboard',
      icon: React.createElement(HomeOutlined),
      label: 'Dashboard',
    },
    {
      key: '/calendar',
      icon: React.createElement(CalendarOutlined),
      label: 'Calendar',
    },
    {
      key: '/chat',
      icon: React.createElement(CommentOutlined),
      label: 'Chat',
    },
  ];

  if (isAdmin) {
    items.push(
      {
        key: '/users',
        icon: React.createElement(UserOutlined),
        label: 'Users',
        adminOnly: true,
      },
      {
        key: '/modules',
        icon: React.createElement(AppstoreOutlined),
        label: 'Modules',
        adminOnly: true,
      },
      {
        key: '/reports',
        icon: React.createElement(BarChartOutlined),
        label: 'Reports',
        adminOnly: true,
      }
    );
  }

  if (isUser) {
    items.push({
      key: '/modules',
      icon: React.createElement(AppstoreOutlined),
      label: 'Modules',
      userOnly: true,
    });
  }

  return items;
};

/**
 * Static bottom sidebar items.
 */
export const BOTTOM_MENU_ITEMS: MenuItem[] = [
  {
    key: '/settings',
    icon: React.createElement(SettingOutlined),
    label: 'Settings',
  },
  {
    key: 'logout',
    icon: React.createElement(LogoutOutlined),
    label: 'Logout',
  },
    {
    key: '/help',
    label: 'Help',
    icon: React.createElement(QuestionCircleOutlined),
  },
];
