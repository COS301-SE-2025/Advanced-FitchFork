import { useEffect, useState } from 'react';
import { Menu, Button, Input } from 'antd';
import { useNavigate, useLocation } from 'react-router-dom';
import type { Task } from '@/types/modules/assignments/tasks';
import { createTask, listTasks } from '@/services/modules/assignments/tasks';
import type { GetListTasksResponse } from '@/types/modules/assignments/tasks/responses';
import { useNotifier } from '@/components/Notifier';
import PageHeader from '@/components/PageHeader';
import SettingsGroup from '@/components/SettingsGroup';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';

const TasksLayout = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const module = useModule();
  const assignment = useAssignment();
  const { notifyError, notifySuccess } = useNotifier();
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (!module.id || !assignment.id) return;

    listTasks(module.id, assignment.id)
      .then(async (res: GetListTasksResponse) => {
        if (res.success) {
          let sortedTasks = res.data.sort((a, b) => a.task_number - b.task_number);

          // Auto-create if no tasks exist
          if (sortedTasks.length === 0) {
            const res = await createTask(module.id!, assignment.id!, {
              task_number: 1,
              command: 'echo Hello World',
            });

            if (res.success && res.data) {
              notifySuccess('First task created', res.message);
              sortedTasks = [res.data];
              navigate(`/modules/${module.id}/assignments/${assignment.id}/tasks/${res.data.id}`, {
                replace: true,
              });
            } else {
              notifyError('Failed to create first task', res.message);
            }
          } else {
            setTasks(sortedTasks);

            const pathEndsWithTasks = location.pathname.endsWith('/tasks');
            if (pathEndsWithTasks) {
              navigate(
                `/modules/${module.id}/assignments/${assignment.id}/tasks/${sortedTasks[0].id}`,
                { replace: true },
              );
            }
          }

          setTasks(sortedTasks);
        } else {
          notifyError('Failed to load tasks', res.message);
        }
      })
      .catch((err) => {
        console.error(err);
        notifyError('Failed to load tasks', 'An unexpected error occurred.');
      })
      .finally(() => setLoading(false));
  }, [module.id, assignment.id, location.pathname]);

  const handleCreateTask = async () => {
    if (!module.id || !assignment.id) return;

    const nextTaskNumber = (tasks[tasks.length - 1]?.task_number ?? 0) + 1;

    const payload = {
      task_number: nextTaskNumber,
      command: 'echo Hello World',
    };

    try {
      const res = await createTask(module.id, assignment.id, payload);
      if (res.success && res.data) {
        notifySuccess('Task created', res.message);
        setTasks((prev) => [...prev, res.data]);
        navigate(`/modules/${module.id}/assignments/${assignment.id}/tasks/${res.data.id}`);
      } else {
        notifyError('Failed to create task', res.message);
      }
    } catch (err) {
      console.error(err);
      notifyError('Failed to create task', 'An unexpected error occurred.');
    }
  };

  const menuItems = tasks
    .sort((a, b) => a.task_number - b.task_number)
    .map((task) => ({
      key: task.id.toString(),
      label: `Task ${task.task_number}`,
    }));

  const selectedKey =
    menuItems.find((item) => location.pathname.includes(`/tasks/${item.key}`))?.key || '';

  const selectedTask = tasks.find((t) => t.id.toString() === selectedKey);

  return (
    <div className="">
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
            selectedKeys={[selectedKey]}
            onClick={({ key }) => {
              navigate(`/modules/${module.id}/assignments/${assignment.id}/tasks/${key}`);
            }}
            items={menuItems}
            className="!bg-transparent !p-0"
            style={{ border: 'none' }}
          />

          {/* New Task Button appears right after the Menu */}
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
              <SettingsGroup
                title={`Task ${selectedTask.task_number}`}
                description="This task defines one evaluation step for student submissions. Each task is run independently."
              >
                <div>
                  <label className="block font-medium mb-1">Command</label>
                  <Input
                    size="large"
                    value={selectedTask.command}
                    readOnly
                    className="bg-gray-100 dark:bg-gray-800 w-80"
                  />
                </div>

                <div>
                  <label className="block font-medium mb-1">Created At</label>
                  <Input
                    size="large"
                    value={new Date(selectedTask.created_at).toLocaleString()}
                    readOnly
                    className="bg-gray-100 dark:bg-gray-800 w-80"
                  />
                </div>

                <div>
                  <label className="block font-medium mb-1">Last Updated</label>
                  <Input
                    size="large"
                    value={new Date(selectedTask.updated_at).toLocaleString()}
                    readOnly
                    className="bg-gray-100 dark:bg-gray-800 w-80"
                  />
                </div>

                {/* TODO: Move this to a dedicated TaskView page */}
              </SettingsGroup>
            </>
          ) : (
            <div className="text-gray-400">No task selected.</div>
          )}
        </div>
      </div>
    </div>
  );
};

export default TasksLayout;
