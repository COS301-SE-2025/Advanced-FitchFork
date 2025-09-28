import { useEffect, useMemo } from 'react';
import { Typography, Space, Card } from 'antd';
import { useHelpToc } from '@/context/HelpContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { useViewSlot } from '@/context/ViewSlotContext';

const { Title, Paragraph, Text } = Typography;

const toc = [
  { key: 'overview', href: '#overview', title: 'Overview' },
  { key: 'load', href: '#load', title: 'Load averages' },
  { key: 'cpu', href: '#cpu', title: 'CPU' },
  { key: 'memory', href: '#memory', title: 'Memory' },
  { key: 'disks', href: '#disks', title: 'Disks' },
  { key: 'code-manager', href: '#code-manager', title: 'Code Manager (Submissions)' },
];

export default function SystemMonitoringHelp() {
  const ids = useMemo(() => toc.map((t) => t.href.slice(1)), []);
  const { setBreadcrumbLabel } = useBreadcrumbContext();
  const { setValue, setBackTo } = useViewSlot();

  useEffect(() => {
    setValue(
      <Typography.Text className="text-base font-medium text-gray-900 dark:text-gray-100 truncate">
        System Monitoring
      </Typography.Text>,
    );
    setBackTo('/help');
  }, []);

  useEffect(() => {
    setBreadcrumbLabel('help/support/system-monitoring', 'System Monitoring');
  }, []);

  useHelpToc({
    items: toc,
    ids,
    extra: (
      <Card className="mt-4" size="small" title="Quick notes" bordered>
        <ul className="list-disc pl-5">
          <li>Header shows a compact snapshot for everyone.</li>
          <li>Admins see full details under System and can change Max Concurrent.</li>
        </ul>
      </Card>
    ),
    onMountScrollToHash: true,
  });

  return (
    <Space direction="vertical" size="small" className="w-full !p-0">
      <Title level={2} className="mb-0">
        System Monitoring &amp; Health
      </Title>

      <section id="overview" className="scroll-mt-24" />
      <Title level={3}>Overview</Title>
      <Paragraph className="mb-0">
        Live system health is broadcast over WebSockets and rendered in the header and System admin
        page. Use it to spot load spikes, memory pressure, or long submission queues.
      </Paragraph>

      <section id="load" className="scroll-mt-24" />
      <Title level={3}>Load averages</Title>
      <Paragraph>
        The three numbers represent the average number of runnable processes over the last
        <Text code> 1</Text>, <Text code>5</Text>, and <Text code>15</Text> minutes. Values near
        your CPU core count suggest the machine is fully utilized; sustained values well above core
        count may indicate saturation.
      </Paragraph>

      <section id="cpu" className="scroll-mt-24" />
      <Title level={3}>CPU</Title>
      <Paragraph>
        <Text strong>Cores</Text> is the logical CPU count. <Text strong>Avg usage</Text> is the
        mean of per-core utilization sampled periodically.
      </Paragraph>

      <section id="memory" className="scroll-mt-24" />
      <Title level={3}>Memory</Title>
      <Paragraph>
        Includes total/used memory and swap usage where applicable. Values appear in human-friendly
        units.
      </Paragraph>

      <section id="disks" className="scroll-mt-24" />
      <Title level={3}>Disks</Title>
      <Paragraph>
        Each mounted disk shows its filesystem type, available space, and total capacity. Low
        available space may cause failures or slowdowns; consider cleaning temporary files or
        expanding storage.
      </Paragraph>

      <section id="code-manager" className="scroll-mt-24" />
      <Title level={3}>Code Manager (Submissions)</Title>
      <Paragraph>
        The submission runtime is capped by <Text strong>Max Concurrent</Text>.{' '}
        <Text strong>Running</Text> shows active containers; <Text strong>Waiting</Text> is queued
        submissions. Adjust concurrency to balance throughput vs. system load.
      </Paragraph>
    </Space>
  );
}
