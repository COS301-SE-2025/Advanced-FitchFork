import type { ApiResponse } from "@/types/common";
import type { MarkAllocatorItem } from "./shared";

// ─────────────────────────────────────────────────────────────
// GET Responses Types
// ─────────────────────────────────────────────────────────────

export type GetMarkAllocatorResponse = ApiResponse<MarkAllocatorItem[]>;
