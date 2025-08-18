import { useState, type ReactNode } from 'react';
import { Typography, Card, Button, Steps, Alert, Segmented, Space } from 'antd';
import {
  ThunderboltOutlined,
  CheckCircleOutlined,
  CloudUploadOutlined,
  FileTextOutlined,
  LoadingOutlined,
  CloseCircleOutlined,
} from '@ant-design/icons';

import { useModule } from '@/context/ModuleContext';
import { useAssignmentSetup } from '@/context/AssignmentSetupContext';
import { uploadAssignmentFile } from '@/services/modules/assignments';
import { createTask } from '@/services/modules/assignments/tasks';
import { generateMemoOutput } from '@/services/modules/assignments/memo-output';
import { generateMarkAllocator } from '@/services/modules/assignments/mark-allocator';
import { DEFAULT_ASSIGNMENT_CONFIG } from '@/constants/assignments';
import { message } from '@/utils/message';
import { type Language, LANGUAGE_TASKS, starterPath } from '@/constants/assignments/defaults';

const { Title, Paragraph, Text } = Typography;

type StepMeta = { title: string; baseIcon?: ReactNode };
type Props = { onManual?: () => void };

async function fetchAssetAsFile(
  url: string,
  downloadName: string,
  mime = 'application/zip',
): Promise<File> {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`Failed to fetch ${url}: ${res.status}`);
  const blob = await res.blob();
  return new File([blob], downloadName, { type: mime });
}

const stepsMeta: readonly StepMeta[] = [
  { title: 'Upload Config', baseIcon: <FileTextOutlined /> },
  { title: 'Fetch Files', baseIcon: <CloudUploadOutlined /> },
  { title: 'Upload Files' },
  { title: 'Create Tasks' },
  { title: 'Memo Output' },
  { title: 'Mark Allocator' },
  { title: 'Complete', baseIcon: <CheckCircleOutlined /> },
] as const;

const StepWelcome = ({ onManual }: Props) => {
  const module = useModule();
  const { assignmentId, refreshAssignment, onStepComplete } = useAssignmentSetup();

  const [language, setLanguage] = useState<Language>('java');
  const [running, setRunning] = useState(false);
  const [step, setStep] = useState(0); // 0..6
  const [err, setErr] = useState<string | null>(null);

  const runQuickStart = async () => {
    if (!assignmentId) {
      message.error('Missing assignment ID. Create the assignment first.');
      return;
    }

    setRunning(true);
    setErr(null);
    setStep(0);

    try {
      // 0) Upload config FIRST
      {
        const blob = new Blob([JSON.stringify(DEFAULT_ASSIGNMENT_CONFIG, null, 2)], {
          type: 'application/json',
        });
        const configFile = new File([blob], 'config.json', { type: 'application/json' });
        const resCfg = await uploadAssignmentFile(module.id, assignmentId, 'config', configFile);
        if (!resCfg.success) throw new Error(resCfg.message || 'Failed to upload config.json');
      }
      await refreshAssignment?.();

      // 1) Fetch default zips for selected language
      setStep(1);
      const [mainFile, memoFile, makefileZip] = await Promise.all([
        fetchAssetAsFile(starterPath(language, 'main'), 'main.zip'),
        fetchAssetAsFile(starterPath(language, 'memo'), 'memo.zip'),
        fetchAssetAsFile(starterPath(language, 'makefile'), 'makefile.zip'),
      ]);

      // 2) Upload required files
      setStep(2);
      {
        const [u1, u2, u3] = await Promise.all([
          uploadAssignmentFile(module.id, assignmentId, 'main', mainFile),
          uploadAssignmentFile(module.id, assignmentId, 'memo', memoFile),
          uploadAssignmentFile(module.id, assignmentId, 'makefile', makefileZip),
        ]);
        if (!u1.success) throw new Error(u1.message || 'Failed to upload main.zip');
        if (!u2.success) throw new Error(u2.message || 'Failed to upload memo.zip');
        if (!u3.success) throw new Error(u3.message || 'Failed to upload makefile.zip');
      }
      await refreshAssignment?.();

      // 3) Seed default tasks — language specific
      setStep(3);
      {
        let nextNumber = 1;
        const tasks = LANGUAGE_TASKS[language];
        for (const t of tasks) {
          const res = await createTask(module.id, assignmentId, {
            task_number: nextNumber++,
            name: t.name,
            command: t.command,
          });
          if (!res.success) throw new Error(res.message || `Failed to create task: ${t.name}`);
        }
      }
      await refreshAssignment?.();

      // 4) Generate memo output
      setStep(4);
      {
        const resMemo = await generateMemoOutput(module.id, assignmentId);
        if (!resMemo.success) throw new Error(resMemo.message || 'Memo output generation failed');
        message.success(resMemo.message || 'Memo output generated');
      }
      await refreshAssignment?.();

      // 5) Generate mark allocator
      setStep(5);
      {
        const resAlloc = await generateMarkAllocator(module.id, assignmentId);
        if (!resAlloc.success)
          throw new Error(resAlloc.message || 'Mark allocator generation failed');
        message.success(resAlloc.message || 'Mark allocator generated');
      }
      await refreshAssignment?.();
      onStepComplete?.();

      // 6) Done
      setStep(6);
      message.success('Starter setup complete. You’re ready to proceed.');
    } catch (e: any) {
      setErr(e?.message || 'Quick start failed');
      message.error(e?.message || 'Quick start failed');
    } finally {
      setRunning(false);
    }
  };

  const stepStatus = err ? 'error' : step === 6 ? 'finish' : 'process';

  const renderIcon = (idx: number) => {
    if (err && idx === step) return <CloseCircleOutlined />;
    if (running && idx === step) return <LoadingOutlined />;
    if (idx < step) return <CheckCircleOutlined />;
    return stepsMeta[idx]?.baseIcon;
  };

  return (
    <div className="space-y-6 !w-full">
      {/* Header row */}
      <div className="flex items-start justify-between gap-4">
        <div>
          <Title level={3} className="!mb-1">
            Welcome
          </Title>
          <Paragraph type="secondary" className="!mb-0">
            One-click starter pack or manual setup. Pick a language, then config, files, tasks, and
            outputs.
          </Paragraph>
        </div>

        <Space size="middle">
          {/* Language selector */}
          <Segmented
            value={language}
            onChange={(val) => setLanguage(val as Language)}
            options={[
              { label: 'Java', value: 'java' },
              { label: 'C++', value: 'cpp' },
            ]}
          />
          <Button
            type="primary"
            icon={<ThunderboltOutlined />}
            onClick={runQuickStart}
            loading={running}
            disabled={running || !assignmentId}
          >
            Use Starter Pack
          </Button>
        </Space>
      </div>

      {/* Quick Setup card */}
      <div className="!space-y-6">
        <Card className="!w-full !border-dashed !rounded-2xl !bg-white dark:!bg-gray-900 !border-gray-300 dark:!border-gray-700">
          <div className="p-4">
            <Steps
              className="!w-full"
              current={Math.min(step, stepsMeta.length - 1)}
              status={stepStatus as any}
              items={stepsMeta.map((s, i) => ({
                title: s.title,
                icon: renderIcon(i),
              }))}
            />

            {/* Simple paragraph */}
            <div className="mt-3">
              <Text type="secondary" className="text-xs block">
                The starter pack prepares config, uploads language-specific default zips, seeds
                sample tasks, and generates outputs — ready for grading in one click.
              </Text>
            </div>

            {err && (
              <Alert
                className="!mt-4"
                type="error"
                showIcon
                message="Quick Start failed"
                description={err}
              />
            )}
          </div>
        </Card>

        {/* Manual Setup card */}
        <Card className="!w-full !rounded-2xl !bg-white dark:!bg-gray-900 !border-gray-200 dark:!border-gray-800">
          <div className="p-4">
            <Title level={5} className="!mb-2">
              Prefer manual setup?
            </Title>
            <Paragraph type="secondary" className="text-sm !mb-3">
              Proceed to the next steps to upload files, define tasks, and generate outputs
              yourself.
            </Paragraph>
            {onManual && (
              <Button onClick={onManual} className="!mt-1" disabled={running}>
                Set up manually
              </Button>
            )}
          </div>
        </Card>
      </div>
    </div>
  );
};

export default StepWelcome;
