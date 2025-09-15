import React, { useMemo } from 'react';
import { Card, Typography, Divider, Descriptions } from 'antd';
import { Pie } from '@ant-design/plots';
import type { PieConfig } from '@ant-design/plots';
import { scaleColor } from '@/utils/color';
import { useUI } from '@/context/UIContext';
import { useTheme } from '@/context/ThemeContext';
import type { AssignmentStats } from '@/types/modules/assignments';

const { Title, Text } = Typography;

type Props = {
  className?: string;
  stats: AssignmentStats | null;
  titleExtra?: React.ReactNode;
};

type PassFailDatum = { type: 'Passed' | 'Failed'; value: number };

const SubmissionStatistics: React.FC<Props> = ({ className, stats, titleExtra }) => {
  const { isXxl } = useUI();
  const { isDarkMode } = useTheme();

  if (!stats || !Number.isFinite(stats.total)) {
    return (
      <Card
        className={className}
        title={
          <Title level={5} className="!mb-0">
            Submission Statistics
          </Title>
        }
      >
        <div className="text-sm text-gray-500">No statistics available.</div>
      </Card>
    );
  }

  const totalMarks = stats.total_marks ?? 0;
  const numSubmissions = stats.total ?? 0;
  const numStudentsSubmitted = stats.num_students_submitted ?? 0;
  const numPassed = stats.num_passed ?? 0;
  const numFailed = stats.num_failed ?? 0;
  const numFullMarks = stats.num_full_marks ?? 0;

  const passFailData = useMemo<PassFailDatum[]>(
    () => [
      { type: 'Passed', value: numPassed },
      { type: 'Failed', value: numFailed },
    ],
    [numPassed, numFailed],
  );

  const PASS = '#16a34a'; // green
  const FAIL = '#ef4444'; // red

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
          position: 'right',
          layout: { justifyContent: 'center' },
          itemMarker: 'circle',
          label: {
            // 👇 force legend text color based on theme
            style: {
              fill: isDarkMode ? '#fff' : '#000',
              fontSize: 13,
              fontWeight: 500,
            },
          },
        },
      } as any,
      radius: 0.85,
      autoFit: true,
      appendPadding: 4,
      label: {
        text: (d: PassFailDatum) => String(d?.value ?? 0),
        position: 'inside',
        fontWeight: 600,
        fill: '#fff',
        fontSize: 12,
      },
      style: {
        lineWidth: 0,
        stroke: 'transparent',
      },
      tooltip: false,
      interactions: [{ type: 'element-active' }],
      state: {
        active: { style: { lineWidth: 1, stroke: 'rgba(0,0,0,0.15)' } },
      },
    }),
    [passFailData, isDarkMode],
  );

  const renderPercent = (value: number, suffix = '%') => (
    <span style={{ color: scaleColor(value, 'red-green'), fontWeight: 500 }}>
      {value.toFixed(1)}
      {suffix}
    </span>
  );

  return (
    <Card
      className={className}
      title={
        <div className="flex items-center justify-between">
          <Title level={5} className="!mb-0">
            Submission Statistics
          </Title>
          {titleExtra}
        </div>
      }
    >
      {/* Summary stats */}
      <Descriptions
        bordered
        size="small"
        column={{ xs: 1, sm: 1, md: 2, lg: 2, xl: 2, xxl: 2 }}
        labelStyle={{ fontWeight: 500 }}
      >
        <Descriptions.Item label="Total Marks">{totalMarks}</Descriptions.Item>
        <Descriptions.Item label="Number of Submissions">{numSubmissions}</Descriptions.Item>
        <Descriptions.Item label="Students Submitted">{numStudentsSubmitted}</Descriptions.Item>
        <Descriptions.Item label="Students Passed">{numPassed}</Descriptions.Item>
        <Descriptions.Item label="Students Failed">{numFailed}</Descriptions.Item>
        <Descriptions.Item label="Full Marks">{numFullMarks}</Descriptions.Item>
      </Descriptions>

      <Divider className="!my-3" />

      {/* Distribution KPIs */}
      <Descriptions
        bordered
        size="small"
        column={{ xs: 1, sm: 1, md: 2, lg: 2, xl: 2, xxl: 2 }}
        labelStyle={{ fontWeight: 500 }}
      >
        <Descriptions.Item label="Average Mark">{renderPercent(stats.avg_mark)}</Descriptions.Item>
        <Descriptions.Item label="Median">{renderPercent(stats.median)}</Descriptions.Item>
        <Descriptions.Item label="75th Percentile">{renderPercent(stats.p75)}</Descriptions.Item>
        <Descriptions.Item label="Std Dev">{stats.stddev.toFixed(1)}</Descriptions.Item>
        <Descriptions.Item label="Highest Mark">{renderPercent(stats.best)}</Descriptions.Item>
        <Descriptions.Item label="Lowest Mark">{renderPercent(stats.worst)}</Descriptions.Item>
      </Descriptions>

      {/* Passed vs Failed Pie — hidden on mobile */}
      {isXxl && (
        <>
          <div className="mt-3">
            <Text strong>Passed vs Failed</Text>
            <div style={{ minHeight: 260 }}>
              <Pie {...passFailConfig} />
            </div>
          </div>
        </>
      )}

      <Divider className="!my-3" />

      {/* Other flags */}
      <Descriptions
        bordered
        size="small"
        column={{ xs: 1, sm: 1, md: 2, lg: 2, xl: 2, xxl: 2 }}
        labelStyle={{ fontWeight: 500 }}
      >
        <Descriptions.Item label="Late">{stats.late}</Descriptions.Item>
        <Descriptions.Item label="On Time">{stats.on_time}</Descriptions.Item>
        <Descriptions.Item label="Ignored">{stats.ignored}</Descriptions.Item>
      </Descriptions>
    </Card>
  );
};

export default SubmissionStatistics;
