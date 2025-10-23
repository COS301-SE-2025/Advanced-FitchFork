// StepFilesResources.tsx
import { useEffect, useMemo, useState, type ReactNode } from 'react';
import { Typography, Upload, Button, List, Space, Input, Tag, Dropdown } from 'antd';
import { UploadOutlined, DownloadOutlined, CodeOutlined, DeleteOutlined } from '@ant-design/icons';

import { useModule } from '@/context/ModuleContext';
import { useAssignmentSetup } from '@/context/AssignmentSetupContext';
import { uploadAssignmentFile, downloadAssignmentFile } from '@/services/modules/assignments';
import {
  uploadInterpreter,
  downloadInterpreter,
  deleteInterpreter,
  getInterpreterInfo,
} from '@/services/modules/assignments/interpreter';
import type { AssignmentFile } from '@/types/modules/assignments';
import type { InterpreterInfo } from '@/types/modules/assignments/interpreter';
import Tip from '@/components/common/Tip';
import { requiresInterpreterForMode } from '@/policies/submission';
import type { SubmissionMode } from '@/types/modules/assignments/config';
import { GatlamLink } from '@/components/common';

const { Title, Paragraph } = Typography;

type RowKey = 'main' | 'memo' | 'makefile' | 'interpreter';

const StepFilesResources = () => {
  const module = useModule();
  const { assignmentId, assignment, readiness, refreshAssignment } = useAssignmentSetup();
  const effectiveMode = (readiness?.submission_mode ?? undefined) as SubmissionMode | undefined;
  const needsInterpreter = requiresInterpreterForMode(effectiveMode);

  const [files, setFiles] = useState<AssignmentFile[]>(assignment?.files ?? []);
  useEffect(() => setFiles(assignment?.files ?? []), [assignment?.files]);

  // interpreter UI state
  const [command, setCommand] = useState('');
  const [interpreterInfo, setInterpreterInfo] = useState<InterpreterInfo | null>(null);
  const [uploading, setUploading] = useState<RowKey | null>(null);
  useEffect(() => {
    (async () => {
      if (!assignmentId) return;
      if (needsInterpreter && readiness?.interpreter_present) {
        const res = await getInterpreterInfo(module.id, assignmentId);
        if (res.success) setInterpreterInfo(res.data);
      } else {
        setInterpreterInfo(null);
      }
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [needsInterpreter, readiness?.interpreter_present, assignmentId]);

  type RowItem = {
    key: RowKey;
    title: string;
    present: boolean;
    desc: ReactNode;
    fileType?: string;
  };

  const list: RowItem[] = useMemo(() => {
    return needsInterpreter
      ? [
          {
            key: 'interpreter',
            title: 'Interpreter',
            present: !!(readiness as any)?.interpreter_present,
            desc: (
              <>
                Upload the interpreter archive and command used to execute generated programs in{' '}
                <GatlamLink tone="inherit" icon={false} underline={false}>
                  GATLAM
                </GatlamLink>{' '}
                mode.
              </>
            ),
          },
          {
            key: 'memo',
            title: 'Memo Files',
            present: !!readiness?.memo_present,
            desc: 'Reference implementation zipped at the root; drives memo output.',
            fileType: '.zip',
          },
          {
            key: 'makefile',
            title: 'Makefile',
            present: !!readiness?.makefile_present,
            desc: 'Archive with a root-level Makefile defining build/run targets.',
            fileType: '.zip',
          },
        ]
      : [
          {
            key: 'main',
            title: 'Main Files',
            present: !!readiness?.main_present,
            desc: (
              <>
                Zipped entry file at archive root; orchestrates execution & prints labels. Not used
                when{' '}
                <GatlamLink tone="inherit" icon={false} underline={false}>
                  GATLAM
                </GatlamLink>{' '}
                mode is enabled.
              </>
            ),
            fileType: '.zip',
          },
          {
            key: 'memo',
            title: 'Memo Files',
            present: !!readiness?.memo_present,
            desc: 'Reference implementation zipped at the root; drives memo output.',
            fileType: '.zip',
          },
          {
            key: 'makefile',
            title: 'Makefile',
            present: !!readiness?.makefile_present,
            desc: 'Archive with a root-level Makefile defining build/run targets.',
            fileType: '.zip',
          },
        ];
  }, [needsInterpreter, readiness]);

  const uploadedFor = (k: RowKey) =>
    k === 'interpreter' ? [] : files.filter((f) => f.file_type === k);

  // Primary help link per row (opened via icon)

  const primaryHelpFor = (k: RowKey): { label: string; href: string } | null => {
    switch (k) {
      case 'main':
        return {
          label: 'Main file requirements',
          href: '/help/assignments/files/main-files#requirements',
        };
      case 'memo':
        return {
          label: 'Memo files requirements',
          href: '/help/assignments/files/memo-files#requirements',
        };
      case 'makefile':
        return {
          label: 'Makefile requirements',
          href: '/help/assignments/files/makefile#requirements',
        };
      case 'interpreter':
        return { label: 'GATLAM & Interpreter', href: '/help/assignments/gatlam' };
      default:
        return null;
    }
  };

  const handleUploadZip = async (key: RowKey, file: File) => {
    if (!assignmentId) return false;
    if (key === 'interpreter') return false; // handled elsewhere
    if (uploading) return false;

    setUploading(key);
    try {
      const res = await uploadAssignmentFile(module.id, assignmentId, key as any, file);
      if (res.success) {
        await refreshAssignment?.();
      }
    } finally {
      setUploading(null);
    }
    return false;
  };

  const handleUploadInterpreter = async (file: File) => {
    if (!assignmentId || !command.trim()) return false;
    if (uploading) return false;

    setUploading('interpreter');
    try {
      const res = await uploadInterpreter(module.id, assignmentId, file, command.trim());
      if (res.success) {
        setCommand('');
        await refreshAssignment?.();
        const info = await getInterpreterInfo(module.id, assignmentId);
        if (info.success) setInterpreterInfo(info.data);
        return true;
      }
    } finally {
      setUploading(null);
    }
    return false;
  };

  return (
    <div className="space-y-6">
      <div>
        <Title level={3} className="!mb-1">
          Files & Resources
        </Title>
        <Paragraph type="secondary" className="!mb-0">
          Provide the required artifacts. The checklist updates automatically.
        </Paragraph>
      </div>

      <List
        bordered
        dataSource={list}
        renderItem={(item) => {
          const { key, title, present, desc } = item;
          const uploaded = uploadedFor(key);
          return (
            <List.Item className="flex flex-col md:flex-row md:items-center md:justify-between gap-3">
              <div className="flex items-start gap-3">
                <div>
                  <div className="flex items-center gap-2 flex-wrap">
                    <div className="font-medium">{title}</div>
                    {primaryHelpFor(key) && (
                      <Tip
                        iconOnly
                        to={primaryHelpFor(key)!.href}
                        newTab
                        text={primaryHelpFor(key)!.label}
                        className="ml-0"
                      />
                    )}
                    {item.fileType && <Tag>{item.fileType}</Tag>}
                    <Tag color={present ? 'success' : 'warning'}>
                      {present ? 'Provided' : 'Missing'}
                    </Tag>
                  </div>
                  <div className="text-xs text-gray-500">{desc}</div>
                  {/* Subtle inline help via icon; explicit link list removed */}
                  {/* Removed filename display; actions moved next to upload */}
                </div>
              </div>

              <div className="w-full md:w-auto">
                {key === 'interpreter' ? (
                  <Space.Compact className="w-full md:w-[520px]">
                    <Input
                      placeholder="Interpreter command (e.g., sh run.sh)"
                      prefix={<CodeOutlined />}
                      value={command}
                      onChange={(e) => setCommand(e.target.value)}
                    />
                    <Upload
                      beforeUpload={handleUploadInterpreter}
                      showUploadList={false}
                      accept="*/*"
                      disabled={uploading === 'interpreter'}
                    >
                      <Button icon={<UploadOutlined />} loading={uploading === 'interpreter'}>
                        Upload
                      </Button>
                    </Upload>
                    {interpreterInfo && (
                      <>
                        <Button
                          onClick={() => downloadInterpreter(module.id, assignmentId!)}
                          icon={<DownloadOutlined />}
                        >
                          Download
                        </Button>
                        <Button
                          danger
                          onClick={() =>
                            deleteInterpreter(module.id, assignmentId!).then(() =>
                              refreshAssignment?.(),
                            )
                          }
                          icon={<DeleteOutlined />}
                        >
                          Delete
                        </Button>
                      </>
                    )}
                  </Space.Compact>
                ) : (
                  <Space>
                    <Upload
                      multiple={key === 'main'}
                      accept=".zip"
                      beforeUpload={(file) => handleUploadZip(key, file)}
                      showUploadList={false}
                      disabled={uploading === key}
                    >
                      <Button icon={<UploadOutlined />} loading={uploading === key}>
                        Upload
                      </Button>
                    </Upload>
                    {uploaded.length === 1 && (
                      <Button
                        icon={<DownloadOutlined />}
                        onClick={() =>
                          downloadAssignmentFile(module.id, assignmentId!, uploaded[0].id)
                        }
                      >
                        Download
                      </Button>
                    )}
                    {uploaded.length > 1 && (
                      <Dropdown
                        menu={{
                          items: uploaded.map((f, idx) => ({
                            key: String(f.id),
                            label: `Download ${idx + 1}`,
                          })),
                          onClick: ({ key }) =>
                            downloadAssignmentFile(module.id, assignmentId!, Number(key)),
                        }}
                      >
                        <Button icon={<DownloadOutlined />}>Downloads</Button>
                      </Dropdown>
                    )}
                  </Space>
                )}
              </div>
            </List.Item>
          );
        }}
      />
    </div>
  );
};

export default StepFilesResources;
