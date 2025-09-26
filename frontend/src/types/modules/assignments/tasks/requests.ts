import type { TaskType } from "./shared";

export type PostTaskRequest = {
  task_number: number;
  name: string;
  command: string;
  task_type: TaskType;
};