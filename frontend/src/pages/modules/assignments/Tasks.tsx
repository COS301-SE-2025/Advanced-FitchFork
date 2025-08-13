import { useEffect, useState } from 'react';
import { Menu, Button, Input, Collapse, Empty, Dropdown, Grid, Space, Typography } from 'antd';
import { MoreOutlined, SaveOutlined } from '@ant-design/icons';
import { useNavigate, useLocation } from 'react-router-dom';

import type { Task } from '@/types/modules/assignments/tasks';
import type { GetTaskResponse } from '@/types/modules/assignments/tasks/responses';

import {
  listTasks,
  createTask,
  editTask,
  getTask,
  deleteTask,
} from '@/services/modules/assignments/tasks';
import SettingsGroup from '@/components/SettingsGroup';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { message } from '@/utils/message';
import CodeEditor from '@/components/common/CodeEditor';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';
import { updateMarkAllocator } from '@/services/modules/assignments/mark-allocator';
import { useViewSlot } from '@/context/ViewSlotContext';

const { Panel } = Collapse;

const Tasks = () => {
  const screens = Grid.useBreakpoint();
  const isMobile = !screens.md;
  const navigate = useNavigate();
  const location = useLocation();
  const module = useModule();
  const { assignment } = useAssignment();
  const { setValue } = useViewSlot();
  const { setBreadcrumbLabel } = useBreadcrumbContext();

  useEffect(() => {
    setValue(
      <Typography.Text
        className="text-base font-medium text-gray-900 dark:text-gray-100 truncate"
        title={'Tasks'}
      >
        Tasks
      </Typography.Text>,
    );
  }, []);

  const [tasks, setTasks] = useState<Task[]>([]);
  const [selectedTask, setSelectedTask] = useState<GetTaskResponse['data'] | null>(null);
  const [loading, setLoading] = useState(true);

  const [editedName, setEditedName] = useState('');
  const [editedCommand, setEditedCommand] = useState('');

  // New cache: taskId -> full task (GetTaskResponse['data'])
  const [taskDetails, setTaskDetails] = useState<Record<number, GetTaskResponse['data']>>({});

  const selectedIdMatch = location.pathname.match(/\/tasks\/(\d+)$/);
  const selectedId = selectedIdMatch ? Number(selectedIdMatch[1]) : null;

  useEffect(() => {
    if (!module.id || !assignment.id) return;

    listTasks(module.id, assignment.id)
      .then(async (res) => {
        if (!res.success) return message.error(res.message);

        const sorted = res.data.sort((a, b) => a.task_number - b.task_number);
        setTasks(sorted);

        // Preload details for ALL tasks so we can save allocator for all at once.
        const details = await Promise.all(
          sorted.map((t) => getTask(module.id, assignment.id, t.id)),
        );

        const map: Record<number, GetTaskResponse['data']> = {};
        details.forEach((r) => {
          if (r.success && r.data) map[r.data.id] = r.data;
        });
        setTaskDetails(map);

        // Navigate default
        const endsWithTasks = location.pathname.endsWith('/tasks');
        if (endsWithTasks && sorted.length > 0) {
          navigate(`/modules/${module.id}/assignments/${assignment.id}/tasks/${sorted[0].id}`, {
            replace: true,
          });
        }
      })
      .catch((err) => {
        message.error('Failed to load tasks');
        console.error(err);
      })
      .finally(() => setLoading(false));
  }, [module.id, assignment.id]);

  useEffect(() => {
    if (!selectedId || !module.id || !assignment.id) {
      setSelectedTask(null);
      return;
    }

    getTask(module.id, assignment.id, selectedId)
      .then((res) => {
        if (res.success) {
          setSelectedTask(res.data);
          setEditedCommand(res.data.command);
          setEditedName(res.data.name ?? '');

          // Set breadcrumb label
          setBreadcrumbLabel(
            `modules/${module.id}/assignments/${assignment.id}/tasks/${res.data.id}`,
            res.data.name ?? `Task #${res.data.id}`,
          );
        } else {
          message.error(res.message);
        }
      })
      .catch((err) => {
        message.error('Failed to load task');
        console.error(err);
      });
  }, [selectedId, module.id, assignment.id]);

  const handleCreateTask = async () => {
    if (!module.id || !assignment.id) return;

    const nextTaskNumber = (tasks[tasks.length - 1]?.task_number ?? 0) + 1;
    const payload = {
      task_number: nextTaskNumber,
      name: `Task ${nextTaskNumber}`,
      command: 'echo Hello World',
    };

    try {
      const res = await createTask(module.id, assignment.id, payload);
      if (res.success && res.data) {
        const updated = await listTasks(module.id, assignment.id);
        if (updated.success) {
          const sorted = updated.data.sort((a, b) => a.task_number - b.task_number);
          setTasks(sorted);
          navigate(`/modules/${module.id}/assignments/${assignment.id}/tasks/${res.data.id}`);
        }
      } else {
        message.error(res.message);
      }
    } catch (err) {
      message.error('Failed to create task');
      console.error(err);
    }
  };

  const handleSaveTask = async () => {
    if (!selectedTask || !module.id || !assignment.id) return;

    try {
      const res = await editTask(module.id, assignment.id, selectedTask.id, {
        name: editedName,
        command: editedCommand,
      });

      if (res.success && res.data) {
        message.success(res.message);
        setSelectedTask({ ...selectedTask, command: editedCommand, name: editedName });
        setTasks((prev) =>
          prev.map((task) =>
            task.id === selectedTask.id
              ? { ...task, command: editedCommand, name: editedName }
              : task,
          ),
        );
      } else {
        message.error(res.message);
      }
    } catch (err) {
      message.error('Failed to update task');
    }
  };

  const handleDeleteTask = async (id: number) => {
    if (!module.id || !assignment.id) return;

    try {
      const res = await deleteTask(module.id, assignment.id, id);

      if (res.success) {
        message.success(res.message);
        const updated = await listTasks(module.id, assignment.id);
        if (updated.success) {
          const sorted = updated.data.sort((a, b) => a.task_number - b.task_number);
          setTasks(sorted);
        }

        if (selectedTask?.id === id) {
          setSelectedTask(null);
          navigate(`/modules/${module.id}/assignments/${assignment.id}/tasks`);
        }
      } else {
        message.error(res.message);
      }
    } catch (err) {
      message.error('Failed to delete task');
    }
  };

  type AllocSub = { name: string; value: number };
  type AllocTaskBody = { name: string; subsections: AllocSub[]; value: number };
  type AllocItem = { [taskKey: string]: AllocTaskBody };

  const saveAllocatorAllTasks = async () => {
    if (!module.id || !assignment.id) return;

    // Ensure we have details for all tasks. If some are missing, fetch them now.
    const missing = tasks.filter((t) => !taskDetails[t.id]);
    if (missing.length > 0) {
      try {
        const fetched = await Promise.all(
          missing.map((t) => getTask(module.id, assignment.id, t.id)),
        );
        setTaskDetails((prev) => {
          const copy = { ...prev };
          fetched.forEach((r) => {
            if (r.success && r.data) copy[r.data.id] = r.data;
          });
          return copy;
        });
      } catch {
        message.error('Failed to load all task details for saving.');
        return;
      }
    }

    // Build tasks[] = [{ "taskN": { name, value, subsections[] } }, ...] in ascending task_number
    const byNumber = [...tasks].sort((a, b) => a.task_number - b.task_number);

    const items: AllocItem[] = byNumber.map((t) => {
      const full = taskDetails[t.id];
      const name = (full?.name ?? t.name) || `Task ${t.task_number}`;

      const subsections: AllocSub[] =
        (full?.subsections ?? []).map((s) => ({
          name: s.name,
          value: Number(s.mark_value ?? 0),
        })) ?? [];

      const value = subsections.reduce((sum, s) => sum + (isFinite(s.value) ? s.value : 0), 0);

      return {
        [`task${t.task_number}`]: {
          name,
          subsections,
          value,
        },
      };
    });

    // Optional: validate before sending
    for (const [, obj] of items.entries()) {
      const [key, body] = Object.entries(obj)[0];
      const sum = body.subsections.reduce((a, b) => a + b.value, 0);
      if (Math.abs(sum - body.value) > 1e-6) {
        return message.error(`Task payload invalid at ${key}: subsections do not sum to value`);
      }
    }

    const payload = {
      generated_at: new Date().toISOString(),
      tasks: items,
    };

    try {
      const res = await updateMarkAllocator(module.id, assignment.id, payload);
      if (res.success) {
        message.success('All task marks saved to allocator.');
      } else {
        message.error(res.message ?? 'Failed to save mark allocator');
      }
    } catch (e) {
      console.error(e);
      message.error('Failed to save mark allocator (network/server).');
    }
  };

  const menuItems = tasks.map((task) => ({
    key: task.id.toString(),
    label: (
      <div className="flex justify-between items-center">
        <span>{task.name}</span>
        <Dropdown
          trigger={['click']}
          menu={{
            items: [
              {
                key: 'delete',
                danger: true,
                label: (
                  <span
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDeleteTask(task.id);
                    }}
                  >
                    Delete
                  </span>
                ),
              },
            ],
          }}
        >
          <Button
            type="text"
            size="small"
            icon={<MoreOutlined />}
            onClick={(e) => e.stopPropagation()}
          />
        </Dropdown>
      </div>
    ),
  }));

  const mobileRender = () => (
    <div>
      <Collapse
        accordion
        onChange={(key) => {
          const taskId = Number(key);
          const task = tasks.find((t) => t.id === taskId);
          if (task) {
            navigate(`/modules/${module.id}/assignments/${assignment.id}/tasks/${task.id}`);
          }
        }}
      >
        {tasks.map((task) => (
          <Collapse.Panel key={task.id} header={task.name}>
            <div className="space-y-6">
              {/* Task Details */}
              <div className="space-y-3">
                <div>
                  <label className="block font-medium mb-1">Task Name</label>
                  <Input
                    value={task.id === selectedTask?.id ? editedName : task.name}
                    onChange={(e) => {
                      setEditedName(e.target.value);
                      setSelectedTask((prev) =>
                        prev?.id === task.id ? { ...prev, name: e.target.value } : prev,
                      );
                    }}
                  />
                </div>

                <div>
                  <label className="block font-medium mb-1">Command</label>
                  <Input
                    value={task.id === selectedTask?.id ? editedCommand : task.command}
                    onChange={(e) => {
                      setEditedCommand(e.target.value);
                      setSelectedTask((prev) =>
                        prev?.id === task.id ? { ...prev, command: e.target.value } : prev,
                      );
                    }}
                  />
                </div>

                <div className="flex gap-2">
                  <Button
                    type="primary"
                    icon={<SaveOutlined />}
                    onClick={handleSaveTask}
                    className="flex-1"
                  >
                    Save Task
                  </Button>
                  <Button danger onClick={() => handleDeleteTask(task.id)} className="flex-1">
                    Delete
                  </Button>
                </div>
              </div>

              {/* Subsections */}
              {task.id === selectedTask?.id &&
                selectedTask.subsections &&
                selectedTask.subsections.length > 0 && (
                  <div className="space-y-3">
                    <h4 className="font-semibold text-gray-700 dark:text-gray-300">Assessment</h4>

                    <Collapse accordion>
                      {selectedTask.subsections.map((sub, index) => (
                        <Collapse.Panel header={sub.name} key={index}>
                          <div className="space-y-4 px-1 pt-1 pb-2">
                            <div>
                              <label className="block font-medium mb-1">Mark</label>
                              <div className="flex items-center gap-2">
                                <Input
                                  type="number"
                                  value={sub.mark_value ?? 0}
                                  className="w-20"
                                  readOnly
                                />
                                <Button size="small" type="primary">
                                  Save Mark
                                </Button>
                              </div>
                            </div>

                            <div>
                              <CodeEditor
                                title="Memo Output"
                                value={sub.memo_output ?? ''}
                                language="plaintext"
                                height={200}
                                readOnly
                              />
                            </div>
                          </div>
                        </Collapse.Panel>
                      ))}
                    </Collapse>
                  </div>
                )}
            </div>
          </Collapse.Panel>
        ))}
      </Collapse>

      <Button block type="dashed" className="!mt-6" onClick={handleCreateTask}>
        + New Task
      </Button>
    </div>
  );

  return (
    <>
      {isMobile ? (
        mobileRender()
      ) : (
        <div className="bg-white dark:bg-gray-900 border rounded-md border-gray-200 dark:border-gray-800 flex overflow-hidden">
          <div className="w-[240px] bg-gray-50 dark:bg-gray-950 border-r border-gray-200 dark:border-gray-800 px-2 py-2">
            <Menu
              mode="inline"
              theme="light"
              selectedKeys={selectedId ? [selectedId.toString()] : []}
              onClick={({ key }) => {
                navigate(`/modules/${module.id}/assignments/${assignment.id}/tasks/${key}`);
              }}
              items={menuItems}
              className="!bg-transparent !p-0"
              style={{ border: 'none' }}
            />
            <div className="px-1 mt-4">
              <Button block type="dashed" onClick={handleCreateTask}>
                + New Task
              </Button>
            </div>
          </div>

          <div className="flex-1 p-6 max-w-6xl">
            {loading ? (
              <div className="text-gray-400">Loading tasks...</div>
            ) : tasks.length === 0 ? (
              <Empty
                description={<div className="text-gray-700 dark:text-gray-300">No Tasks Found</div>}
              >
                <Button type="primary" onClick={handleCreateTask}>
                  + New Task
                </Button>
              </Empty>
            ) : selectedTask ? (
              <div className="!space-y-6">
                <SettingsGroup
                  title={`Task`}
                  description="Basic info and execution command for this task."
                >
                  <div className="space-y-6">
                    <div>
                      <label className="block font-medium mb-1">Task Name</label>
                      <Input
                        value={editedName}
                        onChange={(e) => setEditedName(e.target.value)}
                        className="w-full"
                      />
                    </div>
                    <div>
                      <label className="block font-medium mb-1">Command</label>
                      <Input
                        value={editedCommand}
                        onChange={(e) => setEditedCommand(e.target.value)}
                        className="w-full"
                      />
                    </div>
                    <div className="flex justify-end">
                      <Button icon={<SaveOutlined />} type="primary" onClick={handleSaveTask}>
                        Save Task
                      </Button>
                    </div>
                  </div>
                </SettingsGroup>

                {selectedTask.subsections?.length > 0 && (
                  <SettingsGroup title="Assessment" description="Breakdown of marks by subsection.">
                    <Collapse accordion bordered>
                      {selectedTask.subsections?.map((sub, index) => (
                        <Panel header={sub.name} key={index}>
                          <div className="space-y-4 px-3 pt-1 pb-2">
                            <div>
                              <label className="block font-medium mb-1">Mark</label>
                              <Space.Compact className="flex items-center w-full">
                                <Input
                                  type="number"
                                  step="0.1"
                                  min={0}
                                  max={1}
                                  value={sub.mark_value ?? 0}
                                  onChange={(e) => {
                                    const val = Number(e.target.value);
                                    setSelectedTask((prev) => {
                                      if (!prev) return prev;
                                      const updatedSubs = prev.subsections?.map((s) =>
                                        s.name === sub.name ? { ...s, mark_value: val } : s,
                                      );
                                      const updated = { ...prev, subsections: updatedSubs };
                                      // sync cache
                                      setTaskDetails((m) =>
                                        prev ? { ...m, [prev.id]: updated } : m,
                                      );
                                      return updated;
                                    });
                                  }}
                                />
                                <Button type="primary" onClick={saveAllocatorAllTasks}>
                                  Save Mark
                                </Button>
                              </Space.Compact>
                            </div>

                            <div>
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
                          </div>
                        </Panel>
                      ))}
                    </Collapse>
                  </SettingsGroup>
                )}
              </div>
            ) : (
              <div className="text-gray-400">Loading selected task…</div>
            )}
          </div>
        </div>
      )}
    </>
  );
};

export default Tasks;
