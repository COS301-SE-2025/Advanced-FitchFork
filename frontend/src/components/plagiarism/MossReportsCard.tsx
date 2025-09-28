import { useMemo, useState } from 'react';
import {
  Card,
  Space,
  Button,
  Alert,
  List,
  Typography,
  Popconfirm,
  Modal,
  Descriptions,
  Dropdown,
} from 'antd';
import type { MenuProps } from 'antd';
import dayjs from 'dayjs';
import {
  ExperimentOutlined,
  FileZipOutlined,
  DeleteOutlined,
  EyeOutlined,
  MoreOutlined,
} from '@ant-design/icons';
import { deleteMossReport } from '@/services/modules/assignments/plagiarism/delete';
import { downloadMossArchiveByReport } from '@/services/modules/assignments/plagiarism/get';
import type { MossReport } from '@/types/modules/assignments/plagiarism';
import { message } from '@/utils/message';

type Props = {
  moduleId: number;
  assignmentId: number;
  reports: MossReport[];
  loading?: boolean;
  onOpenRunMoss: () => void;
  onRefresh?: () => void;
};

const MossReportsCard: React.FC<Props> = ({
  moduleId,
  assignmentId,
  reports,
  loading,
  onOpenRunMoss,
  onRefresh,
}) => {
  const latest = useMemo(() => reports?.[0], [reports]);
  const hasArchive = Boolean(latest?.has_archive);

  const [detailsOpen, setDetailsOpen] = useState(false);
  const [selected, setSelected] = useState<MossReport | null>(null);

  const openDetails = (r: MossReport) => {
    setSelected(r);
    setDetailsOpen(true);
  };

  return (
    <>
      <Card
        title="MOSS Reports"
        extra={
          <Space>
            <Button type="primary" icon={<ExperimentOutlined />} onClick={onOpenRunMoss}>
              Run MOSS
            </Button>
            <Button
              icon={<FileZipOutlined />}
              disabled={!hasArchive || !latest}
              onClick={() =>
                latest && downloadMossArchiveByReport(moduleId, assignmentId, latest.id)
              }
            >
              Download ZIP
            </Button>
          </Space>
        }
      >
        {/* Latest summary (always allow open) */}
        {latest ? (
          <Alert
            type="info"
            showIcon
            message={
              <span className="flex flex-wrap items-center gap-x-2">
                <a
                  href={latest.report_url}
                  target="_blank"
                  rel="noreferrer"
                  className="text-blue-600"
                >
                  Open latest report
                </a>
                <Typography.Text type="secondary">
                  • Generated {dayjs(latest.generated_at).fromNow()}
                </Typography.Text>
                {latest.has_archive && latest.archive_generated_at && (
                  <Typography.Text type="secondary">
                    {' '}
                    • Archived {dayjs(latest.archive_generated_at).fromNow()}
                  </Typography.Text>
                )}
              </span>
            }
            description={
              latest.description ? (
                <Typography.Paragraph className="!mb-0" ellipsis={{ rows: 2 }}>
                  {latest.description}
                </Typography.Paragraph>
              ) : undefined
            }
            className="mb-3"
          />
        ) : (
          <Alert
            type="warning"
            showIcon
            className="mb-3"
            message="No MOSS report found"
            description={
              <Typography.Text type="secondary">
                Use <strong>Run MOSS</strong> to generate a report.
              </Typography.Text>
            }
          />
        )}

        {/* Recent reports list */}
        <List<MossReport>
          loading={loading}
          dataSource={reports}
          split
          itemLayout="horizontal"
          locale={{ emptyText: 'No reports yet' }}
          renderItem={(r) => {
            const titleText = (r.description && r.description.trim()) || `Report #${r.id}`;

            // Dropdown items (no url_active gating)
            const items: MenuProps['items'] = [
              {
                key: 'view',
                label: (
                  <span>
                    <EyeOutlined /> View details
                  </span>
                ),
              },
              {
                key: 'download',
                disabled: !r.has_archive,
                label: (
                  <span>
                    <FileZipOutlined /> Download ZIP
                  </span>
                ),
              },
              { type: 'divider' as const },
              {
                key: 'delete',
                label: (
                  <Popconfirm
                    title="Delete this report?"
                    okText="Delete"
                    okButtonProps={{ danger: true }}
                    onConfirm={async () => {
                      const res = await deleteMossReport(moduleId, assignmentId, r.id);
                      if (res.success) {
                        message.success('Report deleted');
                        onRefresh?.();
                      } else {
                        message.error(res.message || 'Failed to delete report');
                      }
                    }}
                  >
                    <span className="text-red-600">
                      <DeleteOutlined /> Delete
                    </span>
                  </Popconfirm>
                ),
              },
            ];

            const onMenuClick: MenuProps['onClick'] = async ({ key }) => {
              switch (key) {
                case 'view':
                  openDetails(r);
                  break;
                case 'download':
                  if (r.has_archive) {
                    await downloadMossArchiveByReport(moduleId, assignmentId, r.id);
                  }
                  break;
              }
            };

            return (
              <List.Item>
                <div className="w-full flex items-center justify-between gap-3">
                  <div className="min-w-0">
                    <Typography.Text className="block" ellipsis={{ tooltip: titleText }}>
                      {titleText}
                    </Typography.Text>
                  </div>

                  <Space size="small">
                    {/* Primary action: Open */}
                    <a href={r.report_url} target="_blank" rel="noreferrer">
                      Open
                    </a>

                    <Dropdown
                      placement="bottomRight"
                      trigger={['click']}
                      menu={{ items, onClick: onMenuClick }}
                    >
                      <Button type="text" icon={<MoreOutlined />} />
                    </Dropdown>
                  </Space>
                </div>
              </List.Item>
            );
          }}
        />
      </Card>

      {/* Details modal */}
      <Modal
        title={selected ? `Report #${selected.id}` : 'Report details'}
        open={detailsOpen}
        onCancel={() => setDetailsOpen(false)}
        footer={<Button onClick={() => setDetailsOpen(false)}>Close</Button>}
      >
        {selected && (
          <Descriptions size="small" column={1} bordered>
            <Descriptions.Item label="Report ID">{selected.id}</Descriptions.Item>
            <Descriptions.Item label="Report URL">
              <a href={selected.report_url} target="_blank" rel="noreferrer">
                {selected.report_url}
              </a>
            </Descriptions.Item>
            {/* Removed URL active row */}
            <Descriptions.Item label="Filter mode">{selected.filter_mode}</Descriptions.Item>
            <Descriptions.Item label="Filter patterns">
              {selected.filter_patterns?.length ? selected.filter_patterns.join(', ') : '—'}
            </Descriptions.Item>
            <Descriptions.Item label="Has archive">
              {selected.has_archive ? 'Yes' : 'No'}
            </Descriptions.Item>
            <Descriptions.Item label="Generated at">
              {dayjs(selected.generated_at).format('YYYY-MM-DD HH:mm:ss')}
            </Descriptions.Item>
            <Descriptions.Item label="Archive generated at">
              {selected.archive_generated_at
                ? dayjs(selected.archive_generated_at).format('YYYY-MM-DD HH:mm:ss')
                : '—'}
            </Descriptions.Item>
            <Descriptions.Item label="Description">{selected.description || '—'}</Descriptions.Item>
          </Descriptions>
        )}
      </Modal>
    </>
  );
};

export default MossReportsCard;
