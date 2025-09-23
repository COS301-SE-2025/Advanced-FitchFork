export interface MarkAllocatorSubsection {
  name: string;
  value: number;
  /**
   * Optional free-text feedback. If omitted, treat as "" in UI where needed.
   */
  feedback?: string;
  /**
   * Optional array of regex strings.
   * When the server runs in Regex scheme, this will exist and its length
   * will equal `value` (one entry per output line, possibly empty strings).
   * Otherwise it may be undefined.
   */
  regex?: string[];
}

export interface MarkAllocatorTask {
  /**
   * e.g. 1 for “task1”. This comes from the DB/task definition, not the array index.
   */
  task_number: number;
  name: string;
  value: number;
  /**
   * Present when the task is a coverage-only task.
   */
  code_coverage?: boolean;
  subsections: MarkAllocatorSubsection[];
}

export interface MarkAllocatorFile {
  /**
   * ISO-8601 timestamp; always present from the API.
   */
  generated_at: string;
  /**
   * Normalized list of tasks; no { "taskX": { … } } indirection anymore.
   */
  tasks: MarkAllocatorTask[];
  /**
   * Sum of all task `value`s.
   */
  total_value: number;
}
