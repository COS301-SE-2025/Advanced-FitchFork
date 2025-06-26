export const LANGUAGE_OPTIONS = ['python', 'cpp', 'java'] as const;
export type Language = typeof LANGUAGE_OPTIONS[number];

export type MarkingScheme = 'exact' | 'percentage' | 'regex';
export type FeedbackScheme = 'auto' | 'manual' | 'ai';

export interface AssignmentConfig {
  timeout_secs: number;
  max_memory: string;              // e.g. "256m"
  max_cpus: string;                // e.g. "1.5"
  max_uncompressed_size: number;
  max_processes: number;
  marking_scheme: MarkingScheme;
  feedback_scheme: FeedbackScheme;

  // Optional frontend extension
  languages?: Language[];
}
