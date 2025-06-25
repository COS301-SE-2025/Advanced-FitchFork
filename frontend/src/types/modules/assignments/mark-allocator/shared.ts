export interface SubsectionAllocator {
  name: string;
  value: number;
}

export interface TaskAllocator {
  name: string;
  value: number;
  subsections?: SubsectionAllocator[];
}

export type MarkAllocatorItem = {
  [taskKey: string]: TaskAllocator;
};