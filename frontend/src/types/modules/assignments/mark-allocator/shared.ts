export interface MarkAllocatorSubsection {
  name: string;
  value: number;
}

export interface MarkAllocatorTask {
  name: string;
  value: number;
  subsections: MarkAllocatorSubsection[];
}

// one object per array element, key is "task1", "task2", ...
export type MarkAllocatorItem = {
  [taskKey: string]: MarkAllocatorTask;
};

export interface MarkAllocatorFile {
  generated_at?: string;
  tasks: MarkAllocatorItem[];
}
