import { useEffect, useState } from 'react';
import { Segmented, Button, Typography, Tag } from 'antd';
import {
  ReloadOutlined,
  LineChartOutlined,
  FileTextOutlined,
  CalendarOutlined,
} from '@ant-design/icons';
import { LineChart } from '@mui/x-charts';

const { Title } = Typography;

const SubmissionsPanel = () => {
  const [view, setView] = useState<'chart' | 'summary'>('chart');
  const [range, setRange] = useState<'day' | 'week' | 'month'>('day');
  const [data, setData] = useState<number[]>([]);
  const [xAxisLabels, setXAxisLabels] = useState<string[]>([]);
  const [isMobile, setIsMobile] = useState(false);

  const generateRandomData = (count: number) =>
    Array.from({ length: count }, () => Math.floor(Math.random() * 150));

  const getXAxisLabels = (range: 'day' | 'week' | 'month') => {
    if (range === 'day') return Array.from({ length: 24 }, (_, i) => `${i}:00`);
    if (range === 'week') return ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
    return Array.from({ length: 30 }, (_, i) => `${i + 1}`);
  };

  useEffect(() => {
    const handleResize = () => setIsMobile(window.innerWidth < 576);
    handleResize();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  useEffect(() => {
    const labels = getXAxisLabels(range);
    setXAxisLabels(labels);
    setData(generateRandomData(labels.length));
  }, [range]);

  const handleRandomize = () => {
    setData(generateRandomData(xAxisLabels.length));
  };

  const segmentedOptions = isMobile
    ? [
        { value: 'chart', icon: <LineChartOutlined /> },
        { value: 'summary', icon: <FileTextOutlined /> },
      ]
    : [
        { label: 'Chart', value: 'chart' },
        { label: 'Summary', value: 'summary' },
      ];

  return (
    <div className="bg-white dark:bg-gray-950 p-4 rounded-lg border border-gray-200 dark:border-gray-700 h-full">
      <div className="flex flex-wrap items-center justify-between gap-2 mb-4">
        {/* Title + range (on small screens: stacked) */}
        <div className="flex flex-col gap-2">
          <Title level={3}>Submissions Overview</Title>
        </div>

        {/* Controls on right */}
        <div className="flex items-center gap-2">
          <Button
            size="middle"
            icon={<ReloadOutlined />}
            onClick={handleRandomize}
            title="Randomize Data"
          />
          <Segmented
            size="middle"
            value={view}
            onChange={(val) => setView(val as 'chart' | 'summary')}
            options={segmentedOptions}
          />
        </div>
      </div>

      <Segmented
        size="middle"
        value={range}
        onChange={(val) => setRange(val as 'day' | 'week' | 'month')}
        options={[
          { label: 'Day', value: 'day', icon: <CalendarOutlined /> },
          { label: 'Week', value: 'week', icon: <CalendarOutlined /> },
          { label: 'Month', value: 'month', icon: <CalendarOutlined /> },
        ]}
        block
        className="!mb-6"
      />

      {view === 'chart' ? (
        <LineChart
          xAxis={[
            {
              id: 'x',
              data: xAxisLabels,
              scaleType: 'point',
              valueFormatter: (value) => {
                const index = xAxisLabels.indexOf(value);
                if (range === 'day' || range === 'month') {
                  return index % 2 === 0 ? value : '';
                }
                return value;
              },
            },
          ]}
          series={[{ id: 'submissions', data }]}
          height={range === 'month' ? 250 : 220}
          grid={{ horizontal: false, vertical: false }}
          margin={{ top: 10, bottom: 30, left: 10, right: 10 }}
          sx={{
            '.MuiLineElement-root': { strokeWidth: 2 },
            '.MuiChartsAxis-tickLabel': { fontSize: 10 },
          }}
        />
      ) : (
        <div className="grid grid-cols-1 gap-3 text-sm text-gray-700 dark:text-gray-300">
          <div className="flex items-center justify-between">
            <span className="text-gray-600 dark:text-gray-400">Total Submissions:</span>
            <Tag color="blue">{data.reduce((a, b) => a + b, 0)}</Tag>
          </div>

          <div className="flex items-center justify-between">
            <span className="text-gray-600 dark:text-gray-400">Average per Unit:</span>
            <Tag color="purple">{(data.reduce((a, b) => a + b, 0) / data.length).toFixed(1)}</Tag>
          </div>

          <div className="flex items-center justify-between">
            <span className="text-gray-600 dark:text-gray-400">Peak Time:</span>
            <Tag color="geekblue">{xAxisLabels[data.indexOf(Math.max(...data))]}</Tag>
          </div>

          <div className="flex items-center justify-between">
            <span className="text-gray-600 dark:text-gray-400">Success Rate:</span>
            <Tag color="green">{`${(95 + Math.random() * 5).toFixed(1)}%`}</Tag>
          </div>

          <div className="flex items-center justify-between">
            <span className="text-gray-600 dark:text-gray-400">Failures:</span>
            <Tag color="red">{Math.floor(Math.random() * 30)}</Tag>
          </div>
        </div>
      )}
    </div>
  );
};

export default SubmissionsPanel;
