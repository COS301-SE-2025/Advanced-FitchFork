export interface MemoSubsection {
  label: string;
  output: string;
}

export interface MemoTaskOutput {
  task_number: number;
  name: string;
  subsections: MemoSubsection[];
}