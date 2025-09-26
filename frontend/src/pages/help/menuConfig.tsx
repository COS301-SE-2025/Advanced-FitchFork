import { InfoCircleOutlined, AppstoreOutlined, FileTextOutlined, SettingOutlined } from '@ant-design/icons';
import type { MenuProps } from 'antd';

export type HelpMenuItem = Required<NonNullable<MenuProps['items']>>[number];

export type HelpLeaf = { key: string; label: string };

export const HELP_MENU_ITEMS: HelpMenuItem[] = [
  {
    key: 'getting-started',
    icon: <InfoCircleOutlined />,
    label: 'Getting Started',
    children: [{ key: 'getting-started/overview', label: 'Overview' }],
  },
  {
    key: 'modules',
    icon: <AppstoreOutlined />,
    label: 'Modules',
    children: [
      { key: 'modules/attendance', label: 'Attendance' },
      { key: 'modules/personnel', label: 'Personnel & Roles' },
    ],
  },
  {
    key: 'assignments',
    icon: <FileTextOutlined />,
    label: 'Assignments',
    children: [
      {
        type: 'group',
        label: 'Setup',
        children: [{ key: 'assignments/setup', label: 'Full Setup Guide' }],
      },
      {
        key: 'assignments/config-sections',
        label: 'Assignment Config',
        children: [
          { key: 'assignments/config/overview', label: 'Overview' },
          { key: 'assignments/config/project', label: 'Language & Mode' },
          { key: 'assignments/config/execution', label: 'Execution' },
          { key: 'assignments/config/output', label: 'Output' },
          { key: 'assignments/config/marking', label: 'Marking' },
          { key: 'assignments/config/security', label: 'Security' },
          { key: 'assignments/config/gatlam', label: 'GATLAM' },
        ],
      },
      {
        type: 'group',
        label: 'Files',
        children: [
          { key: 'assignments/files/main-files', label: 'Main File' },
          { key: 'assignments/files/makefile', label: 'Makefile' },
          { key: 'assignments/files/memo-files', label: 'Memo Files' },
          { key: 'assignments/files/specification', label: 'Specification' },
        ],
      },
      {
        type: 'group',
        label: 'Concepts',
        children: [
          { key: 'assignments/tasks', label: 'Tasks' },
          { key: 'assignments/code-coverage', label: 'Code Coverage' },
          { key: 'assignments/gatlam', label: 'GATLAM & Interpreter' },
        ],
      },
      {
        type: 'group',
        label: 'Submissions',
        children: [
          { key: 'assignments/submissions/how-to-submit', label: 'How to Submit' },
        ],
      },
      {
        type: 'group',
        label: 'Plagiarism',
        children: [{ key: 'assignments/plagiarism/moss', label: 'Plagiarism & MOSS' }],
      },
      {
        type: 'group',
        label: 'Grading',
        children: [
          { key: 'assignments/memo-output', label: 'Memo Output' },
          { key: 'assignments/mark-allocator', label: 'Mark Allocation' },
        ],
      },
    ],
  },
  {
    key: 'support',
    icon: <SettingOutlined />,
    label: 'Support',
    children: [
      { key: 'support/system-monitoring', label: 'System Monitoring' },
      { key: 'support/contact', label: 'Contact' },
    ],
  },
];

function flattenLeafs(items: HelpMenuItem[] | undefined): HelpLeaf[] {
  const out: HelpLeaf[] = [];
  const walk = (arr?: any[]) => {
    if (!arr) return;
    for (const it of arr) {
      if (!it) continue;
      if (it.type === 'group') {
        walk(it.children);
        continue;
      }
      if (Array.isArray(it.children) && it.children.length) {
        walk(it.children);
        continue;
      }
      if (typeof it.key === 'string' && typeof it.label === 'string') {
        out.push({ key: it.key, label: it.label });
      }
    }
  };
  walk(items);
  return out;
}

export const HELP_LEAF_ROUTES = flattenLeafs(HELP_MENU_ITEMS);

export function findHelpLeafLabel(key: string): string | null {
  return HELP_LEAF_ROUTES.find((leaf) => leaf.key === key)?.label ?? null;
}
