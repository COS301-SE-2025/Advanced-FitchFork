import type { User } from "../users";

export type UserPayload = Omit<User, "id" | "created_at" | "updated_at">;


// ─────────────────────────────────────────────────────────────
// POST Request Types
// ─────────────────────────────────────────────────────────────

export type CreateUserPayload = {
  username: string;
  email: string;
  password: string;
};

export type BulkCreateUserPayload = CreateUserPayload[];


// ─────────────────────────────────────────────────────────────
// PUT Request Types
// ─────────────────────────────────────────────────────────────

export type PutUserRequest = UserPayload;