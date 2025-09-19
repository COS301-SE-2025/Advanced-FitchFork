// StepWelcome.tsx
import { useEffect, useMemo, useState } from 'react';
import {
  Typography,
  Button,
  Alert,
  Space,
  Card,
  Tag,
  Divider,
  Select,
  Skeleton,
  Empty,
} from 'antd';
import { ThunderboltOutlined, ToolOutlined, RocketOutlined } from '@ant-design/icons';

import { useModule } from '@/context/ModuleContext';
import { useAssignmentSetup } from '@/context/AssignmentSetupContext';
import { message } from '@/utils/message';

import { getStarterPacks, createStarter } from '@/services/modules/assignments/starter';
import type { StarterPack, StarterId } from '@/types/modules/assignments/starter';
import Tip from '@/components/common/Tip';

const { Title, Paragraph, Text } = Typography;

type Props = { onManual?: () => void };

const StepWelcome = ({ onManual }: Props) => {
  const module = useModule();
  const { assignmentId, refreshAssignment } = useAssignmentSetup();

  const [packs, setPacks] = useState<StarterPack[]>([]);
  const [packsLoading, setPacksLoading] = useState(false);
  const [selectedId, setSelectedId] = useState<StarterId | null>(null);

  const [installing, setInstalling] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  const disabled = installing || !assignmentId;

  // Load available starter packs for this assignment
  useEffect(() => {
    let mounted = true;
    if (!assignmentId) return;
    (async () => {
      setPacksLoading(true);
      try {
        const res = await getStarterPacks(module.id, assignmentId);
        if (mounted) {
          if (res.success) {
            setPacks(res.data || []);
            // If there is exactly one pack, preselect it
            if ((res.data || []).length === 1) setSelectedId(res.data![0].id);
          } else {
            setErr(res.message || 'Failed to load starter packs.');
          }
        }
      } catch (e: any) {
        if (mounted) setErr(e?.message || 'Failed to load starter packs.');
      } finally {
        if (mounted) setPacksLoading(false);
      }
    })();
    return () => {
      mounted = false;
    };
  }, [module.id, assignmentId]);

  const selectedPack = useMemo(
    () => packs.find((p) => p.id === selectedId) || null,
    [packs, selectedId],
  );

  /** One-shot Starter Pack (backend does everything) */
  const runStarter = async () => {
    if (!assignmentId) {
      message.error('Missing assignment ID. Create the assignment first.');
      return;
    }
    if (!selectedId) {
      message.info('Pick a starter pack first.');
      return;
    }

    setInstalling(true);
    setErr(null);

    try {
      const res = await createStarter(module.id, assignmentId, { id: selectedId });
      if (!res.success) throw new Error(res.message || 'Starter installation failed.');
      message.success(res.message || 'Starter installed.');
      await refreshAssignment?.();
    } catch (e: any) {
      const msg = e?.message || 'Starter installation failed.';
      setErr(msg);
      message.error(msg);
    } finally {
      setInstalling(false);
    }
  };

  return (
    <div className="!space-y-6 !w-full">
      {/* Header */}
      <Card className="bg-white dark:bg-gray-900 border-gray-200 dark:border-gray-800 rounded-xl">
        <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
          <div>
            <div className="flex items-center gap-2 flex-wrap">
              <Title level={3} className="!mb-1 !text-gray-900 dark:!text-gray-100">
                Set up your assignment
              </Title>
              <Tip
                iconOnly
                newTab
                to="/help/assignments/setup#overview"
                text="Full setup guide"
              />
            </div>
            <Paragraph type="secondary" className="!mb-0">
              Start with a <b>starter pack</b> (recommended) or proceed with a <b>manual</b> setup.
            </Paragraph>
          </div>
        </div>
      </Card>

      {/* Main grid */}
      <div className="grid grid-cols-1 xl:grid-cols-3 gap-6">
        {/* Left: Starter Pack */}
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
              onClick={runStarter}
              loading={installing}
              disabled={disabled || !selectedId}
            >
              Install Starter
            </Button>
          }
        >
          {packsLoading ? (
            <Skeleton active paragraph={{ rows: 3 }} />
          ) : packs.length === 0 ? (
            <Empty description="No starter packs available. Try Manual Setup." />
          ) : (
            <>
              <Paragraph className="!mb-2" type="secondary">
                Pick a starter. We’ll scaffold files, seed tasks, and (when possible) generate memo
                outputs &amp; a mark allocator.
              </Paragraph>

              <div className="flex flex-col gap-3">
                <div className="flex items-center gap-3">
                  <Text strong>Starter pack</Text>
                  <Select
                    style={{ minWidth: 320 }}
                    placeholder="Select a starter pack"
                    value={selectedId ?? undefined}
                    onChange={(val) => setSelectedId(val)}
                    options={packs.map((p) => ({
                      value: p.id,
                      label: p.name,
                    }))}
                  />
                </div>

                {selectedPack && (
                  <Card size="small" className="bg-transparent border-dashed">
                    <Space size="small" wrap>
                      <Tag color="blue">{selectedPack.language.toUpperCase()}</Tag>
                      {selectedPack.tags?.map((t) => (
                        <Tag key={t}>{t}</Tag>
                      ))}
                    </Space>
                    {selectedPack.description && (
                      <>
                        <Divider className="!my-2" />
                        <Paragraph className="!mb-0" type="secondary">
                          {selectedPack.description}
                        </Paragraph>
                      </>
                    )}
                  </Card>
                )}
              </div>

              {err && (
                <Alert
                  className="!mt-4"
                  type="error"
                  showIcon
                  message="Starter failed"
                  description={err}
                />
              )}
            </>
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
            Prefer to manage files and tasks yourself? Jump straight to <b>Files &amp; Resources</b>
            .
          </Paragraph>

          <ul className="list-disc pl-5 text-sm text-gray-600 dark:text-gray-400 space-y-1">
            <li>Upload main/memo/makefile/spec as needed</li>
            <li>Define or tweak tasks</li>
            <li>Generate memo output &amp; mark allocator when ready</li>
          </ul>

          <div className="pt-4">
            <Button onClick={onManual} block disabled={!assignmentId}>
              Go to Manual Setup
            </Button>
          </div>
        </Card>
      </div>
    </div>
  );
};

export default StepWelcome;
