export interface MarkAllocatorSubsection {
  name: string;
  value: number;
}

export interface MarkAllocatorTask {
  name: string;
  value: number;
  subsections?: MarkAllocatorSubsection[];
}

export type MarkAllocatorItem = {
  [taskKey: string]: MarkAllocatorTask;
};