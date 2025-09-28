// StepFinal.tsx
import { Card, Space, Tag, Typography } from 'antd';
import { CheckCircleTwoTone, RocketOutlined, EyeOutlined } from '@ant-design/icons';
import { useAssignmentSetup } from '@/context/AssignmentSetupContext';
import { LANGUAGE_LABELS } from '@/types/modules/assignments/config';
import type { SubmissionMode } from '@/types/modules/assignments/config';

const { Title, Text, Paragraph } = Typography;

const StepFinal = () => {
  const { assignment, config, readiness } = useAssignmentSetup();

  const displayName =
    assignment?.assignment?.name ?? `Assignment #${assignment?.assignment?.id ?? ''}`;

  const langKey = config?.project?.language;
  const languageLabel = langKey ? (LANGUAGE_LABELS[langKey] ?? langKey) : null;

  const mode = (readiness?.submission_mode ?? config?.project?.submission_mode) as
    | SubmissionMode
    | undefined;

  const modeLabel = mode ? mode.toUpperCase() : null;
  const modeColor =
    mode === 'gatlam'
      ? 'magenta'
      : mode === 'rng'
        ? 'geekblue'
        : mode === 'codecoverage'
          ? 'cyan'
          : 'green';

  return (
    <div className="max-w-3xl mx-auto">
      <Card
        bordered
        className="rounded-2xl border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900"
        bodyStyle={{ padding: 28 }}
      >
        {/* Header */}
        <div className="flex items-start gap-3">
          <CheckCircleTwoTone twoToneColor="#52c41a" style={{ fontSize: 28, lineHeight: 1 }} />
          <div className="flex-1">
            <Title level={3} className="!m-0">
              Setup complete
            </Title>
            <Text type="secondary">
              {displayName} is ready. You can close this wizard now — you can tweak details later.
            </Text>
          </div>
        </div>

        {/* Pills */}
        {(languageLabel || modeLabel) && (
          <div className="mt-4">
            <Space size="small" wrap>
              {languageLabel && (
                <Tag color="blue" className="!m-0 !px-3 !py-1 rounded-full text-[12px]">
                  {languageLabel}
                </Tag>
              )}
              {modeLabel && (
                <Tag color={modeColor} className="!m-0 !px-3 !py-1 rounded-full text-[12px]">
                  {modeLabel}
                </Tag>
              )}
            </Space>
          </div>
        )}

        {/* Divider */}
        <div className="h-px my-6 bg-gray-200 dark:bg-gray-800" />

        {/* Next steps (subtle, bordered) */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <Card
            size="small"
            className="rounded-xl border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900"
          >
            <Space align="start">
              <EyeOutlined className="text-gray-500 dark:text-gray-400 mt-[2px]" />
              <div>
                <Text strong>Review settings</Text>
                <Paragraph type="secondary" className="!m-0 text-[13px]">
                  Double-check language, limits, tasks, and (if applicable) interpreter.
                </Paragraph>
              </div>
            </Space>
          </Card>

          <Card
            size="small"
            bordered
            className="rounded-xl border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900"
            bodyStyle={{ padding: 16 }}
          >
            <Space align="start">
              <RocketOutlined className="text-gray-500 dark:text-gray-400 mt-[2px]" />
              <div>
                <Text strong>Open for students</Text>
                <Paragraph type="secondary" className="!m-0 text-[13px]">
                  Publish the assignment when you’re ready to accept submissions.
                </Paragraph>
              </div>
            </Space>
          </Card>
        </div>
      </Card>
    </div>
  );
};

export default StepFinal;
