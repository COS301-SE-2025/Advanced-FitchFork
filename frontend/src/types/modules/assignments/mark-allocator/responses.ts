import type { ApiResponse } from "@/types/common";
import type { MarkAllocatorFile } from "./shared";

// ─────────────────────────────────────────────────────────────
// GET Response Types
// ─────────────────────────────────────────────────────────────

export type GetMarkAllocatorResponse = ApiResponse<MarkAllocatorFile>;

// ─────────────────────────────────────────────────────────────
// POST Response Types
// ─────────────────────────────────────────────────────────────

export type PostGenerateMarkAllocatorResponse = ApiResponse<MarkAllocatorFile>;
