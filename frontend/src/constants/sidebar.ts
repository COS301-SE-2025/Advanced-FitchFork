import {
  HomeOutlined,
  SettingOutlined,
  LogoutOutlined,
  UserOutlined,
  AppstoreOutlined,
  BarChartOutlined,
  CalendarOutlined,
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
  const { isAdmin, isUser, modulesByRole } = useAuth();

  const items: MenuItem[] = [
    {
      key: '/home',
      icon: React.createElement(HomeOutlined),
      label: 'Home',
    },
    {
      key: '/calendar',
      icon: React.createElement(CalendarOutlined),
      label: 'Calendar',
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
    const children: MenuItem[] = [];

    if (modulesByRole.Student.length > 0) {
      children.push({
        key: '/modules/enrolled',
        label: 'Enrolled',
        userOnly: true,
      });
    }

    if (modulesByRole.Tutor.length > 0) {
      children.push({
        key: '/modules/tutoring',
        label: 'Tutoring',
        userOnly: true,
      });
    }

    if (modulesByRole.Lecturer.length > 0) {
      children.push({
        key: '/modules/lecturing',
        label: 'Lecturing',
        userOnly: true,
      });
    }

    if (children.length > 0) {
      children.push({
        key: '/modules',
        label: 'All Modules',
        userOnly: true,
      });

      items.push({
        key: '/modules',
        icon: React.createElement(AppstoreOutlined),
        label: 'Modules',
        userOnly: true,
        children,
      });
    }
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
];
