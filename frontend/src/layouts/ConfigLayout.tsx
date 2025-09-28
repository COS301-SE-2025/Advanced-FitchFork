import { useMemo, useState } from 'react';
import { Typography, Menu, Button, Upload, Space, Tooltip } from 'antd';
import { Link, Outlet, useLocation } from 'react-router-dom';
import { UploadOutlined, DownloadOutlined } from '@ant-design/icons';
import { useAssignment } from '@/context/AssignmentContext';
import { message } from '@/utils/message';
import { listAssignmentFiles, downloadAssignmentFile } from '@/services/modules/assignments';
import { getAssignmentConfig, setAssignmentConfig } from '@/services/modules/assignments/config';
import Tip from '@/components/common/Tip';
import { requiresMainForMode, requiresInterpreterForMode } from '@/policies/submission';
import type { SubmissionMode } from '@/types/modules/assignments/config';

type MenuKey =
  | 'assignment'
  | 'execution'
  | 'marking'
  | 'coverage'
  | 'security'
  | 'gatlam'
  | 'interpreter'
  | 'files-main'
  | 'files-makefile'
  | 'files-memo'
  | 'files-spec';

const ConfigLayout = () => {
  const { assignment, config, refreshAssignment } = useAssignment();
  const moduleId = assignment.module_id ?? assignment.module_id ?? (assignment as any)?.module?.id;
  const assignmentId = assignment.id;
  const location = useLocation();
  const path = location.pathname;

  const [importing, setImporting] = useState(false);
  const [downloading, setDownloading] = useState(false);

  const selectedKey: MenuKey = useMemo(() => {
    if (path.endsWith('/marking')) return 'marking';
    if (path.endsWith('/execution')) return 'execution';
    if (path.endsWith('/code-coverage')) return 'coverage';
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
  const mode = (config?.project?.submission_mode ?? 'manual') as SubmissionMode;
  const showAI = requiresInterpreterForMode(mode); // gatlam | rng | codecoverage
  const needsMain = requiresMainForMode(mode); // manual only

  const fileChildren = [
    ...(needsMain ? [{ key: 'files-main', label: <Link to="files/main">Main File</Link> }] : []),
    { key: 'files-makefile', label: <Link to="files/makefile">Makefile</Link> },
    { key: 'files-memo', label: <Link to="files/memo">Memo File</Link> },
    { key: 'files-spec', label: <Link to="files/spec">Specification</Link> },
  ];

  const generalGroup = {
    key: 'general-group',
    label: 'General',
    type: 'group' as const,
    children: [
      { key: 'assignment', label: <Link to="assignment">Assignment</Link> },
      { key: 'execution', label: <Link to="execution">Execution Limits</Link> },
      { key: 'marking', label: <Link to="marking">Marking & Feedback</Link> },
      { key: 'coverage', label: <Link to="code-coverage">Code Coverage</Link> },
      { key: 'security', label: <Link to="security">Security</Link> },
    ],
  };

  const aiGroup = showAI
    ? [
        {
          key: 'ai-group',
          label: 'AI',
          type: 'group' as const,
          children: [
            { key: 'gatlam', label: <Link to="gatlam">AI Settings</Link> },
            { key: 'interpreter', label: <Link to="interpreter">Interpreter</Link> },
          ],
        },
      ]
    : [];

  const menuItems = [
    generalGroup,
    ...aiGroup,
    {
      key: 'files-group',
      label: 'Files',
      type: 'group' as const,
      children: fileChildren,
    },
  ];

  // ----------------------------- import/export handlers -----------------------------
  const importProps = {
    accept: '.json,application/json',
    showUploadList: false,
    multiple: false,
    customRequest: async ({ file, onSuccess, onError }: any) => {
      try {
        setImporting(true);
        const text = await (file as File).text();
        const parsed = JSON.parse(text);
        if (typeof parsed !== 'object' || parsed == null) {
          throw new Error('Config JSON must be an object');
        }
        const res = await setAssignmentConfig(moduleId, assignmentId, parsed);
        if (!res?.success) throw new Error(res?.message || 'Failed to save config');
        message.success('Config imported and saved.');
        await refreshAssignment?.();
        onSuccess?.(true);
      } catch (e: any) {
        console.error(e);
        message.error(e?.message || 'Import failed');
        onError?.(e);
      } finally {
        setImporting(false);
      }
    },
  };

  const downloadConfig = async () => {
    try {
      setDownloading(true);
      const files = await listAssignmentFiles(moduleId, assignmentId);
      const cfg = (Array.isArray(files) ? files : files?.data)?.find(
        (f: any) => f.file_type === 'config',
      );
      if (cfg) {
        await downloadAssignmentFile(moduleId, assignmentId, Number(cfg.id));
        message.success('Download started');
        return;
      }
      const res = await getAssignmentConfig(moduleId, assignmentId);
      if (res?.success) {
        const content = JSON.stringify(res.data ?? {}, null, 2);
        const blob = new Blob([content], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'config.json';
        document.body.appendChild(a);
        a.click();
        a.remove();
        URL.revokeObjectURL(url);
        message.success('Config downloaded');
      } else {
        throw new Error(res?.message || 'Failed to fetch config');
      }
    } catch (e: any) {
      console.error(e);
      message.error(e?.message || 'Download failed');
    } finally {
      setDownloading(false);
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <div className="hidden sm:grid w-full grid-cols-[240px_minmax(0,1fr)] bg-white dark:bg-gray-900 border rounded-md border-gray-200 dark:border-gray-800">
        <div className="bg-gray-50 dark:bg-gray-950 border-r border-gray-200 dark:border-gray-800 px-2 py-2">
          <Menu
            mode="inline"
            selectedKeys={[selectedKey]}
            items={menuItems}
            className="!bg-transparent !p-0"
            style={{ border: 'none' }}
          />
        </div>

        <div className="flex flex-col">
          <div className="flex justify-between items-center flex-wrap gap-2 border-b border-gray-200 dark:border-gray-800 p-4">
            <Space align="center" size={6} className="flex-wrap">
              <Typography.Title level={4} className="!mb-0">
                Assignment Configuration
              </Typography.Title>
              <Tip
                iconOnly
                newTab
                to="/help/assignments/config/overview#overview"
                text="Config overview help"
              />
            </Space>

            <Space align="center" wrap>
              <Upload {...importProps}>
                <Tooltip title="Import an execution config from a JSON file">
                  <Button icon={<UploadOutlined />} loading={importing}>
                    Import JSON
                  </Button>
                </Tooltip>
              </Upload>
              <Tooltip title="Download the current execution config">
                <Button icon={<DownloadOutlined />} onClick={downloadConfig} loading={downloading}>
                  Download JSON
                </Button>
              </Tooltip>
            </Space>
          </div>
          <div className="flex flex-col p-4">
            <Outlet />
          </div>
        </div>
      </div>

      <div className="block sm:hidden">
        <Outlet />
      </div>
    </div>
  );
};

export default ConfigLayout;
