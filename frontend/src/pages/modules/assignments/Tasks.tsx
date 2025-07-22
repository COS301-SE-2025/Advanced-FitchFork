import { useEffect, useState } from 'react';
import { Menu, Button, Input, Collapse, Empty, Dropdown } from 'antd';
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
import CodeEditor from '@/components/CodeEditor';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

const { Panel } = Collapse;

const Tasks = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const module = useModule();
  const { assignment } = useAssignment();
  const { setBreadcrumbLabel } = useBreadcrumbContext();

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
            navigate(`/modules/${module.id}/assignments/${assignment.id}/tasks/${sorted[0].id}`, {
              replace: true,
            });
          }
        } else {
          message.error(res.message);
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

  return (
    <div className="bg-white dark:bg-gray-800 rounded border border-gray-200 dark:border-gray-700 flex overflow-hidden">
      <div className="w-[240px] bg-gray-50 dark:bg-gray-900 border-r border-gray-200 dark:border-gray-700 px-2 py-2">
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
                          <div className="flex items-center gap-2">
                            <Input type="number" value={sub.mark_value ?? 0} className="w-16" />
                            <Button type="primary">Save Mark</Button>
                          </div>
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
  );
};

export default Tasks;
