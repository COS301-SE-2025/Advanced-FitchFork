import type { Module } from "./shared";

// ─────────────────────────────────────────────────────────────
// POST Request Types
// ─────────────────────────────────────────────────────────────

export type PostModuleRequest = Omit<Module, "id" | "created_at" | "updated_at">;
export interface PostAssignPersonnelRequest { user_ids: number[]; }

// ─────────────────────────────────────────────────────────────
// PUT Request Types
// ─────────────────────────────────────────────────────────────

export type PutModuleRequest = Omit<Module, "id" | "created_at" | "updated_at">;
