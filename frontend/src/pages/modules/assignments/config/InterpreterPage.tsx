// src/pages/modules/assignments/config/InterpreterPage.tsx
import { useEffect, useState, useCallback } from 'react';
import {
  Typography,
  Form,
  Input,
  Upload,
  Button,
  Space,
  Divider,
  Dropdown,
  App,
  Alert,
} from 'antd';
import {
  UploadOutlined,
  DownloadOutlined,
  DeleteOutlined,
  FileAddOutlined,
  ExclamationCircleFilled,
} from '@ant-design/icons';

import SettingsGroup from '@/components/SettingsGroup';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import {
  uploadInterpreter,
  downloadInterpreter,
  deleteInterpreter,
  getInterpreterInfo,
} from '@/services/modules/assignments/interpreter';
import type { InterpreterInfo } from '@/types/modules/assignments/interpreter';

const { Text } = Typography;

export default function InterpreterPage() {
  const { message, modal } = App.useApp();
  const { setValue } = useViewSlot();
  const { id: moduleId } = useModule();
  const { assignment, refreshAssignment } = useAssignment();
  const assignmentId = assignment.id;

  useEffect(() => {
    setValue(
      <Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        Interpreter
      </Text>,
    );
  }, [setValue]);

  const [command, setCommand] = useState<string>('');
  const [file, setFile] = useState<File | null>(null);
  const [uploading, setUploading] = useState(false);
  const [downloading, setDownloading] = useState(false);
  const [deleting, setDeleting] = useState(false);

  // ---- Interpreter info state ----
  const [info, setInfo] = useState<InterpreterInfo | null>(null);

  const refreshInfo = useCallback(async () => {
    try {
      const res = await getInterpreterInfo(moduleId, assignmentId);
      setInfo(res.success ? (res.data ?? null) : null);
    } catch {
      setInfo(null);
    }
  }, [assignmentId, moduleId]);

  useEffect(() => {
    void refreshInfo();
  }, [refreshInfo]);

  // Prefill inputs from info (without stomping user typing)
  useEffect(() => {
    if (info?.command && command.trim() === '') {
      setCommand(info.command);
    }
  }, [info, command]);

  const beforeSelect = (f: File) => {
    setFile(f);
    return false; // prevent auto-upload
  };
  const clearFile = () => setFile(null);

  const onUpload = useCallback(async () => {
    if (!command.trim()) {
      message.warning('Please enter an interpreter command.');
      return;
    }
    if (!file) {
      message.warning('Please select an interpreter file.');
      return;
    }
    setUploading(true);
    try {
      const res = await uploadInterpreter(moduleId, assignmentId, file, command.trim());
      if (res.success) {
        message.success(res.message || 'Interpreter uploaded.');
        setCommand('');
        clearFile();
        await refreshAssignment();
        await refreshInfo();
      } else {
        message.error(res.message || 'Upload failed.');
      }
    } catch (e: any) {
      message.error(e?.message || 'Upload failed.');
    } finally {
      setUploading(false);
    }
  }, [assignmentId, moduleId, file, command, message, refreshAssignment, refreshInfo]);

  const onDownload = useCallback(async () => {
    setDownloading(true);
    try {
      await downloadInterpreter(moduleId, assignmentId);
      message.success('Download started');
    } catch (e: any) {
      message.error(e?.message || 'Download failed.');
    } finally {
      setDownloading(false);
    }
  }, [assignmentId, moduleId, message]);

  const confirmDelete = useCallback(() => {
    modal.confirm({
      centered: true,
      title: 'Remove interpreter?',
      icon: <ExclamationCircleFilled />,
      content: 'This will delete the current interpreter from the server.',
      okText: 'Delete',
      okType: 'danger',
      cancelText: 'Cancel',
      onOk: async () => {
        setDeleting(true);
        try {
          const res = await deleteInterpreter(moduleId, assignmentId);
          if (res.success) {
            message.success(res.message || 'Interpreter removed.');
            await refreshAssignment();
            await refreshInfo();
            setCommand(''); // clear command when removed
          } else {
            message.error(res.message || 'Delete failed.');
          }
        } finally {
          setDeleting(false);
        }
      },
    });
  }, [assignmentId, moduleId, modal, message, refreshAssignment, refreshInfo]);

  const hasInterpreter = !!info;
  const displayFilename = file?.name ?? info?.filename ?? '';

  return (
    <div className="flex flex-col gap-4">
      <SettingsGroup
        title="Interpreter"
        description={
          <div className="space-y-2">
            <p className="text-gray-600 dark:text-gray-400">
              Upload, download, or remove the interpreter binary/script and specify its run command.
            </p>
            {!hasInterpreter && (
              <Alert
                type="info"
                showIcon
                message="No interpreter uploaded yet"
                description="Select a file and provide the run command, then click Upload."
              />
            )}
          </div>
        }
      >
        <Form layout="vertical" className="max-w-2xl">
          <Form.Item
            label="Run Command"
            tooltip='Example: "python3 main.py" or "./interpreter --flag"'
          >
            <Input
              placeholder="e.g. python3 main.py"
              value={command}
              onChange={(e) => setCommand(e.target.value)}
              allowClear
            />
          </Form.Item>

          <Form.Item
            label="Interpreter File"
            tooltip="Upload exactly one file; uploading again will overwrite the previous interpreter."
          >
            {/* Stack on mobile; inline on ≥sm */}
            <div className="flex flex-col sm:flex-row sm:items-center gap-2">
              {/* File chooser + filename (stay together) */}
              <Space.Compact className="w-full sm:max-w-xl">
                <Upload
                  accept="*/*"
                  maxCount={1}
                  beforeUpload={beforeSelect}
                  showUploadList={false}
                >
                  <Button icon={<FileAddOutlined />}>Choose File</Button>
                </Upload>
                <Input
                  readOnly
                  value={displayFilename}
                  placeholder="No file selected"
                  allowClear
                  onChange={clearFile}
                  className="w-full" // <- responsive, no fixed 280px
                />
              </Space.Compact>

              {/* Primary upload + dropdown (below on mobile, right on desktop) */}
              <Dropdown.Button
                type="primary"
                className="w-full sm:w-auto" // full width on mobile, auto on desktop
                onClick={onUpload}
                loading={uploading}
                menu={{
                  items: [
                    {
                      key: 'download',
                      icon: <DownloadOutlined />,
                      label: 'Download',
                      disabled: !hasInterpreter || downloading,
                    },
                    { type: 'divider' },
                    {
                      key: 'delete',
                      icon: <DeleteOutlined />,
                      label: 'Delete',
                      danger: true,
                      disabled: !hasInterpreter || deleting,
                    },
                  ],
                  onClick: ({ key }) => {
                    if (key === 'download') onDownload();
                    if (key === 'delete') confirmDelete();
                  },
                }}
              >
                <UploadOutlined /> Upload
              </Dropdown.Button>
            </div>

            <Divider className="my-2" />
            <Typography.Paragraph type="secondary" className="!mb-0">
              Notes:
              <ul className="list-disc ml-6">
                <li>Only one interpreter may exist per assignment.</li>
                <li>Uploading again overwrites the existing interpreter.</li>
                <li>Download/Delete will fail with a message if no interpreter is present.</li>
              </ul>
            </Typography.Paragraph>
          </Form.Item>
        </Form>
      </SettingsGroup>
    </div>
  );
}
