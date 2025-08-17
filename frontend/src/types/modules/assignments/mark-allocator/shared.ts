export interface MarkAllocatorSubsection {
  name: string;
  value: number;
}

export interface MarkAllocatorTaskEntry {
  [taskKey: string]: {
    task_number: number;
    name: string;
    value: number;
    subsections: MarkAllocatorSubsection[];
  };
}

export interface MarkAllocatorFile {
  generated_at?: string;          // optional ISO-8601 timestamp
  tasks: MarkAllocatorTaskEntry[]; // list of { "taskX": { … } } objects
  total_value: number;            // sum of all task values
}
