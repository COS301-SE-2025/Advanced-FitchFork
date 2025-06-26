import { useEffect } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import type { Task } from '@/types/modules/assignments/tasks';

// TEMP: replace with real task fetch logic
const useTasks = (): Task[] => {
  return [
    // {
    //   id: 1,
    //   assignment_id: 42,
    //   task_number: 1,
    //   command: 'echo Hello',
    //   created_at: '',
    //   updated_at: '',
    // },
    // {
    //   id: 2,
    //   assignment_id: 42,
    //   task_number: 2,
    //   command: 'echo World',
    //   created_at: '',
    //   updated_at: '',
    // },
  ];
};

const TasksIndex = () => {
  const tasks = useTasks();
  const navigate = useNavigate();
  const { id: module_id, assignment_id } = useParams();

  useEffect(() => {
    if (tasks.length > 0) {
      const firstTask = tasks.sort((a, b) => a.task_number - b.task_number)[0];
      navigate(`/modules/${module_id}/assignments/${assignment_id}/tasks/${firstTask.id}`, {
        replace: true,
      });
    }
  }, [tasks, module_id, assignment_id, navigate]);

  if (tasks.length === 0) {
    return (
      <div className="text-center mt-12 text-gray-500 text-lg">
        No tasks found for this assignment.
        <br />
        Click <strong>+ New Task</strong> to create one.
      </div>
    );
  }

  return null;
};

export default TasksIndex;
