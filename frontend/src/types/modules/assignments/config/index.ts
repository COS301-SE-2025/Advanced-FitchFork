import type { ApiResponse } from "@/types/common";

export interface ExecutionConfig {
  timeout_secs: number;
  max_memory: string;
  max_cpus: string;
  max_uncompressed_size: number;
  max_processes: number;
  language: string;
}

// The backend returns an empty object {} if no config is set,
// so this allows both full and empty configs.
export type GetExecutionConfigResponse = ApiResponse<Partial<ExecutionConfig>>;
