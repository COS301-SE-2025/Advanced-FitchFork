import { useMemo } from 'react';
import { Typography, Menu } from 'antd';
import { Link, Outlet, useLocation } from 'react-router-dom';
import { useAssignment } from '@/context/AssignmentContext';

type MenuKey =
  | 'assignment'
  | 'execution'
  | 'marking'
  | 'output'
  | 'security'
  | 'gatlam'
  | 'interpreter'
  | 'files-main'
  | 'files-makefile'
  | 'files-memo'
  | 'files-spec';

const ConfigLayout = () => {
  const { config } = useAssignment();
  const location = useLocation();
  const path = location.pathname;

  const selectedKey: MenuKey = useMemo(() => {
    if (path.endsWith('/marking')) return 'marking';
    if (path.endsWith('/execution')) return 'execution';
    if (path.endsWith('/output')) return 'output';
    if (path.endsWith('/security')) return 'security';
    if (path.endsWith('/gatlam')) return 'gatlam';
    if (path.endsWith('/interpreter')) return 'interpreter';
    if (path.includes('/files/main')) return 'files-main';
    if (path.includes('/files/makefile')) return 'files-makefile';
    if (path.includes('/files/memo')) return 'files-memo';
    if (path.includes('/files/spec')) return 'files-spec';
    return 'assignment';
  }, [path]);

  // Mode-aware visibility
  const submissionMode = config?.project?.submission_mode ?? 'manual';
  const isGatlam = submissionMode === 'gatlam';

  const fileChildren = [
    ...(isGatlam ? [] : [{ key: 'files-main', label: <Link to={'files/main'}>Main File</Link> }]),
    { key: 'files-makefile', label: <Link to={'files/makefile'}>Makefile</Link> },
    { key: 'files-memo', label: <Link to={'files/memo'}>Memo File</Link> },
    { key: 'files-spec', label: <Link to={'files/spec'}>Specification</Link> },
  ];

  const generalGroup = {
    key: 'general-group',
    label: 'General',
    type: 'group' as const,
    children: [
      { key: 'assignment', label: <Link to="assignment">Assignment</Link> },
      { key: 'execution', label: <Link to="execution">Execution Limits</Link> },
      { key: 'marking', label: <Link to="marking">Marking & Feedback</Link> },
      { key: 'output', label: <Link to="output">Output</Link> },
      { key: 'security', label: <Link to="security">Security</Link> },
    ],
  };

  const gatlamGroup = isGatlam
    ? [
        {
          key: 'gatlam-group',
          label: 'GATLAM',
          type: 'group' as const,
          children: [
            { key: 'gatlam', label: <Link to="gatlam">GATLAM</Link> },
            { key: 'interpreter', label: <Link to="interpreter">Interpreter</Link> },
          ],
        },
      ]
    : [];

  const menuItems = [
    generalGroup,
    ...gatlamGroup,
    {
      key: 'files-group',
      label: 'Files',
      type: 'group' as const,
      children: fileChildren,
    },
  ];

  return (
    <div className="h-full min-h-0 flex flex-col">
      <div className="hidden sm:flex flex-1 min-h-0 bg-white dark:bg-gray-900 border rounded-md border-gray-200 dark:border-gray-800 overflow-hidden">
        {/* Sidebar */}
        <div className="w-[240px] flex-shrink-0 bg-gray-50 dark:bg-gray-950 border-r border-gray-200 dark:border-gray-800 px-2 py-2 overflow-auto">
          <Menu
            mode="inline"
            selectedKeys={[selectedKey]}
            items={menuItems}
            className="!bg-transparent !p-0"
            style={{ border: 'none' }}
          />
        </div>

        {/* Main */}
        <div className="flex-1 min-h-0 flex flex-col">
          <div className="flex justify-between items-center flex-wrap gap-2 border-b border-gray-200 dark:border-gray-800 p-4">
            <Typography.Title level={4} className="!mb-0">
              Assignment Configuration
            </Typography.Title>
          </div>

          <div className="flex-1 min-h-0 overflow-auto flex flex-col p-4">
            <Outlet />
          </div>
        </div>
      </div>

      {/* Mobile */}
      <div className="block sm:hidden flex-1 min-h-0 overflow-auto">
        <Outlet />
      </div>
    </div>
  );
};

export default ConfigLayout;
