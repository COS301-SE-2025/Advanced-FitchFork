export type PostTaskRequest = {
  task_number: number;
  name: string;
  command: string;
  code_coverage: boolean;
};