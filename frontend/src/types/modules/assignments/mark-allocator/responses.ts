import type { ApiResponse } from "@/types/common";
import type { MarkAllocatorItem } from "./shared";

// ─────────────────────────────────────────────────────────────
// GET Responses Types
// ─────────────────────────────────────────────────────────────

export type GetMarkAllocatorResponse = ApiResponse<{generated_at: string;tasks:MarkAllocatorItem[]}>;

// ─────────────────────────────────────────────────────────────
// POST Responses Types
// ─────────────────────────────────────────────────────────────

export type PostGenerateMarkAllcatorReponse = ApiResponse<null>;