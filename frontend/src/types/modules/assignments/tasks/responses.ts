import type { ApiResponse } from "@/types/common";
import type { Task } from "./shared";

export type PostTaskResponse = ApiResponse<Task>;

export type GetListTasksResponse = ApiResponse<Task[]>;