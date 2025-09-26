import React, { useState } from 'react';
import { Button, Collapse, Input, Typography, List, Segmented, Row, Col, Tooltip } from 'antd';
import SettingsGroup from '@/components/SettingsGroup';
import CodeEditor from '@/components/common/CodeEditor';
import { useTasksPage } from '../context';
import { useAssignment } from '@/context/AssignmentContext';
import { useUI } from '@/context/UIContext';

const { Panel } = Collapse;
const { Text } = Typography;

type SubRow = {
  name: string;
  value: number;
  memo_output: string | null;
  feedback?: string; // single string per subsection
  regex?: string[]; // line-aligned regex (blanks allowed)
};

const splitLines = (s: string) => s.replace(/\r\n/g, '\n').replace(/\r/g, '\n').split('\n');

function padTo(arr: string[] | undefined, len: number): string[] {
  const base = Array.isArray(arr) ? [...arr] : [];
  while (base.length < len) base.push('');
  return base;
}

const AssessmentSection: React.FC = () => {
  const { selectedTask, setSelectedTask, setTaskDetails, saveAllocatorAllTasks } = useTasksPage();
  const { config } = useAssignment();
  const { isMobile } = useUI();
  const [rawView, setRawView] = useState<Record<string, boolean>>({});

  if (!selectedTask || selectedTask.task_type === 'coverage') return null;
  const subs: SubRow[] = (selectedTask.subsections ?? []) as SubRow[];
  if (!subs.length) return null;

  const showRegexUI = config?.marking.marking_scheme === 'regex';
  const showFeedbackUI = config?.marking.feedback_scheme === 'manual';

  const updateSub = (subName: string, patch: Record<string, unknown>) => {
    setSelectedTask((prev) => {
      if (!prev) return prev;
      const updatedSubs = prev.subsections?.map((s: any) =>
        s.name === subName ? { ...s, ...patch } : s,
      );
      const updated = { ...prev, subsections: updatedSubs };
      setTaskDetails((m) => (prev ? { ...m, [prev.id]: updated } : m));
      return updated;
    });
  };

  return (
    <SettingsGroup title="Assessment" description="Breakdown of marks by subsection.">
      <Collapse accordion bordered>
        {subs.map((sub, index) => {
          const subKey = sub.name || String(index);
          const isRaw = rawView[subKey] ?? false;

          const memoOutput = sub.memo_output ?? '';
          const lines = splitLines(memoOutput);
          const regexes: string[] = padTo(sub.regex, lines.length);
          const feedback: string = sub.feedback ?? '';

          return (
            <Panel header={sub.name} key={subKey}>
              <div className="space-y-4 px-3 pt-1 pb-2">
                {/* Mark */}
                <div>
                  <label className="block font-medium mb-1">Mark</label>
                  <Input
                    type="number"
                    min={0}
                    step={1}
                    value={sub.value ?? 0}
                    onChange={(e) =>
                      updateSub(sub.name, { value: parseInt(e.target.value, 10) || 0 })
                    }
                    style={{ maxWidth: isMobile ? '100%' : 200 }}
                    aria-label={`Mark for ${sub.name}`}
                  />
                </div>

                {/* Single feedback textarea */}
                {showFeedbackUI && (
                  <div>
                    <label className="block font-medium mb-1">Feedback</label>
                    <Input.TextArea
                      placeholder="Write overall feedback for this subsection..."
                      value={feedback}
                      autoSize={{ minRows: 2, maxRows: 6 }}
                      onChange={(e) => updateSub(sub.name, { feedback: e.target.value })}
                      aria-label={`Feedback for ${sub.name}`}
                    />
                  </div>
                )}

                {(showRegexUI || showFeedbackUI) && (
                  <>
                    <div className="flex items-center justify-between">
                      <label className="block font-medium mb-1">Output Mapping</label>
                      <Segmented
                        size="small"
                        value={isRaw ? 'Raw' : 'Lines'}
                        onChange={(v) => {
                          const toRaw = v === 'Raw';
                          setRawView((old) => ({ ...old, [subKey]: toRaw }));

                          // When switching to "Lines", ensure regex array exists and is padded with "" (not '^$').
                          if (!toRaw) {
                            const lineCount = lines.length;
                            const padded = padTo(sub.regex, lineCount);
                            if (!Array.isArray(sub.regex) || sub.regex.length < lineCount) {
                              updateSub(sub.name, { regex: padded });
                            }
                          }
                        }}
                        options={['Lines', 'Raw']}
                        aria-label={`Toggle raw/lines for ${sub.name}`}
                      />
                    </div>

                    {isRaw ? (
                      <CodeEditor
                        title="Memo Output"
                        value={memoOutput}
                        language="plaintext"
                        height={220}
                        readOnly
                      />
                    ) : (
                      <List
                        size="small"
                        bordered
                        split
                        dataSource={lines.map((line, idx) => ({ line, idx }))}
                        renderItem={({ line, idx }) => {
                          const pattern = regexes[idx] ?? '';
                          return (
                            <List.Item>
                              <Row
                                gutter={12}
                                wrap={isMobile}
                                style={{ width: '100%' }}
                                align="middle"
                              >
                                {/* Line number */}
                                <Col flex="40px">
                                  <Text type="secondary" code>
                                    {idx + 1}
                                  </Text>
                                </Col>

                                {/* Memo line truncated with tooltip */}
                                <Col flex={isMobile ? '100%' : 'auto'}>
                                  <Tooltip title={line || '(empty)'}>
                                    <Text
                                      code
                                      style={{
                                        display: 'inline-block',
                                        maxWidth: isMobile ? '100%' : 360,
                                        whiteSpace: 'nowrap',
                                        overflow: 'hidden',
                                        textOverflow: 'ellipsis',
                                      }}
                                    >
                                      {line === '' ? '␀ (empty)' : line}
                                    </Text>
                                  </Tooltip>
                                </Col>

                                {showRegexUI && (
                                  <Col flex={isMobile ? '100%' : '260px'}>
                                    <label className="block text-xs font-medium mb-1">
                                      Regex pattern
                                    </label>
                                    <Input
                                      allowClear
                                      placeholder="e.g. ^OK$ or /OK/i"
                                      value={pattern}
                                      onChange={(e) => {
                                        const next = padTo(regexes, lines.length);
                                        next[idx] = e.target.value; // keep blanks as ""
                                        updateSub(sub.name, { regex: next });
                                      }}
                                      aria-label={`Regex for line ${idx + 1} in ${sub.name}`}
                                    />
                                  </Col>
                                )}
                              </Row>
                            </List.Item>
                          );
                        }}
                      />
                    )}
                  </>
                )}

                {!showRegexUI && !showFeedbackUI && (
                  <CodeEditor
                    title="Memo Output"
                    value={memoOutput}
                    language="plaintext"
                    height={200}
                    readOnly
                  />
                )}
              </div>
            </Panel>
          );
        })}
      </Collapse>

      {/* Single save button */}
      <div className="flex justify-end mt-4">
        <Button type="primary" onClick={saveAllocatorAllTasks}>
          Save All Changes
        </Button>
      </div>
    </SettingsGroup>
  );
};

export default AssessmentSection;
