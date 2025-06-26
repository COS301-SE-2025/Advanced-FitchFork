

// ─────────────────────────────────────────────────────────────
// GET Responses Types
// ─────────────────────────────────────────────────────────────

import type { ApiResponse } from "@/types/common";
import type { MemoTaskOutput } from "./shared";

export type GetMemoOutputResponse = ApiResponse<MemoTaskOutput[]>;