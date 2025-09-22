import { useState, useEffect } from 'react';
import {
  Modal,
  Form,
  Checkbox,
  Typography,
  Descriptions,
  Divider,
  Collapse,
  Tag,
  Result,
  Alert,
  Table,
} from 'antd';
import { Link } from 'react-router-dom';
import { message } from '@/utils/message';
import { runHashScan } from '@/services/modules/assignments/plagiarism';
import type { HashScanCollisionGroup, HashScanData } from '@/types/modules/assignments/plagiarism';
import { IdTag } from '../common';

type Props = {
  open: boolean;
  onClose: () => void;
  moduleId: number;
  assignmentId: number;
  onRan?: () => void;
};

const { Text, Paragraph } = Typography;
const { Panel } = Collapse;

// Scoped class used to style thin scrollbars inside the Antd table body/content
const SCROLL_CLASS = 'hash-scan-scroll';

export default function HashScanModal({ open, onClose, moduleId, assignmentId, onRan }: Props) {
  const [form] = Form.useForm<{ create_cases: boolean }>();
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<HashScanData | null>(null);

  // Reset when closed or assignment changes
  useEffect(() => {
    if (!open) {
      form.resetFields();
      setResult(null);
    }
  }, [open, moduleId, assignmentId, form]);

  const onSubmit = async () => {
    if (loading) return;
    try {
      const vals = await form.validateFields();
      setLoading(true);
      const res = await runHashScan(moduleId, assignmentId, { create_cases: !!vals.create_cases });
      if (!res.success) {
        message.error(res.message || 'Hash scan failed');
        setResult(null);
        return;
      }
      setResult(res.data);
      const created = res.data.cases?.created?.length ?? 0;
      const skipped = res.data.cases?.skipped_existing ?? 0;
      const groups = res.data.group_count ?? res.data.groups?.length ?? 0;
      message.success(
        `Hash scan complete: ${groups} group(s), ${created} case(s) created, ${skipped} skipped.`,
      );
      onRan?.();
    } finally {
      setLoading(false);
    }
  };

  // ── Group render helpers ──────────────────────────────────────────────────
  const renderGroupHeader = (g: HashScanCollisionGroup) => {
    const count = g.submissions.length;
    return (
      <div className="flex items-center gap-3 flex-wrap">
        <code className="text-xs font-mono max-w-[560px] truncate block">{g.file_hash}</code>
        <Tag>
          {count} {count === 1 ? 'submission' : 'submissions'}
        </Tag>
      </div>
    );
  };

  const renderGroupBody = (g: HashScanCollisionGroup) => {
    // Table data sorted by created_at desc (latest first)
    const data = [...g.submissions].sort(
      (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime(),
    );

    const columns = [
      {
        title: 'Submission',
        dataIndex: 'submission_id',
        key: 'submission',
        width: 140,
        fixed: 'left' as const,
        render: (_: any, s: HashScanCollisionGroup['submissions'][number]) => {
          const to = `/modules/${moduleId}/assignments/${assignmentId}/submissions/${s.submission_id}`;
          return (
            <Link to={to} className="font-medium">
              #{s.submission_id}
            </Link>
          );
        },
      },
      {
        title: 'User',
        dataIndex: 'user_id',
        key: 'user_id',
        width: 120,
        fixed: 'left' as const,
        render: (user_id: number) => <IdTag id={user_id} />,
      },
      {
        title: 'Attempt',
        dataIndex: 'attempt',
        key: 'attempt',
        width: 100,
        align: 'center' as const,
      },
      {
        title: 'Filename',
        dataIndex: 'filename',
        key: 'filename',
        width: 260,
        render: (name: string) => (
          <span className="font-mono max-w-[260px] inline-block truncate align-middle" title={name}>
            {name}
          </span>
        ),
      },
      {
        title: 'Created',
        dataIndex: 'created_at',
        key: 'created_at',
        width: 200,
        fixed: 'right' as const,
        render: (iso: string) => new Date(iso).toLocaleString(),
        sorter: (a: any, b: any) =>
          new Date(a.created_at).getTime() - new Date(b.created_at).getTime(),
        defaultSortOrder: 'descend' as const,
      },
    ];

    return (
      <div className="space-y-2">
        <div className="text-xs text-gray-500 dark:text-gray-400 break-all">
          Full hash: <code className="font-mono">{g.file_hash}</code>
        </div>
        <Table
          className={SCROLL_CLASS}
          dataSource={data}
          columns={columns as any}
          size="small"
          bordered
          pagination={false}
          rowKey={(s) => String(s.submission_id)}
          // horizontal + vertical scroll, sticky header, fixed columns honored
          scroll={{ x: 'max-content', y: 260 }}
          sticky
        />
      </div>
    );
  };

  const renderResult = () => {
    if (!result) return null;

    const groups = result.groups ?? [];
    const created = result.cases?.created?.length ?? 0;
    const skipped = result.cases?.skipped_existing ?? 0;

    return (
      <>
        <Divider className="!my-3" />
        <Descriptions
          size="small"
          column={1}
          bordered
          className="rounded-md"
          items={[
            { key: 'policy', label: 'Policy Used', children: <b>{result.policy_used}</b> },
            { key: 'groups', label: 'Collision Groups', children: <b>{result.group_count}</b> },
            { key: 'created', label: 'Cases Created', children: <b>{created}</b> },
            { key: 'skipped', label: 'Skipped Existing', children: <b>{skipped}</b> },
          ]}
        />

        {groups.length === 0 ? (
          <Result
            status="success"
            title="No hash collisions found"
            subTitle="No groups with identical file hashes were detected."
            className="!p-0"
          />
        ) : (
          <>
            <Text type="secondary" className="block mt-2 mb-1">
              Collision groups
            </Text>
            <Collapse accordion>
              {groups.map((g) => (
                <Panel header={renderGroupHeader(g)} key={g.file_hash}>
                  {renderGroupBody(g)}
                </Panel>
              ))}
            </Collapse>
          </>
        )}
      </>
    );
  };

  const configLink = `/modules/${moduleId}/assignments/${assignmentId}/config/marking`;

  return (
    <Modal
      open={open}
      title="Run Hash Scan"
      onCancel={() => onClose()}
      okText={result ? 'Run Again' : 'Run Scan'}
      onOk={onSubmit}
      okButtonProps={{ loading }}
      destroyOnClose
      width={900}
    >
      {/* Scoped scrollbar CSS (no external deps) */}
      <style>
        {`
        /* Scope to our tables only */
        .${SCROLL_CLASS} .ant-table-container .ant-table-body,
        .${SCROLL_CLASS} .ant-table-container .ant-table-content {
          scrollbar-width: thin;              /* Firefox */
          scrollbar-color: #eaeaea transparent;
          scrollbar-gutter: stable;
        }
        .${SCROLL_CLASS} .ant-table-container .ant-table-body::-webkit-scrollbar,
        .${SCROLL_CLASS} .ant-table-container .ant-table-content::-webkit-scrollbar {
          height: 8px;                        /* horizontal bar height */
          width: 8px;                         /* vertical bar width */
        }
        .${SCROLL_CLASS} .ant-table-container .ant-table-body::-webkit-scrollbar-thumb,
        .${SCROLL_CLASS} .ant-table-container .ant-table-content::-webkit-scrollbar-thumb {
          background-color: #eaeaea;
          border-radius: 8px;
        }
        `}
      </style>

      {/* Form (tight spacing) */}
      <Form
        form={form}
        layout="vertical"
        initialValues={{ create_cases: false }}
        disabled={loading}
        className="!mb-2"
      >
        <Form.Item
          name="create_cases"
          valuePropName="checked"
          className="!mb-2"
          tooltip="Create plagiarism cases for each unique pair in a collision group (skips existing/self pairs)."
        >
          <Checkbox>Create cases for collisions</Checkbox>
        </Form.Item>
      </Form>

      {/* Grading policy info with link */}
      <Alert
        type="info"
        showIcon
        className="!mb-2"
        message={
          <span className="text-sm">
            The scan respects the assignment&apos;s <b>grading policy</b> (Best / Last). Only each
            student&apos;s selected submission is considered. Configure it under{' '}
            <Link to={configLink} className="font-medium underline">
              Marking Settings
            </Link>
            .
          </span>
        }
      />

      {/* One-liner helper */}
      <Paragraph type="secondary" className="!text-xs !mt-1 !mb-0">
        Groups selected submissions by SHA-256 and lists groups with <b>2+</b> submissions. Good for
        exact duplicates; complements MOSS similarity checks.
      </Paragraph>

      {renderResult()}
    </Modal>
  );
}
