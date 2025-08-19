import type { ApiResponse } from "@/types/common";
import type { SubsectionDetail, Task } from "./shared";

// ─────────────────────────────────────────────────────────────
// GET Responses Types
// ─────────────────────────────────────────────────────────────

export type GetListTasksResponse = ApiResponse<Task[]>;

export type GetTaskResponse = ApiResponse<
  {
    name?: string;
    value?: number;
    memo_output?: string;
    subsections: SubsectionDetail[];
  } & Task
>;

// ─────────────────────────────────────────────────────────────
// POST Responses Types
// ─────────────────────────────────────────────────────────────

export type PostTaskResponse = ApiResponse<Task>;

// ─────────────────────────────────────────────────────────────
// PUT Responses Types
// ─────────────────────────────────────────────────────────────

export type PutEditTaskResponse = ApiResponse<Task>;

// ─────────────────────────────────────────────────────────────
// DELETE Responses Types
// ─────────────────────────────────────────────────────────────

export type DeleteTaskResponse = ApiResponse<null>;
