export const LANGUAGE_OPTIONS = ['python', 'cpp', 'java'] as const;

export type Languages = typeof LANGUAGE_OPTIONS[number];

export interface AssignmentConfig {
  timeout_seconds?: number;
  max_memory?: number;
  max_cpus?: number;
  max_uncompressed_size?: number;
  max_processors?: number;
  languages?: Languages;
}
