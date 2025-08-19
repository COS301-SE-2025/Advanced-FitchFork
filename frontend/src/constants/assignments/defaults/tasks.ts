import type { Language } from './languages';

export type StarterTask = { name: string; command: string };

/**
 * Language-specific default tasks
 */
export const DEFAULT_TASKS_JAVA: StarterTask[] = [
  { name: 'Task 1', command: 'make task1' },
  { name: 'Task 2', command: 'make task2' },
  { name: 'Task 3', command: 'make task3' },
];

export const DEFAULT_TASKS_CPP: StarterTask[] = [
  { name: 'Task 1', command: 'make task1' },
  { name: 'Task 2', command: 'make task2' },
  { name: 'Task 3', command: 'make task3' },
  { name: 'Task 4', command: 'make task4' },
];

/** Lookup table by language */
export const LANGUAGE_TASKS: Record<Language, StarterTask[]> = {
  java: DEFAULT_TASKS_JAVA,
  cpp: DEFAULT_TASKS_CPP,
};
