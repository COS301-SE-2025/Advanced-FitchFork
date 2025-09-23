// src/pages/modules/assignments/AssignmentStatisticsPage.tsx
import { useEffect, useMemo } from 'react';
import { Button, Card, Col, Descriptions, Divider, Row, Space, Statistic, Typography } from 'antd';
import { ReloadOutlined } from '@ant-design/icons';
import { Pie } from '@ant-design/plots';
import type { PieConfig } from '@ant-design/plots';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { useViewSlot } from '@/context/ViewSlotContext';
import { useTheme } from '@/context/ThemeContext';
import { useUI } from '@/context/UIContext';
import { scaleColor } from '@/utils/color';

const { Title, Text } = Typography;

type Props = { className?: string };

type PassFailDatum = { type: 'Passed' | 'Failed'; value: number };

export default function AssignmentStatisticsPage({ className }: Props) {
  const module = useModule();
  const { isDarkMode } = useTheme();
  const { isXxl } = useUI();
  const { assignment, assignmentStats: stats, refreshAssignmentStats } = useAssignment();
  const { setValue } = useViewSlot();

  // Page title in the slot
  useEffect(() => {
    setValue(
      <Typography.Text
        className="text-base font-medium text-gray-900 dark:text-gray-100 truncate"
        title="Statistics"
      >
        Statistics
      </Typography.Text>,
    );
  }, [setValue]);

  // Fetch/refresh stats when landing or when IDs change
  useEffect(() => {
    void refreshAssignmentStats?.();
  }, [module.id, assignment.id, refreshAssignmentStats]);

  const safe = (n?: number, d = 0) => (Number.isFinite(n as number) ? (n as number) : d);

  const totalMarks = safe(stats?.total_marks);
  const numSubmissions = safe(stats?.total);
  const numStudentsSubmitted = safe(stats?.num_students_submitted);
  const numPassed = safe(stats?.num_passed);
  const numFailed = safe(stats?.num_failed);
  const numFullMarks = safe(stats?.num_full_marks);
  const avg = safe(stats?.avg_mark);
  const median = safe(stats?.median);
  const p75 = safe(stats?.p75);
  const stddev = safe(stats?.stddev);
  const best = safe(stats?.best);
  const worst = safe(stats?.worst);
  const late = safe(stats?.late);
  const onTime = safe(stats?.on_time);
  const ignored = safe(stats?.ignored);

  const passFailData = useMemo<PassFailDatum[]>(
    () => [
      { type: 'Passed', value: numPassed },
      { type: 'Failed', value: numFailed },
    ],
    [numPassed, numFailed],
  );

  const PASS = '#16a34a';
  const FAIL = '#ef4444';

  const passFailConfig: PieConfig = useMemo(
    () => ({
      theme: isDarkMode ? 'dark' : 'light',
      data: passFailData,
      angleField: 'value',
      colorField: 'type',
      scale: {
        color: {
          domain: ['Passed', 'Failed'],
          range: [PASS, FAIL],
        },
      },
      legend: {
        color: {
          position: 'bottom',
          layout: { justifyContent: 'center' },
          itemMarker: 'circle',
          label: {
            style: {
              fill: isDarkMode ? '#fff' : '#000',
              fontSize: 13,
              fontWeight: 500,
            },
          },
        },
      } as any,
      radius: 0.9,
      height: 300, // keep the chart visible/consistent
      autoFit: true,
      appendPadding: 8,
      label: {
        text: (d: PassFailDatum) => String(d?.value ?? 0),
        position: 'inside',
        fontWeight: 600,
        fill: '#fff',
        fontSize: 12,
      },
      style: { lineWidth: 0, stroke: 'transparent' },
      tooltip: false,
      interactions: [{ type: 'element-active' }],
      state: { active: { style: { lineWidth: 1, stroke: 'rgba(0,0,0,0.15)' } } },
    }),
    [passFailData, isDarkMode],
  );

  const renderPercent = (value: number, suffix = '%') => (
    <span style={{ color: scaleColor(value, 'red-green'), fontWeight: 600 }}>
      {value.toFixed(1)}
      {suffix}
    </span>
  );

  return (
    <div className={`h-full flex flex-col ${className ?? ''}`}>
      {/* Page header */}
      <div className="flex items-center justify-between mb-3">
        <Title level={4} className="!mb-0">
          Submission Statistics
        </Title>
        <Space>
          <Button icon={<ReloadOutlined />} onClick={() => void refreshAssignmentStats?.()}>
            Refresh
          </Button>
        </Space>
      </div>

      {/* Content */}
      <div className="flex-1 min-h-0">
        <Row gutter={[16, 16]}>
          {/* LEFT: KPI cards + distribution + flags */}
          <Col xs={24} xl={14} xxl={16}>
            {/* KPI strip */}
            <Row gutter={[16, 16]}>
              <Col xs={12} md={8}>
                <Card size="small">
                  <Statistic title="Submissions" value={numSubmissions} />
                </Card>
              </Col>
              <Col xs={12} md={8}>
                <Card size="small">
                  <Statistic title="Students Submitted" value={numStudentsSubmitted} />
                </Card>
              </Col>
              <Col xs={12} md={8}>
                <Card size="small">
                  <Statistic title="Full Marks" value={numFullMarks} />
                </Card>
              </Col>

              <Col xs={12} md={8}>
                <Card size="small">
                  <Statistic title="Passed" value={numPassed} valueStyle={{ color: PASS }} />
                </Card>
              </Col>
              <Col xs={12} md={8}>
                <Card size="small">
                  <Statistic title="Failed" value={numFailed} valueStyle={{ color: FAIL }} />
                </Card>
              </Col>
              <Col xs={12} md={8}>
                <Card size="small">
                  <Statistic title="Total Marks (Σ)" value={totalMarks} />
                </Card>
              </Col>
            </Row>

            <Card className="!mt-4" size="small" title="Distribution">
              <Descriptions
                bordered
                size="small"
                column={{ xs: 1, sm: 1, md: 2, lg: 2, xl: 2, xxl: 3 }}
                labelStyle={{ fontWeight: 500 }}
              >
                <Descriptions.Item label="Average">{renderPercent(avg)}</Descriptions.Item>
                <Descriptions.Item label="Median">{renderPercent(median)}</Descriptions.Item>
                <Descriptions.Item label="75th Percentile">{renderPercent(p75)}</Descriptions.Item>
                <Descriptions.Item label="Std Dev">{stddev.toFixed(1)}</Descriptions.Item>
                <Descriptions.Item label="Highest">{renderPercent(best)}</Descriptions.Item>
                <Descriptions.Item label="Lowest">{renderPercent(worst)}</Descriptions.Item>
              </Descriptions>
            </Card>

            <Card className="!mt-4" size="small" title="Timing & Flags">
              <Descriptions
                bordered
                size="small"
                column={{ xs: 1, sm: 1, md: 2, lg: 2, xl: 2, xxl: 3 }}
                labelStyle={{ fontWeight: 500 }}
              >
                <Descriptions.Item label="Late">{late}</Descriptions.Item>
                <Descriptions.Item label="On Time">{onTime}</Descriptions.Item>
                <Descriptions.Item label="Ignored">{ignored}</Descriptions.Item>
              </Descriptions>
            </Card>
          </Col>

          {/* RIGHT: Chart panel */}
          <Col xs={24} xl={10} xxl={8}>
            <Card
              size="small"
              title={
                <div className="flex items-center justify-between">
                  <span>Passed vs Failed</span>
                  {/* On XXL screens you asked previously to hide charts; here we keep it always visible.
                      If you want to hide on smaller screens, wrap with {isXxl && (...)} */}
                  {isXxl ? null : null}
                </div>
              }
            >
              <div style={{ minHeight: 300 }}>
                <Pie {...passFailConfig} />
              </div>
              <Divider className="!my-3" />
              <Text type="secondary">
                This chart reflects unique submissions counted by pass/fail outcomes. Ignored items
                are still included here unless your backend excludes them from the aggregated stats.
              </Text>
            </Card>
          </Col>
        </Row>
      </div>
    </div>
  );
}
