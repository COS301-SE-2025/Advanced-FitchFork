// src/pages/modules/assignments/Tasks/context.tsx
import React, { createContext, useContext, useEffect, useMemo, useState, useCallback } from 'react';
import { useLocation, useNavigate } from 'react-router-dom';
import { Grid } from 'antd';

import type { Task } from '@/types/modules/assignments/tasks';
import type { GetTaskResponse } from '@/types/modules/assignments/tasks/responses';
import type {
  MarkAllocatorFile,
  MarkAllocatorSubsection,
  MarkAllocatorTask,
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
import { fetchAssignmentFileBlob } from '@/services/modules/assignments';
import {
  parseTargetsFromMakefileZip,
  createTasksFromMakefileTargets,
} from '@/utils/makefile_tasks';
import { useBreadcrumbContext } from '@/context/BreadcrumbContext';

function normalizeSubsections(subs: any[]) {
  if (!subs) return [];
  return subs.map((s: any) => {
    const rawMemo: string | null | undefined = s?.memo_output ?? s?.output;
    const memo_output = typeof rawMemo === 'string' ? rawMemo : null;

    const feedback: string = typeof s?.feedback === 'string' ? s.feedback : '';

    // Type the items coming from s.regex as unknown, then narrow to string
    const regex: string[] | undefined = Array.isArray(s?.regex)
      ? (s.regex as unknown[]).map((r: unknown) => (typeof r === 'string' ? r : ''))
      : undefined;

    return {
      name: s?.name ?? s?.label ?? 'Unnamed',
      value: Number.isFinite(Number(s?.value)) ? parseInt(String(s.value), 10) : 0,
      memo_output,
      feedback,
      ...(Array.isArray(regex) ? { regex } : {}),
    };
  });
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
  hasMakefile: boolean;
  generateTasksFromMakefile: () => Promise<void>;
  generatingFromMakefile: boolean;

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
  const { assignment, assignmentFiles, readiness, refreshAssignment } = useAssignment();
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
        message.error(res.message);
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
    } catch (e) {
      console.error(e);
      message.error('Failed to load tasks');
    } finally {
      setLoading(false);
    }
  }, [moduleId, assignmentId]);

  // initial + dependency refresh
  useEffect(() => {
    if (!moduleId || !assignmentId) return;
    refreshTasks();
  }, [moduleId, assignmentId, refreshTasks]);

  useEffect(() => {
    if (!isMobile && location.pathname.endsWith('/tasks') && tasks.length > 0) {
      setSelectedId(tasks[0].id);
    }
  }, [isMobile, location.pathname, tasks, setSelectedId]);

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

  const makefileFile = useMemo(
    () => assignmentFiles?.find((f) => f.file_type === 'makefile') ?? null,
    [assignmentFiles],
  );
  const hasMakefile = Boolean(readiness?.makefile_present && makefileFile);
  const [generatingFromMakefile, setGeneratingFromMakefile] = useState(false);

  const generateTasksFromMakefile = useCallback(async () => {
    if (!moduleId || !assignmentId || !makefileFile) {
      message.info('Upload a Makefile in Files & Config before generating tasks.');
      return;
    }

    if (generatingFromMakefile) return;

    setGeneratingFromMakefile(true);
    try {
      const blob = await fetchAssignmentFileBlob(moduleId, assignmentId, makefileFile.id);
      const file = new File([blob], makefileFile.filename, {
        type: blob.type || 'application/zip',
      });

      const targets = await parseTargetsFromMakefileZip(file);
      if (!targets.length) {
        message.info('No runnable targets were detected in the Makefile.');
        return;
      }

      const created = await createTasksFromMakefileTargets(
        moduleId,
        assignmentId,
        targets,
        refreshAssignment,
      );

      if (created > 0) {
        await refreshTasks();
        message.success(`Generated ${created} task${created === 1 ? '' : 's'} from the Makefile.`);
      } else {
        message.info('No new tasks were created from the Makefile.');
      }
    } catch (err) {
      message.error('Failed to generate tasks from the Makefile.');
      // eslint-disable-next-line no-console
      console.error(err);
    } finally {
      setGeneratingFromMakefile(false);
    }
  }, [
    moduleId,
    assignmentId,
    makefileFile,
    generatingFromMakefile,
    refreshTasks,
    refreshAssignment,
  ]);

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

    // Ensure we have all details in memory
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
        return;
      }
    }

    // Build normalized tasks array
    const byNumber = [...tasks].sort((a, b) => a.task_number - b.task_number);
    const tasksPayload: MarkAllocatorTask[] = byNumber.map((t) => {
      const full = t.id === selectedTask?.id ? selectedTask : taskDetails[t.id];

      const subsections: MarkAllocatorSubsection[] = (full?.subsections ?? []).map((s: any) => {
        const out: MarkAllocatorSubsection = {
          name: s.name,
          // value is the allocated marks, leave it as-is (independent of regex)
          value: Number.isFinite(Number(s.value)) ? parseInt(String(s.value), 10) : 0,
        };

        if (typeof s.feedback === 'string') out.feedback = s.feedback;

        // Size regex array to the number of memo lines – not to `value`
        const memo = typeof s.memo_output === 'string' ? s.memo_output : '';
        const lineCount = memo.replace(/\r\n/g, '\n').replace(/\r/g, '\n').split('\n').length;

        if (Array.isArray(s.regex)) {
          const padded = [...s.regex];
          while (padded.length < lineCount) padded.push('');
          out.regex = padded.slice(0, lineCount).map((r) => (typeof r === 'string' ? r : ''));
        } else {
          out.regex = Array.from({ length: lineCount }, () => '');
        }

        return out;
      });

      const value = subsections.reduce((sum, c) => sum + c.value, 0);
      const name = (full?.name ?? t.name) || `Task ${t.task_number}`;

      return {
        task_number: t.task_number,
        name,
        value,
        subsections,
        ...(typeof full?.code_coverage === 'boolean'
          ? { code_coverage: !!full.code_coverage }
          : {}),
      };
    });

    const totalValue = tasksPayload.reduce((sum, task) => sum + (task?.value ?? 0), 0);

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
      hasMakefile,
      generateTasksFromMakefile,
      generatingFromMakefile,
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
      hasMakefile,
      generateTasksFromMakefile,
      generatingFromMakefile,
      saveAllocatorAllTasks,
      taskDetails,
      setSelectedId,
      setEditedName,
      setEditedCommand,
      setEditedCoverage,
      setSelectedTask,
      setTaskDetails,
    ],
  );

  return <TasksPageContext.Provider value={value}>{children}</TasksPageContext.Provider>;
};
