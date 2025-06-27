// src/pages/modules/assignments/tasks/TasksLayout.tsx

import { useEffect, useState } from 'react';
import { Menu, Button, Input, Typography, Collapse } from 'antd';
import { SaveOutlined } from '@ant-design/icons';
import { useNavigate, useLocation } from 'react-router-dom';

import type { Task } from '@/types/modules/assignments/tasks';
import type { GetTaskResponse } from '@/types/modules/assignments/tasks/responses';

import { listTasks, createTask, editTask, getTask } from '@/services/modules/assignments/tasks';
import { useNotifier } from '@/components/Notifier';
import PageHeader from '@/components/PageHeader';
import SettingsGroup from '@/components/SettingsGroup';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';

const { Panel } = Collapse;

const TasksLayout = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const module = useModule();
  const { assignment, readiness } = useAssignment();
  const { notifyError, notifySuccess } = useNotifier();

  const [tasks, setTasks] = useState<Task[]>([]);
  const [selectedTask, setSelectedTask] = useState<GetTaskResponse['data'] | null>(null);
  const [loading, setLoading] = useState(true);

  const [editedName, setEditedName] = useState('');
  const [editedCommand, setEditedCommand] = useState('');

  const selectedIdMatch = location.pathname.match(/\/tasks\/(\d+)$/);
  const selectedId = selectedIdMatch ? Number(selectedIdMatch[1]) : null;

  useEffect(() => {
    if (!module.id || !assignment.id) return;

    listTasks(module.id, assignment.id)
      .then((res) => {
        if (res.success) {
          const sorted = res.data.sort((a, b) => a.task_number - b.task_number);
          setTasks(sorted);

          const endsWithTasks = location.pathname.endsWith('/tasks');
          if (endsWithTasks && sorted.length > 0) {
            console.log(
              '[REDIRECT] location ends with /tasks, redirecting to first task ID:',
              sorted[0].id,
            );
            navigate(`/modules/${module.id}/assignments/${assignment.id}/tasks/${sorted[0].id}`, {
              replace: true,
            });
          }
        } else {
          notifyError('Failed to load tasks', res.message);
        }
      })
      .catch((err) => {
        console.error(err);
        notifyError('Failed to load tasks', 'An unexpected error occurred.');
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
        } else {
          notifyError('Failed to load task', res.message);
        }
      })
      .catch((err) => {
        console.error(err);
        notifyError('Failed to load task', 'An unexpected error occurred.');
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
        notifySuccess('Task created', res.message);

        const updated = await listTasks(module.id, assignment.id);
        if (updated.success) {
          const sorted = updated.data.sort((a, b) => a.task_number - b.task_number);
          setTasks(sorted);
        }

        // Removed auto-navigation
      } else {
        notifyError('Failed to create task', res.message);
      }
    } catch (err) {
      console.error(err);
      notifyError('Failed to create task', 'An unexpected error occurred.');
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
        notifySuccess('Task updated', res.message);
        setSelectedTask({ ...selectedTask, command: editedCommand, name: editedName });
        setTasks((prev) =>
          prev.map((task) =>
            task.id === selectedTask.id
              ? { ...task, command: editedCommand, name: editedName }
              : task,
          ),
        );
      } else {
        notifyError('Failed to update task', res.message);
      }
    } catch (err) {
      console.error(err);
      notifyError('Failed to update task', 'An unexpected error occurred.');
    }
  };

  const menuItems = tasks.map((task) => ({
    key: task.id.toString(),
    label: `Task ${task.task_number}`,
  }));

  const onClickNextStep = async () => {
    // Optionally trigger a refresh or call a backend readiness endpoint
    // await refreshReadiness(); // if available

    navigate(`/modules/${module.id}/assignments/${assignment.id}/memo-output`);
  };

  return (
    <div>
      <PageHeader
        title="Tasks for This Assignment"
        description="Browse, view, and manage the individual tasks configured for this assignment."
      />

      <div className="flex">
        {/* Sidebar */}
        <div className="w-[240px] bg-white dark:bg-gray-950 border-r border-gray-200 dark:border-gray-800 px-2 py-4">
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
            <Button block type="default" onClick={handleCreateTask}>
              + New Task
            </Button>
          </div>
        </div>

        {/* Main content */}
        <div className="flex-1 p-6 max-w-6xl">
          {loading ? (
            <div className="text-gray-400">Loading tasks...</div>
          ) : selectedTask ? (
            <>
              {/* Group 1: Task Info */}
              <div className="mb-10">
                <SettingsGroup
                  title={`Task ${selectedTask.task_number}`}
                  description="Basic info and execution command for this task."
                >
                  <div className="space-y-6">
                    <div>
                      <label className="block font-medium mb-1">Task Name</label>
                      <Input
                        value={editedName}
                        onChange={(e) => setEditedName(e.target.value)}
                        className="w-80"
                      />
                    </div>

                    <div>
                      <label className="block font-medium mb-1">Command</label>
                      <Input
                        value={editedCommand}
                        onChange={(e) => setEditedCommand(e.target.value)}
                        className="w-full max-w-2xl"
                      />
                    </div>

                    <div className="flex justify-end">
                      <Button icon={<SaveOutlined />} type="primary" onClick={handleSaveTask}>
                        Save Task
                      </Button>
                    </div>
                  </div>
                </SettingsGroup>
              </div>

              {/* Group 2: Assessment */}
              <div className="mb-10">
                <SettingsGroup title="Assessment" description="Breakdown of marks by subsection.">
                  <div className="space-y-6">
                    <div>
                      <label className="block font-medium mb-1">Mark Value</label>
                      <Input
                        value={
                          typeof selectedTask.mark_value === 'number'
                            ? selectedTask.mark_value
                            : 'N/A'
                        }
                        readOnly
                        className="bg-gray-100 dark:bg-gray-800 w-40"
                      />
                    </div>

                    {selectedTask.subsections && selectedTask.subsections.length > 0 && (
                      <div>
                        <label className="block font-medium mb-1">Subsections</label>
                        <Collapse accordion bordered>
                          {selectedTask.subsections.map((sub, index) => (
                            <Panel header={sub.name} key={index}>
                              <div className="space-y-3 px-3 pt-1 pb-2">
                                <div>
                                  <Typography.Text type="secondary">Mark Value:</Typography.Text>{' '}
                                  <Typography.Text strong>
                                    {sub.mark_value ?? 'N/A'}
                                  </Typography.Text>
                                </div>
                                <div>
                                  <Typography.Text type="secondary">Memo Output:</Typography.Text>
                                  {sub.memo_output ? (
                                    <Typography.Paragraph
                                      copyable
                                      className="bg-gray-100 dark:bg-gray-800 mt-1 p-3 rounded-md border border-gray-200 dark:border-gray-700 text-sm whitespace-pre-wrap"
                                    >
                                      {sub.memo_output}
                                    </Typography.Paragraph>
                                  ) : (
                                    <Typography.Text type="secondary" italic>
                                      No memo output
                                    </Typography.Text>
                                  )}
                                </div>
                              </div>
                            </Panel>
                          ))}
                        </Collapse>
                      </div>
                    )}
                  </div>
                </SettingsGroup>
              </div>

              {/* Group 3: Timestamps */}
              <SettingsGroup title="Timestamps">
                <div className="space-y-4">
                  <div>
                    <label className="block font-medium mb-1">Created At</label>
                    <Input
                      size="large"
                      value={
                        selectedTask.created_at
                          ? new Date(selectedTask.created_at).toLocaleString()
                          : 'N/A'
                      }
                      readOnly
                      className="bg-gray-100 dark:bg-gray-800 w-80"
                    />
                  </div>
                  <div>
                    <label className="block font-medium mb-1">Last Updated</label>
                    <Input
                      size="large"
                      value={
                        selectedTask.updated_at
                          ? new Date(selectedTask.updated_at).toLocaleString()
                          : 'N/A'
                      }
                      readOnly
                      className="bg-gray-100 dark:bg-gray-800 w-80"
                    />
                  </div>
                </div>
              </SettingsGroup>
            </>
          ) : (
            <div className="text-gray-400">No task selected.</div>
          )}

          <div className="flex justify-end mt-6">
            <Button type="primary" disabled={!readiness?.tasks_present} onClick={onClickNextStep}>
              Next Step: Memo Output
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
};

export default TasksLayout;
