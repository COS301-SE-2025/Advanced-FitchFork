// src/pages/modules/assignments/Tasks/context.tsx
import React, { createContext, useContext, useEffect, useMemo, useState, useCallback } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { Grid } from 'antd';

import type { Task } from '@/types/modules/assignments/tasks';
import type { GetTaskResponse } from '@/types/modules/assignments/tasks/responses';
import type {
  MarkAllocatorFile,
  MarkAllocatorTaskEntry,
  MarkAllocatorSubsection,
} from '@/types/modules/assignments/mark-allocator';

import {
  listTasks,
  createTask,
  editTask,
  getTask,
  deleteTask as apiDeleteTask,
} from '@/services/modules/assignments/tasks';
import { updateMarkAllocator } from '@/services/modules/assignments/mark-allocator';
import { message } from '@/utils/message';
import { useModule } from '@/context/ModuleContext';
import { useAssignment } from '@/context/AssignmentContext';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

function normalizeSubsections(subs: any[]) {
  if (!subs) return [];
  return subs.map((s) => ({
    name: s?.name ?? s?.label ?? 'Unnamed',
    value: Number.isFinite(Number(s?.value)) ? parseInt(String(s.value), 10) : 0,
    memo_output: s?.memo_output ?? s?.output ?? '',
  }));
}

type TaskDetail = GetTaskResponse['data'];

type Ctx = {
  // routing + layout
  isMobile: boolean;
  moduleId: number;
  assignmentId: number;
  selectedId: number | null;
  setSelectedId: (id: number) => void;

  // lists & details
  loading: boolean;
  tasks: Task[];
  selectedTask: TaskDetail | null;

  // edit fields (Task)
  editedName: string;
  setEditedName: (v: string) => void;
  editedCommand: string;
  setEditedCommand: (v: string) => void;
  editedCoverage: boolean;
  setEditedCoverage: (v: boolean) => void;

  // actions
  refreshTasks: () => Promise<void>;
  createNewTask: () => Promise<void>;
  saveTask: () => Promise<void>;
  deleteTask: (id: number) => Promise<void>;

  // assessment
  setSelectedTask: React.Dispatch<React.SetStateAction<TaskDetail | null>>;
  saveAllocatorAllTasks: () => Promise<void>;
  taskDetails: Record<number, TaskDetail>;
  setTaskDetails: React.Dispatch<React.SetStateAction<Record<number, TaskDetail>>>;
};

const TasksPageContext = createContext<Ctx | null>(null);

export const useTasksPage = () => {
  const ctx = useContext(TasksPageContext);
  if (!ctx) throw new Error('useTasksPage must be used inside <TasksPageProvider>');
  return ctx;
};

export const TasksPageProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const screens = Grid.useBreakpoint();
  const isMobile = !screens.md;

  const module = useModule();
  const { assignment } = useAssignment();
  const moduleId = module.id!;
  const assignmentId = assignment.id!;

  const navigate = useNavigate();
  const location = useLocation();
  const { setBreadcrumbLabel } = useBreadcrumbContext();

  const [tasks, setTasks] = useState<Task[]>([]);
  const [taskDetails, setTaskDetails] = useState<Record<number, TaskDetail>>({});
  const [selectedTask, setSelectedTask] = useState<TaskDetail | null>(null);
  const [loading, setLoading] = useState(true);

  const selectedIdMatch = location.pathname.match(/\/tasks\/(\d+)$/);
  const selectedIdFromPath = selectedIdMatch ? Number(selectedIdMatch[1]) : null;
  const [selectedId, _setSelectedId] = useState<number | null>(selectedIdFromPath);

  const setSelectedId = useCallback(
    (id: number) => {
      _setSelectedId(id);
      navigate(`/modules/${moduleId}/assignments/${assignmentId}/tasks/${id}`);
    },
    [navigate, moduleId, assignmentId],
  );

  const [editedName, setEditedName] = useState('');
  const [editedCommand, setEditedCommand] = useState('');
  const [editedCoverage, setEditedCoverage] = useState(false);

  const refreshTasks = useCallback(async (): Promise<void> => {
    if (!moduleId || !assignmentId) return;
    setLoading(true);
    try {
      const res = await listTasks(moduleId, assignmentId);
      if (!res.success) {
        message.error(res.message); // don't return the value
        return;
      }

      const sorted = res.data.sort((a, b) => a.task_number - b.task_number);
      setTasks(sorted);

      const details = await Promise.all(sorted.map((t) => getTask(moduleId, assignmentId, t.id)));
      const map: Record<number, TaskDetail> = {};
      details.forEach((r) => {
        if (r.success && r.data) {
          map[r.data.id] = { ...r.data, subsections: normalizeSubsections(r.data.subsections) };
        }
      });
      setTaskDetails(map);

      const endsWithTasks = location.pathname.endsWith('/tasks');
      if (endsWithTasks && sorted.length > 0) {
        setSelectedId(sorted[0].id);
      }
    } catch (e) {
      console.error(e);
      message.error('Failed to load tasks');
    } finally {
      setLoading(false);
    }
  }, [moduleId, assignmentId, location.pathname, setSelectedId]);

  // initial + dependency refresh
  useEffect(() => {
    if (!moduleId || !assignmentId) return;
    refreshTasks();
  }, [moduleId, assignmentId, refreshTasks]);

  // load selected task details when path changes
  useEffect(() => {
    if (!selectedId || !moduleId || !assignmentId) {
      setSelectedTask(null);
      return;
    }
    (async () => {
      try {
        const res = await getTask(moduleId, assignmentId, selectedId);
        if (res.success && res.data) {
          const normalized = {
            ...res.data,
            subsections: normalizeSubsections(res.data.subsections),
          };
          setSelectedTask(normalized);
          setEditedCommand(normalized.command);
          setEditedName(normalized.name ?? '');
          setEditedCoverage(!!normalized.code_coverage);

          setBreadcrumbLabel(
            `modules/${moduleId}/assignments/${assignmentId}/tasks/${normalized.id}`,
            normalized.name ?? `Task #${normalized.id}`,
          );
        } else {
          message.error(res.message);
        }
      } catch (e) {
        console.error(e);
        message.error('Failed to load task');
      }
    })();
  }, [selectedId, moduleId, assignmentId]);

  const createNewTask = useCallback(async () => {
    if (!moduleId || !assignmentId) return;
    const nextTaskNumber = (tasks[tasks.length - 1]?.task_number ?? 0) + 1;
    const payload = {
      task_number: nextTaskNumber,
      name: `Task ${nextTaskNumber}`,
      command: 'echo Hello World',
      code_coverage: false,
    };
    try {
      const res = await createTask(moduleId, assignmentId, payload);
      if (res.success && res.data) {
        await refreshTasks();
        setSelectedId(res.data.id);
      } else {
        message.error(res.message);
      }
    } catch (e) {
      console.error(e);
      message.error('Failed to create task');
    }
  }, [moduleId, assignmentId, tasks, refreshTasks, setSelectedId]);

  const saveTask = useCallback(async () => {
    if (!selectedTask) return;
    try {
      const res = await editTask(moduleId, assignmentId, selectedTask.id, {
        name: editedName,
        command: editedCommand,
        code_coverage: editedCoverage,
      });
      if (res.success && res.data) {
        message.success(res.message);
        setSelectedTask((prev) =>
          prev
            ? { ...prev, name: editedName, command: editedCommand, code_coverage: editedCoverage }
            : prev,
        );
        setTasks((prev) =>
          prev.map((t) =>
            t.id === selectedTask.id
              ? { ...t, name: editedName, command: editedCommand, code_coverage: editedCoverage }
              : t,
          ),
        );
      } else {
        message.error(res.message);
      }
    } catch (e) {
      console.error(e);
      message.error('Failed to update task');
    }
  }, [moduleId, assignmentId, selectedTask, editedName, editedCommand, editedCoverage]);

  const deleteTask = useCallback(
    async (id: number) => {
      if (!moduleId || !assignmentId) return;
      try {
        const res = await apiDeleteTask(moduleId, assignmentId, id);
        if (res.success) {
          message.success(res.message);
          await refreshTasks();
          if (selectedTask?.id === id) {
            setSelectedTask(null);
            navigate(`/modules/${moduleId}/assignments/${assignmentId}/tasks`);
          }
        } else {
          message.error(res.message);
        }
      } catch (e) {
        console.error(e);
        message.error('Failed to delete task');
      }
    },
    [moduleId, assignmentId, navigate, selectedTask, refreshTasks],
  );

  const saveAllocatorAllTasks = useCallback(async (): Promise<void> => {
    if (!moduleId || !assignmentId) return;

    const missing = tasks.filter((t) => !taskDetails[t.id]);
    if (missing.length) {
      try {
        const fetched = await Promise.all(
          missing.map((t) => getTask(moduleId, assignmentId, t.id)),
        );
        setTaskDetails((prev) => {
          const copy = { ...prev };
          fetched.forEach((r) => {
            if (r.success && r.data) {
              copy[r.data.id] = {
                ...r.data,
                subsections: normalizeSubsections(r.data.subsections),
              };
            }
          });
          return copy;
        });
      } catch {
        message.error('Failed to load all task details for saving.');
        return; // return void explicitly
      }
    }

    const byNumber = [...tasks].sort((a, b) => a.task_number - b.task_number);
    const tasksPayload: MarkAllocatorTaskEntry[] = byNumber.map((t) => {
      const full = taskDetails[t.id];
      const subsections: MarkAllocatorSubsection[] =
        (full?.subsections ?? []).map((s) => ({
          name: s.name,
          value: Number.isFinite(Number(s.value)) ? parseInt(String(s.value), 10) : 0,
        })) ?? [];
      const value = subsections.reduce((sum, c) => sum + c.value, 0);
      const name = (full?.name ?? t.name) || `Task ${t.task_number}`;
      const key = `task${t.task_number}`;
      return { [key]: { task_number: t.task_number, name, value, subsections } };
    });

    const totalValue = tasksPayload.reduce((sum, entry) => {
      const body = Object.values(entry)[0];
      return sum + (body?.value ?? 0);
    }, 0);

    const payload: MarkAllocatorFile = {
      generated_at: new Date().toISOString(),
      tasks: tasksPayload,
      total_value: totalValue,
    };

    try {
      const res = await updateMarkAllocator(moduleId, assignmentId, payload);
      if (res.success) {
        message.success('All task marks saved to allocator.');
      } else {
        message.error(res.message ?? 'Failed to save mark allocator');
      }
    } catch (e) {
      console.error(e);
      message.error('Failed to save mark allocator (network/server).');
    }
  }, [moduleId, assignmentId, tasks, taskDetails]);

  const value = useMemo<Ctx>(
    () => ({
      isMobile,
      moduleId,
      assignmentId,
      selectedId,
      setSelectedId,
      loading,
      tasks,
      selectedTask,
      editedName,
      setEditedName,
      editedCommand,
      setEditedCommand,
      editedCoverage,
      setEditedCoverage,
      refreshTasks,
      createNewTask,
      saveTask,
      deleteTask,
      setSelectedTask,
      saveAllocatorAllTasks,
      taskDetails,
      setTaskDetails,
    }),
    [
      isMobile,
      moduleId,
      assignmentId,
      selectedId,
      loading,
      tasks,
      selectedTask,
      editedName,
      editedCommand,
      editedCoverage,
      refreshTasks,
      createNewTask,
      saveTask,
      deleteTask,
      saveAllocatorAllTasks,
      taskDetails,
    ],
  );

  return <TasksPageContext.Provider value={value}>{children}</TasksPageContext.Provider>;
};
