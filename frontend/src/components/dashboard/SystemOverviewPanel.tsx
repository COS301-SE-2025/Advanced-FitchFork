import { useState, useEffect } from 'react';
import { Segmented, Typography, Button, Tag } from 'antd';
import {
  DashboardOutlined,
  ClockCircleOutlined,
  BugOutlined,
  DatabaseOutlined,
  ReloadOutlined,
  BarChartOutlined,
  FileTextOutlined,
  FileSearchOutlined,
  CalendarOutlined,
} from '@ant-design/icons';
import { Gauge, gaugeClasses } from '@mui/x-charts/Gauge';
import { LineChart } from '@mui/x-charts';

const { Text, Title } = Typography;

const usageLabels = ['CPU', 'RAM', 'Disk', 'Net', 'Queue'];

const getColor = (value: number) => {
  const clamped = Math.max(0, Math.min(100, value));
  const r = clamped < 50 ? (clamped / 50) * 255 : 255;
  const g = clamped < 50 ? 255 : 255 - ((clamped - 50) / 50) * 255;

  // Slightly darken all colors (90% brightness)
  const factor = 0.9;
  return `rgb(${Math.round(r * factor)}, ${Math.round(g * factor)}, 0)`;
};

const SystemOverviewPanel = () => {
  const [view, setView] = useState<'Performance' | 'Summary' | 'Logs'>('Performance');
  const [range, setRange] = useState<'now' | 'day' | 'week' | 'month'>('now');
  const [usageData, setUsageData] = useState([62, 75, 45, 30, 12]);
  const [isMobile, setIsMobile] = useState(false);

  const timeLabels = Array.from({ length: 24 }, (_, i) => `${i.toString().padStart(2, '0')}:00`);

  const dayUsageData = {
    CPU: Array.from({ length: 24 }, () => Math.floor(Math.random() * 100)),
    RAM: Array.from({ length: 24 }, () => Math.floor(Math.random() * 100)),
    Disk: Array.from({ length: 24 }, () => Math.floor(Math.random() * 100)),
  };

  const weekLabels = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];

  const weekUsageData = {
    CPU: Array.from({ length: 7 }, () => Math.floor(Math.random() * 100)),
    RAM: Array.from({ length: 7 }, () => Math.floor(Math.random() * 100)),
    Disk: Array.from({ length: 7 }, () => Math.floor(Math.random() * 100)),
  };

  const monthLabels = Array.from({ length: 30 }, (_, i) => `Day ${i + 1}`);

  const monthUsageData = {
    CPU: Array.from({ length: 30 }, () => Math.floor(Math.random() * 100)),
    RAM: Array.from({ length: 30 }, () => Math.floor(Math.random() * 100)),
    Disk: Array.from({ length: 30 }, () => Math.floor(Math.random() * 100)),
  };

  useEffect(() => {
    const handleResize = () => setIsMobile(window.innerWidth < 576);
    handleResize();
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  const viewOptions = isMobile
    ? [
        { value: 'Performance', icon: <BarChartOutlined /> },
        { value: 'Summary', icon: <FileTextOutlined /> },
        { value: 'Logs', icon: <FileSearchOutlined /> },
      ]
    : ['Performance', 'Summary', 'Logs'];

  const handleRandomize = () => {
    const randomData = Array.from({ length: usageLabels.length }, () =>
      Math.floor(Math.random() * 100),
    );
    setUsageData(randomData);
  };

  return (
    <div className="bg-white dark:bg-gray-950 p-6 rounded-lg border border-gray-200 dark:border-gray-700">
      <div className="flex flex-wrap items-center justify-between gap-2 mb-4">
        <div className="flex flex-col gap-2">
          <Title level={3}>System Overview</Title>
        </div>
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
            onChange={(val) => setView(val as typeof view)}
            options={viewOptions}
          />
        </div>
      </div>

      {view === 'Performance' && (
        <Segmented
          size="middle"
          block
          value={range}
          onChange={(val) => setRange(val as typeof range)}
          options={[
            { label: 'Now', value: 'now', icon: <CalendarOutlined /> },
            { label: 'Day', value: 'day', icon: <CalendarOutlined /> },
            { label: 'Week', value: 'week', icon: <CalendarOutlined /> },
            { label: 'Month', value: 'month', icon: <CalendarOutlined /> },
          ]}
          className="!mb-6"
        />
      )}

      {view === 'Performance' && range === 'now' && (
        <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5 gap-4 mb-4">
          {usageLabels.map((label, index) => {
            const value = usageData[index];
            const color = getColor(value);

            return (
              <div
                key={label}
                className="flex flex-col items-center justify-center text-center border border-gray-200 dark:border-gray-700 rounded-xl p-4 bg-white dark:bg-gray-900"
              >
                <Tag color="default" className="mt-2 text-sm font-medium">
                  {label}
                </Tag>
                <Gauge
                  width={160}
                  height={160}
                  value={value}
                  valueMax={100}
                  startAngle={-90}
                  endAngle={90}
                  cornerRadius="50%"
                  sx={{
                    [`& .${gaugeClasses.valueArc}`]: { fill: color },
                    [`& .${gaugeClasses.referenceArc}`]: { fill: `${color}22` },
                  }}
                />
              </div>
            );
          })}
        </div>
      )}

      {view === 'Performance' && range === 'day' && (
        <div className="overflow-x-auto">
          <LineChart
            height={300}
            xAxis={[{ scaleType: 'point', data: timeLabels }]}
            yAxis={[{ min: 0, max: 100 }]}
            series={[
              { data: dayUsageData.CPU, label: 'CPU', color: '#f87171' },
              { data: dayUsageData.RAM, label: 'RAM', color: '#60a5fa' },
              { data: dayUsageData.Disk, label: 'Disk', color: '#fbbf24' },
            ]}
          />
        </div>
      )}

      {view === 'Performance' && range === 'week' && (
        <div className="overflow-x-auto">
          <LineChart
            height={300}
            xAxis={[{ scaleType: 'point', data: weekLabels }]}
            yAxis={[{ min: 0, max: 100 }]}
            series={[
              { data: weekUsageData.CPU, label: 'CPU', color: '#f87171' },
              { data: weekUsageData.RAM, label: 'RAM', color: '#60a5fa' },
              { data: weekUsageData.Disk, label: 'Disk', color: '#fbbf24' },
            ]}
          />
        </div>
      )}

      {view === 'Performance' && range === 'month' && (
        <div className="overflow-x-auto">
          <LineChart
            height={300}
            xAxis={[{ scaleType: 'point', data: monthLabels }]}
            yAxis={[{ min: 0, max: 100 }]}
            series={[
              { data: monthUsageData.CPU, label: 'CPU', color: '#f87171' },
              { data: monthUsageData.RAM, label: 'RAM', color: '#60a5fa' },
              { data: monthUsageData.Disk, label: 'Disk', color: '#fbbf24' },
            ]}
          />
        </div>
      )}

      {view === 'Summary' && (
        <div className="space-y-3 text-sm text-gray-700 dark:text-gray-300">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <DashboardOutlined className="text-blue-600" />
              <Text>CPU / RAM Load</Text>
            </div>
            <Tag color="blue">{`${usageData[0]}% / ${usageData[1]}%`}</Tag>
          </div>

          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <ClockCircleOutlined className="text-purple-600" />
              <Text>Avg Marking Time</Text>
            </div>
            <Tag color="purple">1.4s</Tag>
          </div>

          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <BugOutlined className="text-red-600" />
              <Text>Recent Errors</Text>
            </div>
            <Tag color="red">3</Tag>
          </div>

          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <DatabaseOutlined className="text-green-600" />
              <Text>Service Uptime</Text>
            </div>
            <Tag color="green">100%</Tag>
          </div>
        </div>
      )}

      {view === 'Logs' && (
        <div className="space-y-2 text-xs bg-gray-100 dark:bg-gray-900 text-gray-800 dark:text-gray-100 rounded-md overflow-auto max-h-64 p-3 border border-gray-300 dark:border-gray-700">
          {`[2025-06-29 09:12:43] INFO  system: Restarted marking service.
[2025-06-29 09:12:45] WARN  auth: Invalid login attempt by user_id=45.
[2025-06-29 09:13:00] INFO  db: Assignment table migration completed.
[2025-06-29 09:14:22] ERROR grader: Task 213 crashed on module COS332.
[2025-06-29 09:15:01] INFO  user_mgmt: User #324 approved by admin.
[2025-06-29 09:16:10] INFO  container: Pulled new grading container image.
[2025-06-29 09:16:48] WARN  modules: High failure rate in COS344 submission.
[2025-06-29 09:17:23] INFO  server: System uptime is 99.99%.`
            .trim()
            .split('\n')
            .map((line, idx) => {
              const match = line.match(/^\[(.*?)\]\s+(\w+)\s+(.*?):\s+(.*)$/);
              if (!match) return <div key={idx}>{line}</div>;

              const [, timestamp, level, source, message] = match;
              const levelColor =
                level === 'ERROR'
                  ? 'red'
                  : level === 'WARN'
                    ? 'orange'
                    : level === 'INFO'
                      ? 'blue'
                      : 'default';

              return (
                <div key={idx} className="flex gap-2 items-start leading-relaxed">
                  <span className="text-gray-500 dark:text-gray-400 min-w-[145px]">
                    {timestamp}
                  </span>
                  <Tag color={levelColor} className="px-2 min-w-[50px] text-center">
                    {level}
                  </Tag>
                  <span className="font-semibold text-gray-700 dark:text-gray-100">{source}:</span>
                  <span className="text-gray-800 dark:text-gray-200">{message}</span>
                </div>
              );
            })}
        </div>
      )}
    </div>
  );
};

export default SystemOverviewPanel;
