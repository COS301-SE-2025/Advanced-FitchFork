import React from 'react';
import { Button, Collapse, Input, Space } from 'antd';
import SettingsGroup from '@/components/SettingsGroup';
import CodeEditor from '@/components/common/CodeEditor';
import { useTasksPage } from '../context';

const { Panel } = Collapse;

const AssessmentSection: React.FC = () => {
  const { selectedTask, setSelectedTask, setTaskDetails, saveAllocatorAllTasks } = useTasksPage();
  if (!selectedTask || selectedTask.code_coverage) return null;

  const subs = selectedTask.subsections ?? [];
  if (!subs.length) return null;

  return (
    <SettingsGroup title="Assessment" description="Breakdown of marks by subsection.">
      <Collapse accordion bordered>
        {subs.map((sub, index) => (
          <Panel header={sub.name} key={index}>
            <div className="space-y-4 px-3 pt-1 pb-2">
              <div>
                <label className="block font-medium mb-1">Mark</label>
                <Space.Compact className="flex items-center w-full">
                  <Input
                    type="number"
                    min={0}
                    step={1}
                    value={sub.value ?? 0}
                    onChange={(e) => {
                      const val = parseInt(e.target.value, 10) || 0;
                      setSelectedTask((prev) => {
                        if (!prev) return prev;
                        const updatedSubs = prev.subsections?.map((s) =>
                          s.name === sub.name ? { ...s, value: val } : s,
                        );
                        const updated = { ...prev, subsections: updatedSubs };
                        setTaskDetails((m) => (prev ? { ...m, [prev.id]: updated } : m));
                        return updated;
                      });
                    }}
                  />
                  <Button type="primary" onClick={saveAllocatorAllTasks}>
                    Save Mark
                  </Button>
                </Space.Compact>
              </div>

              <div className="mt-2">
                <CodeEditor
                  title="Memo Output"
                  value={sub.memo_output ?? ''}
                  language="plaintext"
                  height={200}
                  readOnly
                />
              </div>
            </div>
          </Panel>
        ))}
      </Collapse>
    </SettingsGroup>
  );
};

export default AssessmentSection;
