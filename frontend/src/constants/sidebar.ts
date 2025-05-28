import {
  HomeOutlined,
  SettingOutlined,
  LogoutOutlined,
  UserOutlined,
  AppstoreOutlined,
  BarChartOutlined,
} from '@ant-design/icons';
import React from 'react';

export interface MenuItem {
  key: string;
  label: string;
  icon?: React.ReactNode;
  adminOnly?: boolean;
  userOnly?: boolean;
  children?: MenuItem[];
}

export const TOP_MENU_ITEMS: MenuItem[] = [
  {
    key: '/home',
    icon: React.createElement(HomeOutlined),
    label: 'Home',
    adminOnly: false,
  },
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
    key: '/modules',
    icon: React.createElement(AppstoreOutlined),
    label: 'Modules',
    userOnly: true,
    children: [
      {
        key: '/modules/enrolled',
        label: 'Enrolled',
        userOnly: true,
      },
      {
        key: '/modules/tutoring',
        label: 'Tutoring',
        userOnly: true,
      },
      {
        key: '/modules/lecturing',
        label: 'Lecturing',
        userOnly: true,
      },
      {
        key: '/modules', // Admin default view
        label: 'All Modules',
        userOnly: true,
      },
    ],
  },
  {
    key: '/reports',
    icon: React.createElement(BarChartOutlined),
    label: 'Reports',
    adminOnly: true,
  },
];


export const BOTTOM_MENU_ITEMS: MenuItem[] = [
  {
    key: '/settings',
    icon: React.createElement(SettingOutlined),
    label: 'Settings',
    adminOnly: false,
  },
  {
    key: 'logout',
    icon: React.createElement(LogoutOutlined),
    label: 'Logout',
    adminOnly: false,
  },
];
