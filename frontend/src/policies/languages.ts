// src/policies/languages.ts
import type { Language } from '@/types/modules/assignments/config';
import type { TaskType } from '@/types/modules/assignments/tasks';

export const COVERAGE_LANGS: Readonly<Language[]> = ['cpp', 'java'] as const;
export const VALGRIND_LANGS: Readonly<Language[]> = ['cpp'] as const;

export const isCoverageSupported = (lang?: Language | null) =>
  !!lang && (COVERAGE_LANGS as readonly string[]).includes(lang);

export const isValgrindSupported = (lang?: Language | null) =>
  !!lang && (VALGRIND_LANGS as readonly string[]).includes(lang);

export const taskTypeLabel = (t: TaskType) =>
  t === 'normal' ? 'Normal' : t === 'coverage' ? 'Code Coverage' : 'Memory Leak Test';

/** Segmented options filtered by language */
export const taskTypeOptionsForLanguage = (lang?: Language | null) => {
  const opts: { label: string; value: TaskType }[] = [{ label: taskTypeLabel('normal'), value: 'normal' }];
  if (isCoverageSupported(lang)) opts.push({ label: taskTypeLabel('coverage'), value: 'coverage' });
  if (isValgrindSupported(lang)) opts.push({ label: taskTypeLabel('valgrind'), value: 'valgrind' });
  return opts;
};
