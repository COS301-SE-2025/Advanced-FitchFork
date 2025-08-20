import { useState, type ReactNode } from 'react';
import {
  Typography,
  Button,
  Steps,
  Alert,
  Segmented,
  Space,
  Tooltip,
  Card,
  Tag,
  Divider,
} from 'antd';
import {
  ThunderboltOutlined,
  CheckCircleOutlined,
  CloudUploadOutlined,
  FileTextOutlined,
  LoadingOutlined,
  CloseCircleOutlined,
  InfoCircleOutlined,
  CoffeeOutlined,
  CodeOutlined,
  RocketOutlined,
  ToolOutlined,
} from '@ant-design/icons';

import { useModule } from '@/context/ModuleContext';
import { useAssignmentSetup } from '@/context/AssignmentSetupContext';
import { createTask } from '@/services/modules/assignments/tasks';
import { generateMemoOutput } from '@/services/modules/assignments/memo-output';
import { generateMarkAllocator } from '@/services/modules/assignments/mark-allocator';
import { message } from '@/utils/message';
import { type Language, LANGUAGE_TASKS, starterPath } from '@/constants/assignments/defaults';

// config services
import { resetAssignmentConfig, setAssignmentConfig } from '@/services/modules/assignments/config';

import type { SubmissionMode } from '@/types/modules/assignments/config';

const { Title, Paragraph, Text } = Typography;

type SimpleStepMeta = { title: string; baseIcon?: ReactNode };
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

/** UI: collapsed 4-step progress (we still run the same inner workflow) */
const simpleSteps: readonly SimpleStepMeta[] = [
  { title: 'Configure', baseIcon: <FileTextOutlined /> }, // reset+save config
  { title: 'Prepare', baseIcon: <CloudUploadOutlined /> }, // fetch+upload files + seed tasks
  { title: 'Outputs', baseIcon: <RocketOutlined /> }, // memo output + mark allocator
  { title: 'Complete', baseIcon: <CheckCircleOutlined /> },
] as const;

const StepWelcome = ({ onManual }: Props) => {
  const module = useModule();
  const { assignmentId, refreshAssignment, setConfig } = useAssignmentSetup();

  // Selections shared by flows
  const [language, setLanguage] = useState<Language>('java');
  const [submissionMode, setSubmissionMode] = useState<SubmissionMode>('manual');

  // Starter-pack internal progress (0..6)
  // 0: save config, 1: fetch assets, 2: upload files, 3: seed tasks, 4: memo, 5: allocator, 6: done
  const [step, setStep] = useState(0);
  const [running, setRunning] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  /** Map internal step -> collapsed UI step (0..3) */
  const uiStep = (() => {
    if (step <= 0) return 0; // Configure
    if (step <= 3) return 1; // Prepare
    if (step <= 5) return 2; // Outputs
    return 3; // Complete
  })();

  /** Reset → patch project.language / project.submission_mode → save */
  const saveConfigFromDefaults = async () => {
    if (!assignmentId) throw new Error('Missing assignment ID. Create the assignment first.');

    const resetRes = await resetAssignmentConfig(module.id, assignmentId);
    if (!resetRes.success)
      throw new Error(resetRes.message || 'Failed to reset config to defaults');
    const base = resetRes.data;

    const patched = {
      ...base,
      project: { ...base.project, language, submission_mode: submissionMode },
    };

    const setRes = await setAssignmentConfig(module.id, assignmentId, patched);
    if (!setRes.success) throw new Error(setRes.message || 'Failed to save config');
    setConfig(setRes.data ?? patched);

    await refreshAssignment?.();
  };

  // ---------- Starter Pack ----------
  const runQuickStart = async () => {
    if (!assignmentId) {
      message.error('Missing assignment ID. Create the assignment first.');
      return;
    }

    setRunning(true);
    setErr(null);
    setStep(0);

    try {
      // (Configure)
      await saveConfigFromDefaults(); // step = 0

      // (Prepare) fetch assets
      setStep(1);
      const [mainFile, memoFile, makefileZip] = await Promise.all([
        fetchAssetAsFile(starterPath(language, 'main'), 'main.zip'),
        fetchAssetAsFile(starterPath(language, 'memo'), 'memo.zip'),
        fetchAssetAsFile(starterPath(language, 'makefile'), 'makefile.zip'),
      ]);

      // (Prepare) upload files
      setStep(2);
      const { uploadAssignmentFile } = await import('@/services/modules/assignments');
      const [u1, u2, u3] = await Promise.all([
        uploadAssignmentFile(module.id, assignmentId, 'main', mainFile),
        uploadAssignmentFile(module.id, assignmentId, 'memo', memoFile),
        uploadAssignmentFile(module.id, assignmentId, 'makefile', makefileZip),
      ]);
      if (!u1.success) throw new Error(u1.message || 'Failed to upload main.zip');
      if (!u2.success) throw new Error(u2.message || 'Failed to upload memo.zip');
      if (!u3.success) throw new Error(u3.message || 'Failed to upload makefile.zip');
      await refreshAssignment?.();

      // (Prepare) seed default tasks
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

      // (Outputs) generate memo
      setStep(4);
      {
        const resMemo = await generateMemoOutput(module.id, assignmentId);
        if (!resMemo.success) throw new Error(resMemo.message || 'Memo output generation failed');
        message.success(resMemo.message || 'Memo output generated');
      }
      await refreshAssignment?.();

      // (Outputs) generate allocator
      setStep(5);
      {
        const resAlloc = await generateMarkAllocator(module.id, assignmentId);
        if (!resAlloc.success)
          throw new Error(resAlloc.message || 'Mark allocator generation failed');
        message.success(resAlloc.message || 'Mark allocator generated');
      }
      await refreshAssignment?.();

      // (Complete)
      setStep(6);
      message.success('Starter pack complete. You’re ready to proceed.');
    } catch (e: any) {
      setErr(e?.message || 'Quick start failed');
      message.error(e?.message || 'Quick start failed');
    } finally {
      setRunning(false);
    }
  };

  // ---------- Manual: save config, then advance to Files step ----------
  const startManualSetup = async () => {
    try {
      await saveConfigFromDefaults();
      onManual?.(); // parent goes to Files & Resources
      message.success('Configuration saved — continue to Files & Resources.');
    } catch (e: any) {
      message.error(e?.message || 'Failed to start manual setup');
    }
  };

  const stepStatus = err ? 'error' : uiStep === 3 ? 'finish' : 'process';
  const renderIcon = (idx: number) => {
    if (err && idx === uiStep) return <CloseCircleOutlined />;
    if (running && idx === uiStep) return <LoadingOutlined />;
    if (idx < uiStep) return <CheckCircleOutlined />;
    return simpleSteps[idx]?.baseIcon;
  };

  const disabled = running || !assignmentId;

  // Optional tiny label for the current sub-stage
  const subLabel =
    step <= 0
      ? 'Saving configuration…'
      : step <= 3
        ? 'Preparing files & tasks…'
        : step <= 5
          ? 'Generating outputs…'
          : 'All done!';

  return (
    <div className="!space-y-6 !w-full">
      {/* Header */}
      <Card className="bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800 rounded-xl">
        <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
          <div>
            <Title level={3} className="!mb-1 !text-gray-900 dark:!text-gray-100">
              Set up your assignment
            </Title>
            <Paragraph type="secondary" className="!mb-0">
              Choose a <b>language</b> and <b>submission mode</b>. Use the Starter Pack for a
              one-click scaffold, or go manual to upload files yourself.
            </Paragraph>
          </div>

          {/* Selectors */}
          <Space size="large" wrap className="mt-2 md:mt-0">
            <div className="flex items-center gap-2">
              <Text strong>Language</Text>
              <Segmented
                value={language}
                onChange={(val) => setLanguage(val as Language)}
                options={[
                  {
                    label: (
                      <Space>
                        <CoffeeOutlined /> Java
                      </Space>
                    ),
                    value: 'java',
                  },
                  {
                    label: (
                      <Space>
                        <CodeOutlined /> C++
                      </Space>
                    ),
                    value: 'cpp',
                  },
                ]}
              />
            </div>

            <div className="flex items-center gap-2">
              <Text strong>Submission</Text>
              <Tooltip title="Impacts which steps and validations are relevant.">
                <InfoCircleOutlined />
              </Tooltip>
              <Segmented
                value={submissionMode}
                onChange={(val) => setSubmissionMode(val as SubmissionMode)}
                options={[
                  {
                    label: (
                      <Space>
                        <CloudUploadOutlined /> Manual
                      </Space>
                    ),
                    value: 'manual',
                  },
                  {
                    label: (
                      <Space>
                        <ThunderboltOutlined /> GATLAM
                      </Space>
                    ),
                    value: 'gatlam',
                  },
                ]}
              />
            </div>
          </Space>
        </div>

        {/* Current selection summary */}
        <Divider className="!my-4" />
        <div className="flex flex-wrap items-center gap-2">
          <Text type="secondary">Selected:</Text>
          <Tag className="!m-0" color="blue">
            {language.toUpperCase()}
          </Tag>
          <Tag className="!m-0" color={submissionMode === 'gatlam' ? 'magenta' : 'green'}>
            {submissionMode.toUpperCase()}
          </Tag>
        </div>
      </Card>

      {/* Main grid */}
      <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
        {/* Left: Starter Pack (wider) */}
        <Card
          title={
            <Space>
              <RocketOutlined /> <span>Starter Pack</span>
            </Space>
          }
          className="xl:col-span-2 bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800 rounded-xl"
          extra={
            <Button
              type="primary"
              icon={<ThunderboltOutlined />}
              onClick={runQuickStart}
              loading={running}
              disabled={disabled}
            >
              Run Starter Pack
            </Button>
          }
        >
          <Steps
            className="!w-full"
            size="small"
            current={uiStep}
            status={stepStatus as any}
            items={simpleSteps.map((s, i) => ({
              title: s.title,
              icon: renderIcon(i),
            }))}
          />

          <div className="mt-3">
            <Text type="secondary" className="text-xs block">
              {subLabel}
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
        </Card>

        {/* Right: Manual Setup */}
        <Card
          title={
            <Space>
              <ToolOutlined /> <span>Manual Setup</span>
            </Space>
          }
          className="bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800 rounded-xl"
        >
          <Paragraph className="!mb-3" type="secondary">
            Prefer to manage files and tasks yourself? We’ll still apply your language and
            submission mode to the config before you continue.
          </Paragraph>

          <ul className="list-disc pl-5 text-sm text-gray-600 dark:text-gray-400 space-y-1">
            <li>Save config with your selections</li>
            <li>Upload main/memo/makefile</li>
            <li>Define or tweak tasks</li>
            <li>Generate memo output &amp; mark allocator</li>
          </ul>

          <div className="pt-4">
            <Button onClick={startManualSetup} block disabled={disabled}>
              Continue with Manual Setup
            </Button>
          </div>
        </Card>
      </div>
    </div>
  );
};

export default StepWelcome;
