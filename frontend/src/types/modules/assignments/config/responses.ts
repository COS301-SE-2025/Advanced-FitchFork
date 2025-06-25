import type { ApiResponse } from "@/types/common";
import type { ExecutionConfig } from "./shared";

// ─────────────────────────────────────────────────────────────
// GET Responses Types
// ─────────────────────────────────────────────────────────────

export type GetExecutionConfigResponse = ApiResponse<ExecutionConfig>;
