import type { User } from "../users";

export type UserPayload = Omit<User, "id" | "created_at" | "updated_at">;

// ─────────────────────────────────────────────────────────────
// PUT Request Types
// ─────────────────────────────────────────────────────────────

export type PutUserRequest = UserPayload;