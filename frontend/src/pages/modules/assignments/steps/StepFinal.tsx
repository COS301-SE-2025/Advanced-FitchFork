// StepFinal.tsx
import { Alert, Card, Row, Col, Space, Typography, Tag, Divider } from 'antd';
import { useAssignmentSetup } from '@/context/AssignmentSetupContext';
import { LANGUAGE_LABELS } from '@/types/modules/assignments/config';
import type { SubmissionMode } from '@/types/modules/assignments/config';

const { Title, Text } = Typography;

const StepFinal = () => {
  const { assignment, config, readiness } = useAssignmentSetup();

  const displayName =
    assignment?.assignment?.name ?? `Assignment #${assignment?.assignment?.id ?? ''}`;

  const langKey = config?.project?.language;
  const languageLabel = langKey ? (LANGUAGE_LABELS[langKey] ?? langKey) : '—';

  const mode = (readiness?.submission_mode ?? config?.project?.submission_mode) as
    | SubmissionMode
    | undefined;
  const modeLabel = mode ? mode.toUpperCase() : '—';
  const modeColor = mode === 'gatlam' ? 'magenta' : 'green';

  // a concise sentence about which artifacts mattered for this setup
  const filesSentence =
    mode === 'gatlam'
      ? 'Interpreter, Memo, and Makefile are in place.'
      : 'Main files, Memo, and Makefile are in place.';

  return (
    <div className="max-w-5xl mx-auto">
      {/* Compact success banner */}
      <Alert
        type="success"
        showIcon
        message="Setup complete"
        description="Everything required is in place. You can close this wizard or fine-tune details later."
        className="rounded-lg !mb-4"
      />

      {/* Summary */}
      <Card size="small" className="rounded-xl">
        <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-3">
          <div>
            <Title level={4} className="!mb-0">
              {displayName}
            </Title>
            <Text type="secondary">Ready for students once you open/publish it.</Text>
          </div>

          <Space size="large" wrap>
            <Space size={6}>
              <Text type="secondary">Language:</Text>
              <Tag color="blue" className="!m-0">
                {languageLabel}
              </Tag>
            </Space>
            <Space size={6}>
              <Text type="secondary">Submission:</Text>
              <Tag color={modeColor} className="!m-0">
                {modeLabel}
              </Tag>
            </Space>
          </Space>
        </div>

        <Divider className="!my-8" />

        {/* Minimal, tag-based section statuses (no check icons, no shadows) */}
        <Row gutter={[24, 24]}>
          <Col xs={24} md={8}>
            <Card bordered size="small" className="rounded-lg">
              <Space direction="vertical" size={8} className="w-full">
                <div className="flex items-center justify-between">
                  <Text strong>Configuration</Text>
                  <Tag color="success" className="!m-0">
                    Saved
                  </Tag>
                </div>
                <Text type="secondary">
                  Timeout {config?.execution?.timeout_secs ?? '—'}s · Grading{' '}
                  {config?.marking?.grading_policy ?? '—'}
                </Text>
              </Space>
            </Card>
          </Col>

          <Col xs={24} md={8}>
            <Card bordered size="small" className="rounded-lg">
              <Space direction="vertical" size={8} className="w-full">
                <div className="flex items-center justify-between">
                  <Text strong>Files & Resources</Text>
                  <Tag color="success" className="!m-0">
                    Uploaded
                  </Tag>
                </div>
                <Text type="secondary">{filesSentence}</Text>
              </Space>
            </Card>
          </Col>

          <Col xs={24} md={8}>
            <Card bordered size="small" className="rounded-lg">
              <Space direction="vertical" size={8} className="w-full">
                <div className="flex items-center justify-between">
                  <Text strong>Tasks & Outputs</Text>
                  <Tag color="success" className="!m-0">
                    Ready
                  </Tag>
                </div>
                <Text type="secondary">Tasks defined; Memo Output & Mark Allocator generated.</Text>
              </Space>
            </Card>
          </Col>
        </Row>

        <Divider className="!my-8" />

        <Space direction="vertical" size={2}>
          <Text type="secondary">You can tweak details later from the assignment page.</Text>
          <Text type="secondary">Open the assignment to let students begin.</Text>
        </Space>
      </Card>
    </div>
  );
};

export default StepFinal;
